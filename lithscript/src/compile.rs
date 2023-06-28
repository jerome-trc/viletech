//! Types relevant to both the frontend and backend.

use std::{mem::MaybeUninit, path::PathBuf, sync::Arc};

use cranelift::prelude::settings::OptLevel;
use cranelift_jit::{JITBuilder, JITModule};
use doomfront::{rowan::GreenNode, ParseTree};
use parking_lot::Mutex;

use crate::{issue::Issue, lir, Syn, Version};

/// It is at this point when all native functions and data that JIT scripts can
/// call must be registered.
pub struct LibBuilder {
	jit: JITBuilder,
	pub name: String,
	pub version: Version,
	pub sources: Vec<ContainerSource>,
}

impl LibBuilder {
	#[must_use]
	pub fn new(
		name: String,
		version: Version,
		sources: Vec<ContainerSource>,
		opt: OptLevel,
	) -> Self {
		Self {
			jit: {
				let o_lvl = match opt {
					OptLevel::None => "none",
					OptLevel::Speed => "speed",
					OptLevel::SpeedAndSize => "speed_and_size",
				};

				JITBuilder::with_flags(
					&[
						("use_colocated_libcalls", "false"),
						("is_pic", "false"),
						("opt_level", o_lvl),
					],
					cranelift_module::default_libcall_names(),
				)
				.expect("JIT module builder creation failed")
			},
			name,
			version,
			sources,
		}
	}

	/// See [`JITBuilder::symbol`].
	pub fn symbol(&mut self, name: impl Into<String>, ptr: *const u8) {
		self.jit.symbol(name, ptr);
	}

	/// See [`JITBuilder::symbols`].
	pub fn symbols<I, N>(&mut self, symbols: I)
	where
		I: IntoIterator<Item = (N, *const u8)>,
		N: Into<String>,
	{
		self.jit.symbols(symbols);
	}

	#[must_use]
	pub(crate) fn finish(self) -> LibSource {
		LibSource {
			name: self.name,
			version: self.version,
			jit: Arc::new(JitModule(Mutex::new(MaybeUninit::new(JITModule::new(
				self.jit,
			))))),
			sources: self.sources,
		}
	}
}

#[derive(Debug)]
pub struct ContainerSource {
	pub(crate) path: PathBuf,
	pub(crate) root: GreenNode,
}

impl ContainerSource {
	#[must_use]
	pub fn new(path: PathBuf, ptree: ParseTree<Syn>) -> Self {
		assert!(
			!ptree.any_errors(),
			"encountered one or more parsing errors"
		);

		Self {
			path,
			root: ptree.into_inner(),
		}
	}
}

#[derive(Debug)]
pub(crate) struct LibSource {
	pub(crate) name: String,
	pub(crate) version: Version,
	pub(crate) jit: Arc<JitModule>,
	pub(crate) sources: Vec<ContainerSource>,
}

// Precompile //////////////////////////////////////////////////////////////////

/// The output of lowering source to LIR, to be passed to [`crate::codegen`].
#[derive(Debug)]
pub(crate) struct _Precompile {
	_module: Arc<JitModule>,
	_ir: Vec<lir::Container>,
	_issues: Vec<Issue>,
}

// JitModule ///////////////////////////////////////////////////////////////////

/// Ensures proper JIT code de-allocation (from behind an [`Arc`]).
#[derive(Debug)]
pub(crate) struct JitModule(pub(crate) Mutex<MaybeUninit<JITModule>>);

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			std::mem::replace(&mut self.0, Mutex::new(MaybeUninit::uninit()))
				.into_inner()
				.assume_init()
				.free_memory();
		}
	}
}

// TODO: Prove that this is safe when codegen machinery is in working order.
unsafe impl Send for JitModule {}
unsafe impl Sync for JitModule {}
