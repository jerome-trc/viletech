//! Internal asset preparation functions.
//!
//! After mounting is done, start composing useful assets from raw files.

mod level;

use std::{io::Cursor, ops::Range, sync::Arc};

use arrayvec::ArrayVec;
use bevy::prelude::info;
use byteorder::ReadBytesExt;
use image::Rgba;
use parking_lot::Mutex;
use rayon::prelude::*;
use slotmap::SlotMap;
use smallvec::smallvec;

use crate::{vzs, VPathBuf};

use super::{
	detail::{AssetKey, AssetSlotKey, Outcome},
	Asset, AssetHeader, Audio, Catalog, FileRef, Image, LoadTracker, MountInfo, MountKind, Palette,
	PaletteSet, PrepError, PrepErrorKind,
};

#[derive(Debug)]
pub(super) struct Context {
	pub(super) tracker: Arc<LoadTracker>,
	pub(super) new_mounts: Range<usize>,
	/// Returning errors through the asset prep call tree is somewhat
	/// inflexible, so pass an array down through the context instead.
	pub(super) errors: Vec<Mutex<Vec<PrepError>>>,
}

impl Context {
	#[must_use]
	pub(super) fn new(tracker: Arc<LoadTracker>, new_mounts: Range<usize>) -> Self {
		Self {
			tracker,
			new_mounts: new_mounts.clone(),
			errors: {
				let new_mount_count = new_mounts.end - new_mounts.start;
				let mut e = Vec::with_capacity(new_mount_count);

				for _ in 0..new_mount_count {
					e.push(Mutex::new(vec![]));
				}

				e
			},
		}
	}
}

