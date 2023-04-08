//! VZS's memory model; a per-runtime managed heap and garbage collector.
//!
//! This is based off Mike Pall's design for LuaJIT 3.0's GC, outlined here:
//! <https://web.archive.org/web/20220826233802/http://wiki.luajit.org/New-Garbage-Collector>
//!
//! Crucial distinctions from Pall's specification:
//! - Supports arbitrary release or destruction of a managed object to meet
//! the particular requirements of backwards compatibility with ZScript.
//! - What Pall (and much of existing GC literature) calls "cells" are known here
//! as "shards" since Rust already has [`std::cell::Cell`]; "blocks" are instead
//! called "regions".
//!
//! Other notes:
//! - Bumpalo is not used here since it cannot allocate 1 MB-aligned chunks.
//!
//! Things that currently need doing:
//! - Generational collection.
//! - Segregated first and best fit allocation strategies for when bump fails.
//! - Mark and sweep logic; sequential store buffer, gray stack, gray queue.
//! - Support for 32-bit architectures.
//! - A way to set an arbitrary heap size limit before a forced engine panic.
//! - Soak testing for leaks over long periods of time.
//! - All kinds of tuning on all kinds of platforms.

#![allow(dead_code)] // TODO: Remove

#[cfg(target_pointer_width = "32")]
std::compile_error!("VZS's heap does not yet support 32-bit architectures.");

use std::{alloc::Layout, collections::HashMap, marker::PhantomData, ptr::NonNull};

use bitvec::prelude::BitArray;

use crate::vzs::Symbol;

use super::{tsys, Handle, Runtime, TypeInfo};

// Public //////////////////////////////////////////////////////////////////////

/// Benefits from null-pointer optimization.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Ptr(NonNull<RegionHeader>);

impl Ptr {
	#[must_use]
	pub(super) unsafe fn typeinfo(&self) -> &Handle<TypeInfo> {
		&(*self.0.as_ptr()).tinfo
	}

	#[must_use]
	fn chunk(&self) -> NonNull<ChunkHeader> {
		let addr = (self.0.as_ptr() as usize) & 0x00000fffffffffff;
		NonNull::new(addr as *mut ChunkHeader).unwrap()
	}

	/// Comes with a read barrier; returns `NULL` if attempting to read an
	/// object which has been marked for forced destruction.
	#[must_use]
	unsafe fn get<T>(&self) -> *mut T {
		if (*self.0.as_ptr()).flags.contains(RegionFlags::DESTROYED) {
			return std::ptr::null_mut();
		}

		self.0.as_ptr().cast::<T>()
	}

	unsafe fn write_barrier(&self) {
		if (*self.0.as_ptr()).flags.contains(RegionFlags::GRAY) {
			return;
		}

		unimplemented!()
	}
}

/// "Typed pointer". Benefits from null-pointer optimization.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TPtr<T>(NonNull<RegionHeader>, PhantomData<T>);

// SAFETY: This structure gets used as a key in certain specialized collection types.
// Safety is guaranteed when going through these collections' interfaces.
// It can be dereferenced as a raw pointer, but this is always unsafe, so the
// responsibility to do so in a thread-safe way falls to the API consumer.
unsafe impl<T> Send for TPtr<T> {}
unsafe impl<T> Sync for TPtr<T> {}

/// "Index pointer". Double-wide, and necessary for pointing to certain kinds of
/// objects which can not reasonably be allocated next to a header on a
/// per-instance basis, such as map lines.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IPtr {
	base: Ptr,
	index: usize,
}

#[derive(Debug)]
pub(super) struct Heap {
	/// Are we marking, sweeping, or none of the above?
	status: Status,
	/// Total number of bytes across all allocation requests fulfilled thus far.
	/// Increased with each call to `Self::alloc` or `Self::alloc_huge`; gets
	/// reduced by each collection cycle. Does not include arena headers.
	allocated: usize,
	traced: Vec<Arena>,
	/// Heap allocations that don't need traversal during GC (e.g. boxed primitives,
	/// structures with no pointers) are stored in these arenas.
	untraced: Vec<Arena>,
	/// For tracking allocations larger than a VZS type's maximum size.
	/// Each allocation's address is a key.
	huge: HashMap<NonNull<u8>, HugeHeader>,
	/// How high can `Self::allocated` get before a GC step starts?
	garbage_threshold: usize,
}

impl Runtime {
	#[must_use]
	pub(super) unsafe fn alloc_t(&mut self, tinfo: Handle<TypeInfo>) -> Ptr {
		let layout = tinfo.heap_layout();

		debug_assert_ne!(
			layout.size(),
			0,
			"Tried to allocate a zero-sized type on the VZS heap: {}",
			tinfo.header().name,
		);

		debug_assert_eq!(
			layout.align(),
			16,
			"VZS type is not 16-byte aligned: {}",
			tinfo.header().name
		);

		debug_assert!(
			layout.size() < HUGE_SIZE,
			"VZS type is oversized: {}",
			tinfo.header().name,
		);

		let ptr = self.alloc(layout).cast::<RegionHeader>();

		let header = RegionHeader {
			flags: RegionFlags::GRAY,
			tinfo,
		};

		std::ptr::write(ptr.as_ptr(), header);

		Ptr(ptr)
	}

