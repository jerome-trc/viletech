//! Internal data preparation functions.
//!
//! After mounting is done, start composing useful objects from raw files.

mod level;
mod udmf;
mod vanilla;
mod wad;

use std::{any::Any, sync::Arc};

use bevy::prelude::info;
use parking_lot::Mutex;
use rayon::prelude::*;
use smallvec::smallvec;

use crate::{data::dobj::DATUM_TYPE_NAMES, vzs, Id8, VPathBuf};

use self::vanilla::{PatchTable, TextureX};

use super::{
	detail::{DatumKey, Outcome},
	dobj::{DatumStore, Store},
	Catalog, Datum, LoadTracker, MountInfo, MountKind, PrepError, PrepErrorKind, WadExtras,
};

#[derive(Debug)]
pub(super) struct Context {
	pub(super) tracker: Arc<LoadTracker>,
	/// Returning errors through the prep call tree is somewhat
	/// inflexible, so pass an array down through the context instead.
	pub(super) errors: Vec<Mutex<Vec<PrepError>>>,
}

impl Context {
	#[must_use]
	pub(super) fn new(tracker: Arc<LoadTracker>, mounts_len: usize) -> Self {
		Self {
			tracker,
			errors: {
				let mut ret = vec![];
				ret.resize_with(mounts_len, || Mutex::new(vec![]));
				ret
			},
		}
	}

	#[must_use]
	pub(super) fn any_fatal_errors(&self) -> bool {
		self.errors
			.par_iter()
			.any(|mutex| mutex.lock().iter().any(|err| err.is_fatal()))
	}

	#[must_use]
	pub(super) fn into_errors(mut self) -> Vec<Vec<PrepError>> {
		std::mem::take(&mut self.errors)
			.into_iter()
			.map(|mutex| mutex.into_inner())
			.collect()
	}
}

/// Context relevant to operations on one mount.
#[derive(Debug)]
pub(self) struct SubContext<'ctx> {
	pub(self) tracker: &'ctx Arc<LoadTracker>,
	pub(self) mntinfo: &'ctx MountInfo,
	pub(self) artifacts: &'ctx Mutex<Artifacts>,
	pub(self) errors: &'ctx Mutex<Vec<PrepError>>,
}

#[derive(Debug, Default)]
pub(self) struct Artifacts {
	pub(self) objs: Vec<StagedDatum>,
	pub(self) extras: WadExtras,
	pub(self) pnames: Option<PatchTable>,
	pub(self) texture1: Option<TextureX>,
	pub(self) texture2: Option<TextureX>,
}

#[derive(Debug)]
pub(self) struct StagedDatum {
	key: DatumKey,
	key_nick: DatumKey,
	datum: Arc<dyn DatumStore>,
}

impl SubContext<'_> {
	pub(self) fn add_datum<D: Datum>(&self, datum: D, id_suffix: impl AsRef<str>) {
		let id = format!("{}/{}", self.mntinfo.id(), id_suffix.as_ref());

		self.artifacts.lock().objs.push(StagedDatum {
			key: DatumKey::new::<D>(&id),
			key_nick: DatumKey::new::<D>(id.split('/').last().unwrap()),
			datum: Arc::new(Store::new(id, datum)),
		});
	}
}

#[derive(Debug)]
#[must_use]
pub(super) struct Output {
	/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
	pub(super) errors: Vec<Vec<PrepError>>,
}

impl Catalog {
	/// Preconditions:
	/// - `self.files` has been populated. All directories know their contents.
	/// - `self.mounts` has been populated.
	/// - Load tracker has already had its prep target number set.
	/// - `ctx.errors` has been populated.
	pub(super) fn prep(&mut self, ctx: Context) -> Outcome<Output, Vec<Vec<PrepError>>> {
		let to_reserve = ctx.tracker.prep_target();
		debug_assert!(!ctx.errors.is_empty());
		debug_assert!(to_reserve > 0);

		if let Err(err) = self.objs.try_reserve(to_reserve) {
			panic!("Failed to reserve memory for approx. {to_reserve} new assets. Error: {err:?}",);
		}

		let mut artifacts = vec![];
		artifacts.resize_with(self.mounts.len(), || Mutex::new(Artifacts::default()));

		// Pass 1: compile VZS; transpile EDF and (G)ZDoom DSLs.

		for (i, mount) in self.mounts.iter().enumerate() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				tracker: &ctx.tracker,
				mntinfo: &mount.info,
				artifacts: &artifacts[i],
				errors: &ctx.errors[i],
			};