/// Context relevant to operations on one mount.
#[derive(Debug)]
pub(self) struct SubContext<'ctx> {
	pub(self) tracker: &'ctx Arc<LoadTracker>,
	pub(self) assets: &'ctx Mutex<SlotMap<AssetSlotKey, Arc<dyn Asset>>>,
	pub(self) i_mount: usize,
	pub(self) mntinfo: &'ctx MountInfo,
	pub(self) errors: &'ctx Mutex<Vec<PrepError>>,
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
		debug_assert!(!ctx.errors.is_empty());

		let orig_modules_len = self.vzscript.modules().len();
		let to_reserve = ctx.tracker.prep_target();

		debug_assert!(to_reserve > 0);

		if let Err(err) = self.assets.try_reserve(to_reserve) {
			panic!("Failed to reserve memory for approx. {to_reserve} new assets. Error: {err:?}",);
		}

		let mut staging = Vec::with_capacity(ctx.new_mounts.end - ctx.new_mounts.start);
		staging.resize_with(ctx.new_mounts.end - ctx.new_mounts.start, || {
			Mutex::new(SlotMap::default())
		});

		// Pass 1: compile VZS; transpile EDF and (G)ZDoom DSLs.

		for i in ctx.new_mounts.clone() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				tracker: &ctx.tracker,
				i_mount: i,
				mntinfo: &self.mounts[i].info,
				assets: &staging[i - ctx.new_mounts.start],
				errors: &ctx.errors[i - ctx.new_mounts.start],
			};

			let module = match subctx.mntinfo.kind {
				MountKind::VileTech => self.prep_pass1_vpk(&subctx),
				MountKind::ZDoom => self.prep_pass1_pk(&subctx),
				MountKind::Eternity => todo!(),
				MountKind::Wad => self.prep_pass1_wad(&subctx),
				MountKind::Misc => self.prep_pass1_file(&subctx),
			};

			if let Outcome::Ok(m) = module {
				self.vzscript.add_module(m);
			} // Otherwise, errors and warnings have already been added to `ctx`.
		}

		// Pass 2: dependency-free assets; trivial to parallelize. Includes:
		// - Palettes and colormaps.
		// - Sounds and music.
		// - Non-picture-format images.

		for i in ctx.new_mounts.clone() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				tracker: &ctx.tracker,
				i_mount: i,
				mntinfo: &self.mounts[i].info,
				assets: &staging[i - ctx.new_mounts.start],
				errors: &ctx.errors[i - ctx.new_mounts.start],
			};

			match subctx.mntinfo.kind {
				MountKind::Wad => self.prep_pass2_wad(&subctx),
				MountKind::VileTech => {} // Soon!
				_ => unimplemented!("Soon!"),
			}
		}

		// TODO: Forbid further loading without a PLAYPAL present?

		// Pass 3: assets dependent on pass 2. Includes:
		// - Picture-format images, which need palettes.
		// - Maps, which need textures, music, scripts, blueprints...

		for i in ctx.new_mounts.clone() {
			if ctx.tracker.is_cancelled() {
				return Outcome::Cancelled;
			}

			let subctx = SubContext {
				tracker: &ctx.tracker,
				i_mount: i,
				mntinfo: &self.mounts[i].info,
				assets: &staging[i - ctx.new_mounts.start],
				errors: &ctx.errors[i - ctx.new_mounts.start],
			};

			match subctx.mntinfo.kind {
				MountKind::Wad => self.prep_pass3_wad(&subctx),
				MountKind::VileTech => {} // Soon!
				_ => unimplemented!("Soon!"),
			}
		}

		let mut errors: Vec<Vec<PrepError>> = std::mem::take(&mut ctx.errors)
			.into_iter()
			.map(|mutex| mutex.into_inner())
			.collect();

		for subvec in &mut errors {
			subvec.sort_by(|err1, err2| err1.path.cmp(&err2.path));
		}

		let ret = Output { errors };

		if ret.any_errs() {
			self.on_prep_fail(&ctx, orig_modules_len);
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
	fn prep_pass1_vpk(&self, ctx: &SubContext) -> Outcome<vzs::Module, ()> {
		let ret = None;

		let script_root: VPathBuf = if let Some(srp) = &ctx.mntinfo.script_root {
			[ctx.mntinfo.virtual_path(), srp].iter().collect()
		} else {
			todo!()
		};

		let script_root = match self.get_file(&script_root) {
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

		let inctree = vzs::parse_include_tree(ctx.mntinfo.virtual_path(), script_root);

		if inctree.any_errors() {
			unimplemented!("Soon");
		}

		if ctx.tracker.is_cancelled() {
			return Outcome::Cancelled;
		}

		match ret {
			Some(module) => Outcome::Ok(module),
			None => Outcome::None,
		}
	}

	fn prep_pass1_file(&self, ctx: &SubContext) -> Outcome<vzs::Module, ()> {
		let ret = None;

		let file = self.get_file(ctx.mntinfo.virtual_path()).unwrap();

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
		} else if file.file_stem().eq_ignore_ascii_case("decorate") {
			unimplemented!();
		} else if file.file_stem().eq_ignore_ascii_case("zscript") {
			unimplemented!();
		} else if file.file_stem().eq_ignore_ascii_case("edfroot") {
			unimplemented!();
		}

		match ret {
			Some(module) => Outcome::Ok(module),
			None => Outcome::None,
		}
	}

	fn prep_pass1_pk(&self, _ctx: &SubContext) -> Outcome<vzs::Module, ()> {
		// TODO: Soon!
		Outcome::None
	}

	fn prep_pass1_wad(&self, _ctx: &SubContext) -> Outcome<vzs::Module, ()> {
		// TODO: Soon!
		Outcome::None
	}

	fn prep_pass2_wad(&self, ctx: &SubContext) {
		let wad = self.get_file(ctx.mntinfo.virtual_path()).unwrap();

		wad.children().par_bridge().for_each(|child| {
			if !child.is_readable() {
				return;
			}

			if ctx.tracker.is_cancelled() {
				return;
			}

			let bytes = child.read_bytes();
			let fstem = child.file_stem();

			let res = if fstem.starts_with("PLAYPAL") {
				self.prep_playpal(ctx, bytes)
			} else {
				return;
			};

			match res {
				Ok(()) => {}
				Err(err) => {
					ctx.errors.lock().push(PrepError {
						path: child.path.to_path_buf(),
						kind: PrepErrorKind::Io(err),
					});
				}
			}
		});
	}

	fn prep_pass3_wad(&self, ctx: &SubContext) {
		let wad = self.get_file(ctx.mntinfo.virtual_path()).unwrap();

		wad.child_refs()
			.filter(|c| !c.is_empty())
			.par_bridge()
			.for_each(|child| {
				if ctx.tracker.is_cancelled() {
					return;
				}

				if child.is_dir() {
					self.prep_pass3_wad_dir(ctx, child)
				} else {
					self.prep_pass3_wad_entry(ctx, child)
				};
			});
	}

	fn prep_pass3_wad_entry(&self, ctx: &SubContext, vfile: FileRef) {
		let bytes = vfile.read_bytes();
		let fstem = vfile.file_stem();

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
					path: vfile.path.to_path_buf(),
					kind: PrepErrorKind::Io(err),
				});
			}
		}
	}

	fn prep_pass3_wad_dir(&self, ctx: &SubContext, dir: FileRef) {
		match self.try_prep_level_vanilla(ctx, dir) {
			Some(Ok(_key)) => {}
			Some(Err(_err)) => {}
			None => {}
		}

		match self.try_prep_level_udmf(ctx, dir) {
			None => {}
			Some(Err(_err)) => {}
			Some(Ok(_key)) => {}
		}
	}

	fn on_prep_fail(&mut self, _ctx: &Context, orig_modules_len: usize) {
		self.vzscript.truncate(orig_modules_len);
		self.clean_maps();
	}
}

