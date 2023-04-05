//! Internal postprocessing functions.
//!
//! After mounting is done, start composing useful assets from raw files.

use std::{
	io::Cursor,
	ops::Range,
	sync::{atomic, Arc},
};

use arrayvec::ArrayVec;
use bevy::prelude::info;
use byteorder::ReadBytesExt;
use dashmap::DashSet;
use image::Rgba;
use parking_lot::Mutex;
use rayon::prelude::*;
use smallvec::smallvec;

use crate::{lith, VPathBuf};

use super::{
	detail::AssetKey, Asset, Audio, Catalog, FileRef, Image, LoadTracker, MountInfo, MountKind,
	Palette, PaletteSet, PostProcError, PostProcErrorKind, Record,
};

#[derive(Debug)]
pub(super) struct Context {
	pub(super) tracker: Arc<LoadTracker>,
	// To enable atomicity, remember where `self.files` and `self.mounts` were.
	// Truncate back to them in the event of failure.
	pub(super) orig_files_len: usize,
	pub(super) orig_mounts_len: usize,
	/// To enable atomicity, remember which assets were added.
	/// Remove them all in the event of failure.
	pub(super) added: DashSet<AssetKey>,
	pub(super) new_mounts: Range<usize>,
	/// Returning errors through the post-proc call tree is somewhat
	/// inflexible, so pass an array down through the context instead.
	pub(super) errors: Mutex<Vec<Vec<PostProcError>>>,
}

#[derive(Debug)]
#[must_use]
pub(super) struct Output {
	/// Every *new* mount gets a sub-vec, but that sub-vec may be empty.
	pub(super) errors: Vec<Vec<PostProcError>>,
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
	/// - Load tracker has already had its post-proc target number set.
	pub(super) fn postproc(&mut self, mut ctx: Context) -> Output {
		let orig_modules_len = self.modules.len();
		let to_reserve = ctx.tracker.pproc_target.load(atomic::Ordering::SeqCst) as usize;

		debug_assert!(to_reserve > 0);

		if let Err(err) = self.assets.try_reserve(to_reserve) {
			panic!("Failed to reserve memory for approx. {to_reserve} new assets. Error: {err:?}",);
		}

		// Pass 1: compile Lith; transpile EDF and (G)ZDoom DSLs.

		for i in ctx.new_mounts.clone() {
			let module = match &self.mounts[i].kind {
				MountKind::VileTech => self.pproc_pass1_vpk(i, &ctx),
				MountKind::ZDoom => self.pproc_pass1_pk(i, &ctx),
				MountKind::Eternity => todo!(),
				MountKind::Wad => self.pproc_pass1_wad(i, &ctx),
				MountKind::Misc => self.pproc_pass1_file(i, &ctx),
			};

			if let Ok(Some(m)) = module {
				self.modules.push(m);
			} // Otherwise, errors and warnings have already been added to `ctx`.
		}

		// Pass 2: dependency-free assets; trivial to parallelize. Includes:
		// - Palettes and colormaps.
		// - Sounds and music.
		// - Non-picture-format images.

		for i in 0..self.mounts.len() {
			match &self.mounts[i].kind {
				MountKind::Wad => self.pproc_pass2_wad(i, &ctx),
				MountKind::VileTech => {} // Soon!
				_ => unimplemented!("Soon!"),
			}
		}

		// TODO: Forbid further loading without a PLAYPAL present.

		// Pass 3: assets dependent on pass 2. Includes:
		// - Picture-format images, which need palettes.
		// - Maps, which need textures, music, scripts, blueprints...

		for i in 0..self.mounts.len() {
			match &self.mounts[i].kind {
				MountKind::Wad => self.pproc_pass3_wad(i, &ctx),
				MountKind::VileTech => {} // Soon!
				_ => unimplemented!("Soon!"),
			}
		}

		let errors = std::mem::replace(&mut ctx.errors, Mutex::new(vec![])).into_inner();
		let ret = Output { errors };

		if ret.any_errs() {
			self.on_pproc_fail(&ctx, orig_modules_len);
		} else {
			// TODO: Make each successfully processed file increment progress.
			ctx.tracker.pproc_progress.store(
				ctx.tracker.pproc_target.load(atomic::Ordering::SeqCst),
				atomic::Ordering::SeqCst,
			);

			info!("Loading complete.");
		}

		ret
	}

	/// Try to compile non-ACS scripts from this package. Lith, EDF, and (G)ZDoom
	/// DSLs all go into the same Lith module, regardless of which are present
	/// and which are absent.
	fn pproc_pass1_vpk(&self, mount: usize, ctx: &Context) -> Result<Option<lith::Module>, ()> {
		let ret = None;
		let mntinfo = &self.mounts[mount];

		let script_root: VPathBuf = if let Some(srp) = &mntinfo.script_root {
			[mntinfo.virtual_path(), srp].iter().collect()
		} else {
			todo!()
		};

		let script_root = match self.get_file(&script_root) {
			Some(fref) => fref,
			None => {
				ctx.errors.lock()[mount].push(PostProcError {
					path: script_root.to_path_buf(),
					kind: PostProcErrorKind::MissingScriptRoot,
				});

				return Err(());
			}
		};

		let inctree = lith::parse_include_tree(mntinfo.virtual_path(), script_root);

		if inctree.any_errors() {
			unimplemented!("Soon");
		}

		Ok(ret)
	}