			let _ = match subctx.mntinfo.kind {
				MountKind::VileTech => self.prep_pass1_vpk(&subctx),
				MountKind::ZDoom => self.prep_pass1_pk(&subctx),
				MountKind::Eternity => todo!(),
				MountKind::Wad => self.prep_pass1_wad(&subctx),
				MountKind::Misc => self.prep_pass1_file(&subctx),
			};
		}

		if ctx.any_fatal_errors() {
			ctx.tracker.finish_prep();
			return Outcome::Err(ctx.into_errors());
		}

		// Pass 2: dependency-free assets; trivial to parallelize. Includes:
		// - Palettes and colormaps.
		// - Sounds and music.
		// - Non-picture-format images.

		for (i, mount) in self.mounts.iter().enumerate() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				tracker: &ctx.tracker,
				mntinfo: &mount.info,
				artifacts: &artifacts[i],
				errors: &ctx.errors[i],
			};

			match subctx.mntinfo.kind {
				MountKind::Wad => self.prep_pass2_wad(&subctx),
				MountKind::VileTech => {} // Soon!
				_ => unimplemented!("Soon!"),
			}
		}

		if ctx.any_fatal_errors() {
			ctx.tracker.finish_prep();
			return Outcome::Err(ctx.into_errors());
		}

		self.register_artifacts(&artifacts);

		if self.last_paletteset().is_none() {
			unimplemented!("Further loading without a PLAYPAL is unsupported for now.");
		}

		// Pass 3: assets dependent on pass 2. Includes:
		// - Picture-format images, which need palettes.
		// - Maps, which need textures, music, scripts, blueprints...

		for (i, mount) in self.mounts.iter().enumerate() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				tracker: &ctx.tracker,
				mntinfo: &mount.info,
				artifacts: &artifacts[i],
				errors: &ctx.errors[i],
			};

			let _outcome = match subctx.mntinfo.kind {
				MountKind::Wad => self.prep_pass3_wad(&subctx),
				MountKind::VileTech => Outcome::None, // Soon!
				_ => unimplemented!("Soon!"),
			};
		}

		if ctx.any_fatal_errors() {
			ctx.tracker.finish_prep();
			return Outcome::Err(ctx.into_errors());
		}

		self.register_artifacts(&artifacts);

		// TODO: Make each successfully processed file increment progress.
		ctx.tracker.finish_prep();

		info!("Loading complete.");

		Outcome::Ok(Output {
			errors: ctx.into_errors(),
		})
	}

	/// Try to compile non-ACS scripts from this package. VZS, EDF, and (G)ZDoom
	/// DSLs all go into the same VZS module, regardless of which are present
	/// and which are absent.
	fn prep_pass1_vpk(&self, ctx: &SubContext) -> Outcome<(), ()> {
		if let Some(vzscript) = &ctx.mntinfo.vzscript {
			let root_dir_path: VPathBuf = [ctx.mntinfo.virtual_path(), &vzscript.root_dir]
				.iter()
				.collect();

			let root_dir = match self.vfs.get(&root_dir_path) {
				Some(fref) => fref,
				None => {
					ctx.errors.lock().push(PrepError {
						path: ctx.mntinfo.virtual_path().join(&vzscript.root_dir),
						kind: PrepErrorKind::MissingVzsDir,
					});

					return Outcome::Err(());
				}
			};

			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let inctree = vzs::IncludeTree::new(root_dir);

			if inctree.any_errors() {
				let mut errors = ctx.errors.lock();
				let ptrees = inctree.into_inner();

				for ptree in ptrees {
					let path = ptree.path;

					for err in ptree.inner.errors {
						errors.push(PrepError {
							path: path.clone(),
							kind: PrepErrorKind::VzsParse(err),
						});
					}
				}
			}

			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}
		}

		Outcome::None
	}

	fn prep_pass1_file(&self, ctx: &SubContext) -> Outcome<(), ()> {
		let file = self.vfs.get(ctx.mntinfo.virtual_path()).unwrap();

		// Pass 1 only deals in text files.
		if !file.is_text() {
			return Outcome::None;
		}

		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
		}

		if file
			.path_extension()
			.filter(|p_ext| p_ext.eq_ignore_ascii_case("vzs"))
			.is_some()
		{
			unimplemented!();
		} else if file.file_prefix().eq_ignore_ascii_case("decorate") {
			unimplemented!();
		} else if file.file_prefix().eq_ignore_ascii_case("zscript") {
			unimplemented!();
		} else if file.file_prefix().eq_ignore_ascii_case("edfroot") {
			unimplemented!();
		}

		Outcome::None
	}

	fn prep_pass1_pk(&self, _ctx: &SubContext) -> Outcome<(), ()> {
		// TODO: Soon!
		Outcome::None
	}

	// Common functions ////////////////////////////////////////////////////////

	fn register_artifacts(&mut self, staging: &[Mutex<Artifacts>]) {
		for (i, mutex) in staging.iter().enumerate() {
			let mut artifacts = mutex.lock();
			let slotmap = &mut self.mounts[i].objs;
			slotmap.reserve(artifacts.objs.len());

			artifacts.objs.drain(..).for_each(
				|StagedDatum {
				     key: id,
				     key_nick: nick,
				     datum,
				 }| {
					let lookup = self.objs.entry(id);

					if matches!(lookup, dashmap::mapref::entry::Entry::Occupied(_)) {
						info!(
							"Overwriting: {} ({})",
							datum.id(),
							DATUM_TYPE_NAMES.get(&datum.type_id()).unwrap(),
						);
					}

					let slotkey = slotmap.insert(datum);

					if let Some(mut kvp) = self.nicknames.get_mut(&nick) {
						kvp.value_mut().push((i, slotkey));
					} else {
						self.nicknames.insert(nick, smallvec![(i, slotkey)]);
					};

					match lookup {
						dashmap::mapref::entry::Entry::Occupied(mut occu) => {
							occu.insert((i, slotkey));
						}
						dashmap::mapref::entry::Entry::Vacant(vacant) => {
							vacant.insert((i, slotkey));
						}
					}
				},
			);

			if let Some(colormap) = artifacts.extras.colormap.take() {
				self.mounts[i].extras.colormap = Some(colormap);
			}

			if let Some(endoom) = artifacts.extras.endoom.take() {
				self.mounts[i].extras.endoom = Some(endoom);
			}

			if let Some(palset) = artifacts.extras.palset.take() {
				self.mounts[i].extras.palset = Some(palset);
			}
		}
	}
}

/// Returns `None` if `id8` starts with a NUL.
/// Return values have no trailing NUL bytes.
#[must_use]
pub(self) fn read_id8(id8: [u8; 8]) -> Option<Id8> {
	if id8.starts_with(&[b'\0']) {
		return None;
	}

	let mut ret = Id8::new();

	for byte in id8 {
		if byte == b'\0' {
			break;
		}

		ret.push(char::from(byte));
	}

	Some(ret)
}