// Processors for individual data formats.
impl Catalog {
	/// Returns `None` to indicate that `bytes` was checked
	/// and determined to not be a picture.
	#[must_use]
	fn prep_picture(&self, ctx: &SubContext, bytes: &[u8], id: &str) -> Option<()> {
		// TODO: Wasteful to run a hash lookup before checking if this is a picture.
		let palettes = self.last_asset_by_nick::<PaletteSet>("PLAYPAL").unwrap();

		if let Some(image) = Image::try_from_picture(bytes, &palettes.palettes[0]) {
			self.register_asset::<Image>(
				ctx,
				Image {
					header: AssetHeader {
						id: format!("{mount_id}/{id}", mount_id = ctx.mntinfo.id()),
					},
					inner: image.0,
					offset: image.1,
				},
			);

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

		self.register_asset::<PaletteSet>(
			ctx,
			PaletteSet {
				header: AssetHeader {
					id: format!("{}/PLAYPAL", ctx.mntinfo.id()),
				},
				palettes: palettes.into_inner().unwrap(),
			},
		);

		Ok(())
	}
}

// Common functions.
impl Catalog {
	fn register_asset<A: Asset>(&self, ctx: &SubContext, asset: A) {
		let nickname = asset.header().nickname();
		let key_full = AssetKey::new::<A>(&asset.header().id);
		let key_nick = AssetKey::new::<A>(nickname);
		let lookup = self.assets.entry(key_full);

		if matches!(lookup, dashmap::mapref::entry::Entry::Occupied(_)) {
			info!(
				"Overwriting asset: {} type ({})",
				asset.header().id,
				asset.type_name()
			);
		}

		let slotkey = ctx.assets.lock().insert(Arc::new(asset));

		if let Some(mut kvp) = self.nicknames.get_mut(&key_nick) {
			kvp.value_mut().push((ctx.i_mount, slotkey));
		} else {
			self.nicknames
				.insert(key_nick, smallvec![(ctx.i_mount, slotkey)]);
		};

		match lookup {
			dashmap::mapref::entry::Entry::Occupied(mut occu) => {
				occu.insert((ctx.i_mount, slotkey));
			}
			dashmap::mapref::entry::Entry::Vacant(vacant) => {
				vacant.insert((ctx.i_mount, slotkey));
			}
		}
	}
}
