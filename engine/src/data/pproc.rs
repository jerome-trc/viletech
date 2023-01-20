//! Internal postprocessing functions.
//!
//! After mounting is done, start composing useful assets from raw files.

#![allow(dead_code)]
#![allow(unused)]

use std::sync::Arc;

use parking_lot::RwLock;

use crate::lith;

use super::{Catalog, LoadTracker, PostProcError};

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
	pub(super) results: Vec<Result<(), PostProcError>>,
}

impl Catalog {
	pub(super) fn postproc(&mut self, ctx: Context) -> Output {
		unimplemented!("Soon!")
	}
}