	#[must_use]
	pub(super) unsafe fn alloc(&mut self, layout: Layout) -> NonNull<u8> {
		if layout.size() > HUGE_SIZE {
			let ret = self.alloc_huge(layout);

			if self.on_allocate(layout) {
				self.gc_cycle();
			}

			return ret;
		}

		let arena = self
			.heap
			.traced
			.iter_mut()
			.find(|a| a.capacity() >= layout.size());

		// First, attempt to bump to an arena that has space.

		if let Some(arena) = arena {
			let ret = arena.bump(layout);

			if self.on_allocate(layout) {
				self.gc_cycle();
			}

			return ret;
		}

		unimplemented!()

		// If no existing arena can be bumped, try a best fit.

		// If best-fit fails, try first fit.

		// If first-fit fails, allocate a new arena.
	}

	/// Huge blocks are allocated directly through a `malloc` call.
	#[must_use]
	unsafe fn alloc_huge(&mut self, layout: Layout) -> NonNull<u8> {
		let mut size = 0;

		for i in 1..8 {
			if Arena::CHUNK_LAYOUT.size() * i <= layout.size() {
				size = Arena::CHUNK_LAYOUT.size() * i;
				break;
			}
		}

		assert_ne!(size, 0, "Oversized VZS allocation.");

		let layout = Layout::from_size_align(size, 1 << 20).unwrap();
		let ptr = NonNull::new(std::alloc::alloc(layout)).unwrap();

		self.heap.huge.insert(
			ptr,
			HugeHeader {
				layout,
				block: false,
				mark: false,
			},
		);

		ptr
	}

	fn gc_cycle(&mut self) {
		self.heap.status = Status::Mark;

		// TODO: Type information needed on the stack for root tracing.
		// This depends on future reclamation schemes for concurrent collections.
		// (e.g. assets, module symbols). Might be Hyaline, hazard pointers, epoch...

		self.heap.status = Status::Sweep;

		self.heap.status = Status::Nominal;

		unimplemented!()
	}

	/// Returns `true` if a garbage collection step is deemed to be necessary.
	#[must_use]
	fn on_allocate(&mut self, layout: Layout) -> bool {
		self.heap.allocated += layout.size();
		self.heap.allocated > self.heap.garbage_threshold
	}
}

/// `base` should be the layout for a script type on its own (e.g. all of a class'
/// fields). It gets prepended with space for GC information and a type pointer.
#[must_use]
pub(super) fn layout_for(base: Layout) -> Layout {
	Layout::new::<RegionHeader>()
		.extend(base)
		.unwrap()
		.0
		.pad_to_align()
}

impl Default for Heap {
	fn default() -> Self {
		let (traced, untraced) = unsafe { (Arena::new(), Arena::new()) };

		Self {
			status: Status::default(),
			allocated: 0,
			traced: vec![traced],
			untraced: vec![untraced],
			huge: HashMap::default(),
			garbage_threshold: Arena::CHUNK_LAYOUT.size() * 4,
		}
	}
}

impl Drop for Heap {
	fn drop(&mut self) {
		unsafe {
			for (ptr, header) in &mut self.huge {
				std::alloc::dealloc(ptr.as_ptr().cast::<u8>(), header.layout);
			}
		}
	}
}

// SAFETY: `Runtime` governs this struct entirely.
unsafe impl Send for Heap {}
unsafe impl Sync for Heap {}

const HUGE_SIZE: usize = tsys::MAX_SIZE;

// Internal ////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Status {
	#[default]
	Nominal,
	Mark,
	Sweep,
}

/// Governs a memory allocation with the size and alignment of [`Chunk`].
#[derive(Debug)]
struct Arena {
	/// The start of the allocation. Immutable.
	header: NonNull<ChunkHeader>,
	/// `Self::header` plus the width of a `ChunkHeader`. Immutable.
	data: NonNull<u8>,
	/// Current bump position.
	ptr: NonNull<u8>,
	/// Each element is a shard index. During marking, objects marked dark gray
	/// get pushed. Goes untouched if this arena does not contain traversables.
	gray_stack: Vec<usize>,
}

impl Arena {
	// Not used except as a provider of static assertions.
	#[allow(unused)]
	const HEADER_LAYOUT: Layout = {
		let ret = Layout::new::<ChunkHeader>();

		if ret.size() != ((1 << 20) / 64) {
			panic!("Chunk header size is incorrect.");
		}

		if ret.align() != 16 {
			panic!("Chunk header alignment is not 16.");
		}

		ret
	};

	const CHUNK_LAYOUT: Layout = Layout::new::<Chunk>();

	/// Number of shards in each chunk.
	const SHARD_LEN: usize = 8192;

	#[must_use]
	unsafe fn new() -> Self {
		let header = std::alloc::alloc(Self::CHUNK_LAYOUT).cast::<ChunkHeader>();
		std::ptr::write(header, ChunkHeader::default());
		let data = header.add(1).cast::<u8>();

		let header = NonNull::new(header).unwrap();
		let data = NonNull::new(data).unwrap();

		Self {
			header,
			data,
			ptr: data,
			gray_stack: vec![],
		}
	}

