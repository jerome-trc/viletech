//! Internal asset preparation functions.
//!
//! After mounting is done, start composing useful assets from raw files.

mod level;

use std::{io::Cursor, sync::Arc};

use arrayvec::ArrayVec;
use bevy::prelude::info;
use byteorder::ReadBytesExt;
use image::Rgba;
use parking_lot::Mutex;
use rayon::prelude::*;
use smallvec::smallvec;

use crate::{vzs, VPathBuf};

use super::{
	detail::{AssetKey, MountSlotKey, Outcome},
	Asset, AssetHeader, Audio, Catalog, FileRef, Image, LoadTracker, MountInfo, MountKind, Palette,
	PaletteSet, PrepError, PrepErrorKind,
};

#[derive(Debug)]
pub(super) struct Context {
	pub(super) tracker: Arc<LoadTracker>,
	/// Returning errors through the asset prep call tree is somewhat
	/// inflexible, so pass an array down through the context instead.
	pub(super) errors: Vec<Mutex<Vec<PrepError>>>,
	pub(super) new_mounts: Vec<MountSlotKey>,
}

impl Context {
	#[must_use]
	pub(super) fn new(tracker: Arc<LoadTracker>, new_mounts: Vec<MountSlotKey>) -> Self {
		Self {
			tracker,
			errors: {
				let mut ret = vec![];
				ret.resize_with(new_mounts.len(), || Mutex::new(vec![]));
				ret
			},
			new_mounts,
		}
	}
}

/// Context relevant to operations on one mount.
#[derive(Debug)]
pub(self) struct SubContext<'ctx> {
	pub(self) tracker: &'ctx Arc<LoadTracker>,
	pub(self) mntinfo: &'ctx MountInfo,
	pub(self) assets: &'ctx Mutex<Vec<StagedAsset>>,
	pub(self) errors: &'ctx Mutex<Vec<PrepError>>,
}

#[derive(Debug)]
pub(self) struct StagedAsset {
	key_full: AssetKey,
	key_nick: AssetKey,
	asset: Arc<dyn Asset>,
}

impl SubContext<'_> {
	pub(self) fn add_asset<A: Asset>(&self, asset: A) {
		let nickname = asset.header().nickname();
		let key_full = AssetKey::new::<A>(&asset.header().id);
		let key_nick = AssetKey::new::<A>(nickname);

		self.assets.lock().push(StagedAsset {
			key_full,
			key_nick,
			asset: Arc::new(asset),
		});
	}
}

#[derive(Debug)]
#[must_use]
pub(super) struct Output {
	/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
	pub(super) errors: Vec<Vec<PrepError>>,
}

impl Output {
	#[must_use]
	pub(super) fn any_errs(&self) -> bool {
		self.errors.iter().any(|res| !res.is_empty())
	}
}

impl Catalog {
	/// Preconditions:
	/// - `self.files` has been populated. All directories know their contents.
	/// - `self.mounts` has been populated.
	/// - Load tracker has already had its asset prep target number set.
	/// - `ctx.errors` has been populated.
	pub(super) fn prep(&mut self, mut ctx: Context) -> Outcome<Output, Vec<Vec<PrepError>>> {
		let to_reserve = ctx.tracker.prep_target();
		debug_assert!(!ctx.errors.is_empty());
		debug_assert!(to_reserve > 0);

		if let Err(err) = self.assets.try_reserve(to_reserve) {
			panic!("Failed to reserve memory for approx. {to_reserve} new assets. Error: {err:?}",);
		}

		let mut staging = vec![];
		staging.resize_with(ctx.new_mounts.len(), || Mutex::new(vec![]));

		// Pass 1: compile VZS; transpile EDF and (G)ZDoom DSLs.

		for (i, mount_key) in ctx.new_mounts.iter().enumerate() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				tracker: &ctx.tracker,
				mntinfo: &self.mounts[*mount_key].info,
				assets: &staging[i],
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

		// Pass 2: dependency-free assets; trivial to parallelize. Includes:
		// - Palettes and colormaps.
		// - Sounds and music.
		// - Non-picture-format images.

		for (i, mount_key) in ctx.new_mounts.iter().enumerate() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				tracker: &ctx.tracker,
				mntinfo: &self.mounts[*mount_key].info,
				assets: &staging[i],
				errors: &ctx.errors[i],
			};

