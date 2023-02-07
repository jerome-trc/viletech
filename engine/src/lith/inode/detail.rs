use std::{fmt::Debug, marker::PhantomData, mem::MaybeUninit, sync::Arc};

use crate::lith::abi::QWord;

/// Points to an i-node. Used by other i-nodes and the
/// [`Runtime`](crate::lith::Runtime)'s "instruction pointer".
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::lith) struct Index(pub(super) usize);

/// Generalizes over different instruction fields, allowing the same enum to be
/// re-used for instructions that own their data and instructions that refer to it.
pub(in crate::lith) trait NodeKind {
	type Index: Debug;
	type ArcT<AT: 'static + Debug>: Debug;
}

#[derive(Debug)]
pub(in crate::lith) struct OwningNode;
#[derive(Debug)]
pub(in crate::lith) struct RefNode<'i>(PhantomData<&'i ()>);

impl NodeKind for OwningNode {
	type Index = Index;
	type ArcT<AT: 'static + Debug> = Arc<AT>;
}

impl<'i> NodeKind for RefNode<'i> {
	type Index = QWord;
	type ArcT<AT: 'static + Debug> = &'i Arc<AT>;
}

/// Ensures proper JIT code de-allocation.
#[derive(Debug)]
#[repr(transparent)]
pub(in crate::lith) struct JitModule(MaybeUninit<cranelift_jit::JITModule>);

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			let i = std::mem::replace(&mut self.0, MaybeUninit::uninit());
			i.assume_init().free_memory();
		}
	}
}