	fn pproc_pass1_file(&self, mount: usize, _ctx: &Context) -> Result<Option<lith::Module>, ()> {
		let ret = None;

		let file = self.get_file(self.mounts[mount].virtual_path()).unwrap();

		// Pass 1 only deals in text files.
		if !file.is_text() {
			return Ok(None);
		}

		if file
			.path_extension()
			.filter(|p_ext| p_ext.eq_ignore_ascii_case("lith"))
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

		Ok(ret)
	}

	fn pproc_pass1_pk(&self, _mount: usize, _ctx: &Context) -> Result<Option<lith::Module>, ()> {
		let ret = None;

		Ok(ret)
	}

	fn pproc_pass1_wad(&self, _mount: usize, _ctx: &Context) -> Result<Option<lith::Module>, ()> {
		let ret = None;

		Ok(ret)
	}

	fn pproc_pass2_wad(&self, mount: usize, ctx: &Context) {
		let mntinfo = &self.mounts[mount];
		let wad = self.get_file(mntinfo.virtual_path()).unwrap();

		wad.children().par_bridge().for_each(|child| {
			if !child.is_readable() {
				return;
			}

			let bytes = child.read_bytes();
			let fstem = child.file_stem();

			let res = if fstem.starts_with("PLAYPAL") {
				self.pproc_playpal(bytes, mntinfo.id())
			} else {
				return;
			};

			match res {
				Ok(key) => {
					ctx.added.insert(key);
				}
				Err(err) => {
					unimplemented!("Unhandled error: {err}");
				}
			}
		});
	}

	fn pproc_pass3_wad(&self, mount: usize, ctx: &Context) {
		let mntinfo = &self.mounts[mount];
		let wad = self.get_file(mntinfo.virtual_path()).unwrap();

		wad.child_refs()
			.filter(|c| !c.is_empty())
			.par_bridge()
			.for_each(|child| {
				if child.is_dir() {
					self.pproc_pass3_wad_dir(child, ctx, mntinfo)
				} else {
					self.pproc_pass3_wad_entry(child, ctx, mntinfo)
				};
			});
	}

	fn pproc_pass3_wad_entry(&self, vfile: FileRef, ctx: &Context, mntinfo: &MountInfo) {
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

		let key = self.pproc_picture(bytes, fstem, mntinfo.id());

		let res: std::io::Result<AssetKey> = if let Some(key) = key {
			Ok(key)
		} else {
			return;
		};

		match res {
			Ok(key) => {
				ctx.added.insert(key);
			}
			Err(err) => {
				unimplemented!("Unhandled error: {err}");
			}
		}
	}

	fn pproc_pass3_wad_dir(&self, dir: FileRef, ctx: &Context, mntinfo: &MountInfo) {
		match self.try_pproc_map_vanilla(dir, ctx, mntinfo) {
			Some(Ok(_key)) => {}
			Some(Err(_err)) => {}
			None => {}
		}

		match self.try_pproc_map_udmf(dir, ctx, mntinfo) {
			None => {}
			Some(Err(_err)) => {}
			Some(Ok(_key)) => {}
		}
	}

	#[must_use]
	fn try_pproc_map_vanilla(
		&self,
		_dir: FileRef,
		_ctx: &Context,
		_mntinfo: &MountInfo,
	) -> Option<Result<AssetKey, PostProcError>> {
		todo!()
	}

	#[must_use]
	fn try_pproc_map_udmf(
		&self,
		_dir: FileRef,
		_ctx: &Context,
		_mntinfo: &MountInfo,
	) -> Option<Result<AssetKey, PostProcError>> {
		todo!()
	}

	fn on_pproc_fail(&mut self, ctx: &Context, orig_modules_len: usize) {
		ctx.added.par_iter().for_each(|key| {
			let removed = self.assets.remove(key.key());
			debug_assert!(removed.is_some());
		});

		self.modules.truncate(orig_modules_len);

		self.on_mount_fail(ctx.orig_files_len, ctx.orig_mounts_len);
	}
}

// Post-processors for individual data formats.
impl Catalog {
	#[must_use]
	fn pproc_picture(&self, bytes: &[u8], id: &str, mount_id: &str) -> Option<AssetKey> {
		let palettes = self.last_asset_by_nick::<PaletteSet>("PLAYPAL").unwrap();

		if let Some(image) = Image::try_from_picture(bytes, &palettes.0[0]) {
			let id = format!("{mount_id}/{id}");
			drop(palettes); // Drop `DashMap` ref, or else get deadlocks.
			Some(self.register_asset::<Image>(id, image))
		} else {
			None
		}
	}

	fn pproc_playpal(&self, bytes: &[u8], mount_id: &str) -> std::io::Result<AssetKey> {
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

		let id = format!("{mount_id}/PLAYPAL");
		let ret = self.register_asset::<PaletteSet>(id, PaletteSet(palettes.into_inner().unwrap()));

		Ok(ret)
	}
}

// Common functions.
impl Catalog {
	#[must_use]
	fn register_asset<A: Asset>(&self, id: String, asset: A) -> AssetKey {
		let key = AssetKey::new::<A>(&id);
		let nick = id.split('/').last().unwrap();

		let record = if let Some(mut kvp) = self.nicknames.get_mut(nick) {
			let record = Arc::new(Record::new(id, asset));
			let weak = Arc::downgrade(&record);
			kvp.value_mut().push(weak);
			record
		} else {
			let nick = nick.to_string().into_boxed_str();
			let record = Arc::new(Record::new(id, asset));
			let weak = Arc::downgrade(&record);
			self.nicknames.insert(nick, smallvec![weak]);
			record
		};

		let clobbered = self.assets.insert(key, record);

		if let Some(record) = clobbered {
			info!("Overwriting asset: {}", record.id());
		}

		key
	}
}