			match subctx.mntinfo.kind {
				MountKind::Wad => self.prep_pass2_wad(&subctx),
				MountKind::VileTech => {} // Soon!
				_ => unimplemented!("Soon!"),
			}
		}

		// TODO: Forbid further loading without a PLAYPAL present?

		self.register_assets(&ctx.new_mounts, &staging);

		// Pass 3: assets dependent on pass 2. Includes:
		// - Picture-format images, which need palettes.
		// - Maps, which need textures, music, scripts, blueprints...

		for (i, mount_key) in ctx.new_mounts.iter().enumerate() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				tracker: &ctx.tracker,
				mntinfo: &self.mounts[*mount_key].info,
				assets: &staging[i],
				errors: &ctx.errors[i],
			};

			let _outcome = match subctx.mntinfo.kind {
				MountKind::Wad => self.prep_pass3_wad(&subctx),
				MountKind::VileTech => Outcome::None, // Soon!
				_ => unimplemented!("Soon!"),
			};
		}

		self.register_assets(&ctx.new_mounts, &staging);

		let mut errors: Vec<Vec<PrepError>> = std::mem::take(&mut ctx.errors)
			.into_iter()
			.map(|mutex| mutex.into_inner())
			.collect();

		for subvec in &mut errors {
			subvec.sort_by(|err1, err2| err1.path.cmp(&err2.path));
		}

		let ret = Output { errors };

		if ret.any_errs() {
			self.on_prep_fail();
		} else {
			// TODO: Make each successfully processed file increment progress.
			ctx.tracker.finish_prep();
			info!("Loading complete.");
		}

		Outcome::Ok(ret)
	}

	/// Try to compile non-ACS scripts from this package. VZS, EDF, and (G)ZDoom
	/// DSLs all go into the same VZS module, regardless of which are present
	/// and which are absent.
	fn prep_pass1_vpk(&self, ctx: &SubContext) -> Outcome<(), ()> {
		let script_root: VPathBuf = if let Some(srp) = &ctx.mntinfo.script_root {
			[ctx.mntinfo.virtual_path(), srp].iter().collect()
		} else {
			todo!()
		};

		let script_root = match self.vfs.get(&script_root) {
			Some(fref) => fref,
			None => {
				ctx.errors.lock().push(PrepError {
					path: script_root.to_path_buf(),
					kind: PrepErrorKind::MissingScriptRoot,
				});

				return Outcome::Err(());
			}
		};

		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
		}

		let inctree = vzs::IncludeTree::new(ctx.mntinfo.virtual_path(), script_root);

		if inctree.any_errors() {
			unimplemented!("Soon!");
		}

		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
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

	fn prep_pass1_wad(&self, _ctx: &SubContext) -> Outcome<(), ()> {
		// TODO: Soon!
		Outcome::None
	}

	fn prep_pass2_wad(&self, ctx: &SubContext) {
		let wad = self.vfs.get(ctx.mntinfo.virtual_path()).unwrap();

		wad.children().unwrap().par_bridge().try_for_each(|child| {
			if !child.is_readable() {
				return Some(());
			}

			if ctx.tracker.is_cancelled() {
				return None;
			}

			let bytes = child.read_bytes();
			let fstem = child.file_prefix();

			let res = if fstem.starts_with("PLAYPAL") {
				self.prep_playpal(ctx, bytes)
			} else {
				return Some(());
			};

			match res {
				Ok(()) => Some(()),
				Err(err) => {
					ctx.errors.lock().push(PrepError {
						path: child.path().to_path_buf(),
						kind: PrepErrorKind::Io(err),
					});

					Some(())
				}
			}
		});
	}

	fn prep_pass3_wad(&self, ctx: &SubContext) -> Outcome<(), ()> {
		let wad = self.vfs.get(ctx.mntinfo.virtual_path()).unwrap();

		let cancelled = wad
			.child_refs()
			.unwrap()
			.filter(|c| !c.is_empty())
			.par_bridge()
			.try_for_each(|child| {
				if ctx.tracker.is_cancelled() {
					return None;
				}

				if child.is_dir() {
					self.prep_pass3_wad_dir(ctx, child);
				} else {
					self.prep_pass3_wad_entry(ctx, child);
				};

				Some(())
			});

		match cancelled {
			Some(()) => Outcome::Ok(()),
			None => Outcome::Cancelled,
		}
	}

	fn prep_pass3_wad_entry(&self, ctx: &SubContext, vfile: FileRef) {
		let bytes = vfile.read_bytes();
		let fstem = vfile.file_prefix();

		/// Kinds of WAD entries irrelevant to this pass.
		const UNHANDLED: &[&str] = &[
			"COLORMAP", "DMXGUS", "ENDOOM", "GENMIDI", "PLAYPAL", "PNAMES", "TEXTURE1", "TEXTURE2",
		];

		if UNHANDLED.iter().any(|&name| fstem == name)
			|| Audio::is_pc_speaker_sound(bytes)
			|| Audio::is_dmxmus(bytes)
		{
			return;
		}

		ctx.tracker.add_prep_progress(1);

		let is_pic = self.prep_picture(ctx, bytes, fstem);

		// TODO: Processors for more file formats.

		let res: std::io::Result<()> = if is_pic.is_some() {
			Ok(())
		} else {
			return;
		};

		match res {
			Ok(()) => {}
			Err(err) => {
				ctx.errors.lock().push(PrepError {
					path: vfile.path().to_path_buf(),
					kind: PrepErrorKind::Io(err),
				});
			}
		}
	}

	fn prep_pass3_wad_dir(&self, ctx: &SubContext, dir: FileRef) {
		match self.try_prep_level_vanilla(ctx, dir) {
			Outcome::Ok(()) | Outcome::Err(()) => return,
			Outcome::None => {}
			_ => unreachable!(),
		}

		match self.try_prep_level_udmf(ctx, dir) {
			Outcome::Ok(()) | Outcome::Err(()) => {}
			Outcome::None => {}
			_ => unreachable!(),
		}
	}

	fn on_prep_fail(&mut self) {
		self.unload_all();
	}

	// Processors for individual data formats //////////////////////////////////

	/// Returns `None` to indicate that `bytes` was checked
	/// and determined to not be a picture.
	#[must_use]
	fn prep_picture(&self, ctx: &SubContext, bytes: &[u8], id: &str) -> Option<()> {
		// TODO: Wasteful to run a hash lookup before checking if this is a picture.
		let palettes = self.last_asset_by_nick::<PaletteSet>("PLAYPAL").unwrap();

		if let Some(image) = Image::try_from_picture(bytes, &palettes.palettes[0]) {
			ctx.add_asset::<Image>(Image {
				header: AssetHeader {
					id: format!("{mount_id}/{id}", mount_id = ctx.mntinfo.id()),
				},
				inner: image.0,
				offset: image.1,
			});

			Some(())
		} else {
			None
		}
	}

	fn prep_playpal(&self, ctx: &SubContext, bytes: &[u8]) -> std::io::Result<()> {
		let mut palettes = ArrayVec::<_, 14>::default();
		let mut cursor = Cursor::new(bytes);

		for _ in 0..14 {
			let mut pal = Palette::black();

			for ii in 0..256 {
				let r = cursor.read_u8()?;
				let g = cursor.read_u8()?;
				let b = cursor.read_u8()?;
				pal.0[ii] = Rgba([r, g, b, 255]);
			}

			palettes.push(pal);
		}

		ctx.add_asset::<PaletteSet>(PaletteSet {
			header: AssetHeader {
				id: format!("{}/PLAYPAL", ctx.mntinfo.id()),
			},
			palettes: palettes.into_inner().unwrap(),
		});

		Ok(())
	}

	// Common functions ////////////////////////////////////////////////////////

	fn register_assets(
		&mut self,
		new_mounts: &[MountSlotKey],
		staging: &[Mutex<Vec<StagedAsset>>],
	) {
		for (i, mutex) in staging.iter().enumerate() {
			let mount_key = new_mounts[i];
			let mut staged = mutex.lock();
			let slotmap = &mut self.mounts[mount_key].assets;
			slotmap.reserve(staged.len());

			staged.drain(..).for_each(
				|StagedAsset {
				     key_full,
				     key_nick,
				     asset,
				 }| {
					let lookup = self.assets.entry(key_full);

					if matches!(lookup, dashmap::mapref::entry::Entry::Occupied(_)) {
						info!(
							"Overwriting asset: {} type ({})",
							asset.header().id,
							asset.type_name()
						);
					}

					let slotkey = slotmap.insert(asset);

					if let Some(mut kvp) = self.nicknames.get_mut(&key_nick) {
						kvp.value_mut().push((mount_key, slotkey));
					} else {
						self.nicknames
							.insert(key_nick, smallvec![(mount_key, slotkey)]);
					};

					match lookup {
						dashmap::mapref::entry::Entry::Occupied(mut occu) => {
							occu.insert((mount_key, slotkey));
						}
						dashmap::mapref::entry::Entry::Vacant(vacant) => {
							vacant.insert((mount_key, slotkey));
						}
					}
				},
			);
		}
	}
}
