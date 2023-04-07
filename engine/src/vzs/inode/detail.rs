use std::{fmt::Debug, marker::PhantomData, mem::MaybeUninit, sync::Arc};

use crate::vzs::{abi::QWord, Handle, Symbol};

/// Points to an i-node. Used by other i-nodes and the
/// [`Runtime`](crate::vzs::Runtime)'s "instruction pointer".
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::vzs) struct Index(pub(super) usize);

/// Generalizes over different instruction fields, allowing the same enum to be
/// re-used for instructions that own their data and instructions that refer to it.
pub(in crate::vzs) trait NodeKind {
	type Index: Debug;
	type ArcT<AT: 'static + Debug>: Debug;
	type HandleT<HT: 'static + Symbol>: Debug;
}

#[derive(Debug)]
pub(in crate::vzs) struct OwningNode;
#[derive(Debug)]
pub(in crate::vzs) struct RefNode<'i>(PhantomData<&'i ()>);

impl NodeKind for OwningNode {
	type Index = Index;
	type ArcT<AT: 'static + Debug> = Arc<AT>;
	type HandleT<HT: 'static + Symbol> = Handle<HT>;
}

impl<'i> NodeKind for RefNode<'i> {
	type Index = QWord;
	type ArcT<AT: 'static + Debug> = &'i Arc<AT>;
	type HandleT<HT: 'static + Symbol> = &'i Handle<HT>;
}

/// Ensures proper JIT code de-allocation.
#[derive(Debug)]
#[repr(transparent)]
pub(in crate::vzs) struct JitModule(MaybeUninit<cranelift_jit::JITModule>);

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			let i = std::mem::replace(&mut self.0, MaybeUninit::uninit());
			i.assume_init().free_memory();
		}
	}
}
