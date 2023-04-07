//! The VZScript execution context: stack, heap, garbage collector, coroutines...

use std::mem::MaybeUninit;

use super::{
	abi::{Abi, QWord},
	heap::Heap,
	inode,
};

/// Context for VZScript execution.
///
/// Fully re-entrant; VZS has no global state.
#[derive(Debug, Default)]
pub struct Runtime {
	pub(super) iptr: InstPtr,
	pub(super) stack: Stack,
	pub(super) heap: Heap,
	pub(super) icache: ICache,
	// See vzs::heap for memory management, GC methods.
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) enum InstPtr {
	#[default]
	None,
	Running(inode::Index),
	Return,
	Panic,
}

#[derive(Debug)]
pub(super) struct Stack {
	pub(super) buf: Box<[QWord]>,
	pub(super) top: *mut QWord,
}

impl Stack {
	pub(super) unsafe fn push<T: Abi>(&mut self, value: T) {
		let sz = std::mem::size_of::<T::Repr>();

		assert!(self.capacity() >= sz, "VZS stack overflow.");

		unsafe {
			std::ptr::write(self.top as *mut T::Repr, value.to_words());
			self.top = self.top.add(sz);
		}
	}

	pub(super) unsafe fn pop<T: Abi>(&mut self) -> T {
		let sz = std::mem::size_of::<T::Repr>();

		assert!(
			self.top as usize >= (self.buf.as_ptr() as usize + sz),
			"Tried to pop an empty VZS stack."
		);

		unsafe {
			self.top = self.top.sub(sz);
			let ret = std::ptr::read(self.top.cast::<T::Repr>());
			T::from_words(ret)
		}
	}

	/// In terms of bytes, not quad-words.
	#[must_use]
	fn capacity(&self) -> usize {
		(self.buf.as_ptr() as usize + self.buf.len()) - self.top as usize
	}
}

impl Default for Stack {
	fn default() -> Self {
		let mut buf = vec![];
		buf.resize(8 * 1024, QWord::default());
		let ptr = buf.as_mut_ptr();

		Self {
			buf: buf.into_boxed_slice(),
			top: ptr,
		}
	}
}

// SAFETY: Internal pointer is only mutated through exclusive references.
unsafe impl Send for Stack {}
unsafe impl Sync for Stack {}

/// Gets filled with the results of instruction node evaluation.
#[derive(Debug)]
pub(super) struct ICache(pub(super) Vec<MaybeUninit<QWord>>);

impl Default for ICache {
	fn default() -> Self {
		Self(Vec::with_capacity(64 * 1024))
	}
}