	#[must_use]
	unsafe fn bump(&mut self, layout: Layout) -> NonNull<u8> {
		debug_assert!(self.capacity() >= layout.size());

		let ret = self.ptr;

		self.ptr = NonNull::new_unchecked(self.ptr.as_ptr().add(layout.size()));

		let header = self.header.as_mut();
		let index = Self::index_of(ret);

		debug_assert!(header.shard_is_free(index) || header.shard_is_extent(index));
		header.set_shard_white(index);

		ret
	}

	/// How many bytes have been bumped so far?
	#[must_use]
	fn allocated(&self) -> usize {
		let cur_addr = self.ptr.as_ptr() as usize;
		let data_addr = self.data.as_ptr() as usize;
		cur_addr - data_addr
	}

	/// How many bytes can still be bumped?
	#[must_use]
	fn capacity(&self) -> usize {
		Self::CHUNK_LAYOUT.size() - self.allocated()
	}

	/// Used for indexing into the two header bitmaps.
	#[must_use]
	fn index_of(ptr: NonNull<u8>) -> usize {
		(ptr.as_ptr() as usize) & 0x00000000000fffff
	}
}

impl Drop for Arena {
	fn drop(&mut self) {
		unsafe {
			std::alloc::dealloc(self.header.as_ptr().cast::<u8>(), Self::CHUNK_LAYOUT);
		}
	}
}

/// Only used to provide an aligned layout.
#[repr(align(1048576))]
struct Chunk([ChunkHeader; 64]);

/// Makes up the very front of each [`Arena`]'s allocation.
#[derive(Debug, Default)]
#[repr(align(16))]
struct ChunkHeader {
	block_bits: BitArray<[usize; Arena::SHARD_LEN / 8]>,
	mark_bits: BitArray<[usize; Arena::SHARD_LEN / 8]>,
}

impl ChunkHeader {
	/// In bits, not pointer widths.
	#[cfg(target_pointer_width = "32")]
	const BITMAP_LEN: usize = (Arena::SHARD_LEN / 8) * 32;
	/// In bits, not pointer widths.
	#[cfg(target_pointer_width = "64")]
	const BITMAP_LEN: usize = (Arena::SHARD_LEN / 8) * 64;

	#[allow(unused)]
	#[must_use]
	fn shard_is_extent(&self, index: usize) -> bool {
		!self.block_bits[index] && !self.mark_bits[index]
	}

	#[allow(unused)]
	#[must_use]
	fn shard_is_free(&self, index: usize) -> bool {
		!self.block_bits[index] && self.mark_bits[index]
	}

	#[allow(unused)]
	#[must_use]
	fn shard_is_white(&self, index: usize) -> bool {
		self.block_bits[index] && !self.mark_bits[index]
	}

	#[allow(unused)]
	#[must_use]
	fn shard_is_black(&self, index: usize) -> bool {
		self.block_bits[index] && self.mark_bits[index]
	}

	#[allow(unused)]
	fn set_shard_extent(&mut self, index: usize) {
		self.block_bits.set(index, false);
		self.mark_bits.set(index, false);
	}

	#[allow(unused)]
	fn set_shard_free(&mut self, index: usize) {
		self.block_bits.set(index, false);
		self.mark_bits.set(index, true);
	}

	#[allow(unused)]
	fn set_shard_white(&mut self, index: usize) {
		self.block_bits.set(index, true);
		self.mark_bits.set(index, false);
	}

	#[allow(unused)]
	fn set_shard_black(&mut self, index: usize) {
		self.block_bits.set(index, true);
		self.mark_bits.set(index, true);
	}
}

/// The front of any heap allocation.
#[derive(Debug)]
#[repr(align(16))]
struct RegionHeader {
	flags: RegionFlags,
	tinfo: Handle<TypeInfo>,
}

/// Not inlined into a huge allocation; kept in the [`Heap::huge`] hash table.
#[derive(Debug)]
struct HugeHeader {
	layout: Layout,
	block: bool,
	mark: bool,
}

bitflags::bitflags! {
	pub struct RegionFlags: u8 {
		/// Gets set immediately for newly-created objects.
		const GRAY = 1 << 0;
		const DESTROYED = 1 << 1;
	}
}

#[cfg(test)]
mod test {
	use crate::vzs::Project;

	use super::*;

	/// Verify that the address masking functions allowing a region to retrieve
	/// its home chunk's header are working properly.
	#[test]
	fn arena_resolution() {
		let project = Project::default();
		let mut runtime = Runtime::default();
		let t = project.get::<TypeInfo>("i32").unwrap();

		unsafe {
			let ptr = runtime.alloc_t(t.into());
			let a = ptr.chunk();
			assert_eq!(a.as_ref().block_bits.len(), ChunkHeader::BITMAP_LEN);
			assert_eq!(a.as_ref().mark_bits.len(), ChunkHeader::BITMAP_LEN);
		}
	}
}
