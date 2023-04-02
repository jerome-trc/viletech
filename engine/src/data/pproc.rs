//! Internal postprocessing functions.
//!
//! After mounting is done, start composing useful assets from raw files.

#![allow(dead_code)]
#![allow(unused)]

use std::sync::Arc;

use parking_lot::RwLock;

use crate::{lith, VPath, VPathBuf};

use super::{Catalog, FileRef, LoadTracker, MountInfo, MountKind, PostProcError};

#[derive(Debug)]
pub(super) struct Context {
	pub(super) tracker: Arc<LoadTracker>,
	// To enable atomicity, remember where `self.files` and `self.mounts` were.
	// Truncate back to them upon a failure.
	pub(super) orig_files_len: usize,
	pub(super) orig_mounts_len: usize,
}

#[derive(Debug)]
#[must_use]
pub(super) struct Output {
	/// One per mount.
	pub(super) results: Vec<Result<(), Vec<PostProcError>>>,
}

impl Catalog {
	/// Preconditions:
	/// - `self.files` has been populated. All directories know their contents.
	/// - `self.mounts` has been populated.
	pub(super) fn postproc(&mut self, ctx: Context) -> Output {
		let mut results = vec![];

		// Pass 1: compile Lith; transpile EDF and (G)ZDoom DSLs.

		for i in 0..self.mounts.len() {
			let module = match &self.mounts[i].kind {
				MountKind::VileTech => self.pproc_pass1_vpk(i, &ctx),
				MountKind::ZDoom => self.pproc_pass1_pk(i, &ctx),
				MountKind::Eternity => todo!(),
				MountKind::Wad => self.pproc_pass1_wad(i, &ctx),
				MountKind::Misc => self.pproc_pass1_file(i, &ctx),
			};

			match module {
				Ok(Some(m)) => {
					self.modules.push(m);
					results.push(Ok(()));
				}
				Ok(None) => {
					results.push(Ok(()));
				}
				Err(errs) => {
					results.push(Err(errs));
					continue;
				}
			}
		}

		// Pass 2: load images, sounds, music, maps.
		// ...soon!

		Output { results }
	}

	/// Try to compile non-ACS scripts from this package. Lith, EDF, and (G)ZDoom
	/// DSLs all go into the same Lith module, regardless of which are present
	/// and which are absent.
	fn pproc_pass1_vpk(
		&self,
		mount: usize,
		ctx: &Context,
	) -> Result<Option<lith::Module>, Vec<PostProcError>> {
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
				return Err(vec![PostProcError::MissingScriptRoot(
					script_root.to_path_buf(),
				)]);
			}
		};

		let inctree = lith::parse_include_tree(mntinfo.virtual_path(), script_root);

		if inctree.any_errors() {
			unimplemented!("Soon");
		}

		Ok(ret)
	}

	fn pproc_pass1_file(
		&self,
		mount: usize,
		ctx: &Context,
	) -> Result<Option<lith::Module>, Vec<PostProcError>> {
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

	fn pproc_pass1_pk(
		&self,
		mount: usize,
		ctx: &Context,
	) -> Result<Option<lith::Module>, Vec<PostProcError>> {
		let ret = None;

		Ok(ret)
	}

	fn pproc_pass1_wad(
		&self,
		mount: usize,
		ctx: &Context,
	) -> Result<Option<lith::Module>, Vec<PostProcError>> {
		let ret = None;

		Ok(ret)
	}
}
