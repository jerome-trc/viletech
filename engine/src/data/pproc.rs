//! Internal postprocessing functions.
//!
//! After mounting is done, start composing useful assets from raw files.

#![allow(dead_code)]
#![allow(unused)]

use std::sync::Arc;

use parking_lot::RwLock;

use crate::{lith, VPath, VPathBuf};

use super::{Catalog, FileRef, LoadTracker, Mount, MountInfo, PostProcError};

#[derive(Debug)]
pub(super) struct Context {
	pub(super) project: Arc<RwLock<lith::Project>>,
	pub(super) tracker: Arc<LoadTracker>,
	// To enable atomicity, remember where `self.files` and `self.mounts` were.
	// Truncate back to them upon a failure
	pub(super) orig_files_len: usize,
	pub(super) orig_mounts_len: usize,
}

#[derive(Debug)]
pub(super) struct Output {
	/// One per mount.
	pub(super) results: Vec<Result<(), PostProcError>>,
}

impl Catalog {
	/// Preconditions:
	/// - `self.files` has been populated. All directories know their contents.
	/// - `self.mounts` has been populated.
	#[must_use]
	pub(super) fn postproc(&mut self, ctx: Context) -> Output {
		let mut outcomes = Vec::with_capacity(self.mounts.len());
		outcomes.resize_with(self.mounts.len(), || Outcome::Uninit);

		for i in 0..self.mounts.len() {
			let outcome = match &self.mounts[i].info.kind {
				super::MountKind::VileTech => self.postproc_vile_pkg(i, &ctx),
				super::MountKind::ZDoom => todo!(),
				super::MountKind::Eternity => todo!(),
				super::MountKind::Wad => todo!(),
				super::MountKind::Misc => todo!(),
			};
		}

		unimplemented!("Soon!")
	}

	fn postproc_vile_pkg(&mut self, mount: usize, ctx: &Context) -> Outcome {
		let mntinfo = &self.mounts[mount].info;

		let script_root = if let Some(srp) = &mntinfo.script_root {
			srp
		} else {
			return self.postproc_zdoom_pkg(mount, ctx);
		};

		let script_root = match self.get_file(script_root) {
			Some(fref) => fref,
			None => {
				return Outcome::Err(PostProcError::MissingScriptRoot(script_root.to_path_buf()))
			}
		};

		let inctree = lith::parse_include_tree(mntinfo.virtual_path(), script_root);

		if inctree.any_errors() {
			unimplemented!("Soon");
		}

		if inctree.tree.is_none() {
			return self.postproc_zdoom_pkg(mount, ctx);
		}

		Outcome::Ok {}
	}

	fn postproc_zdoom_pkg(&mut self, mount: usize, ctx: &Context) -> Outcome {
		Outcome::Ok {}
	}
}

#[derive(Debug)]
enum Outcome {
	Uninit,
	Err(PostProcError),
	Ok {},
}
