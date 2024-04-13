//! [`PushVec`]; a concurrent, append-only counterpart to [`Vec`].
//!
//! This module is a fork of the [append-only-vec] crate by [David Roundy].
//! For licensing information, see the repository's ATTRIB.md file.
//!
//! [append-only-vec]: https://crates.io/crates/append-only-vec
//! [David Roundy]: https://github.com/droundy

use std::{
    cell::UnsafeCell,
    ops::Index,
    sync::atomic::{AtomicUsize, Ordering},
};

/// A concurrent, append-only counterpart to [`Vec`].
pub struct PushVec<T> {
    count: AtomicUsize,
    reserved: AtomicUsize,
    data: [UnsafeCell<*mut T>; BITS_USED - 1 - 3],
}

impl<T> PushVec<T> {
    /// Construct a new, empty array. This function performs no heap allocation.
    #[must_use]
    pub const fn new() -> Self {
        PushVec {
            count: AtomicUsize::new(0),
            reserved: AtomicUsize::new(0),
            data: [Self::EMPTY; BITS_USED - 1 - 3],
        }
    }

    /// Return an `Iterator` over the elements of the vec.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator {
        // FIXME this could be written to be a little more efficient probably,
        // if we made it read each pointer only once.  On the other hand, that
        // could make a reversed iterator less efficient?
        (0..self.len()).map(|i| unsafe { self.get_unchecked(i) })
    }

    /// Get the length of the array.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }

    /// Append an element to the array.
    ///
    /// This is notable in that it doesn't require a `&mut self`, because it
    /// does appropriate atomic synchronization.
    #[must_use]
    pub fn push(&self, val: T) -> usize {
        let idx = self.reserved.fetch_add(1, Ordering::Relaxed);
        let (array, offset) = indices(idx);
        let ptr = if self.len() < 1 + idx - offset {
            // We are working on a new array, which may not have been allocated...
            if offset == 0 {
                // It is our job to allocate the array!  The size of the array
                // is determined in the self.layout method, which needs to be
                // consistent with the indices function.
                let layout = self.layout(array);
                let ptr = unsafe { std::alloc::alloc(layout) } as *mut T;

                unsafe {
                    *self.data[array as usize].get() = ptr;
                }

                ptr
            } else {
                // We need to wait for the array to be allocated.
                while self.len() < 1 + idx - offset {
                    std::hint::spin_loop();
                }
                // The Ordering::Acquire semantics of self.len() ensures that
                // this pointer read will get the non-null pointer allocated
                // above.
                unsafe { *self.data[array as usize].get() }
            }
        } else {
            // The Ordering::Acquire semantics of self.len() ensures that
            // this pointer read will get the non-null pointer allocated
            // above.
            unsafe { *self.data[array as usize].get() }
        };

        // The contents of this offset are guaranteed to be unused (so far)
        // because we got the idx from our fetch_add above, and ptr is
        // guaranteed to be valid because of the loop we used above, which used
        // self.len() which has Ordering::Acquire semantics.
        unsafe { (ptr.add(offset)).write(val) };

        // Now we need to increase the size of the vec, so it can get read.  We
        // use Release upon success, to ensure that the value which we wrote is
        // visible to any thread that has confirmed that the count is big enough
        // to read that element.  In case of failure, we can be relaxed, since
        // we don't do anything with the result other than try again.
        while self
            .count
            .compare_exchange(idx, idx + 1, Ordering::Release, Ordering::Relaxed)
            .is_err()
        {
            // This means that someone else *started* pushing before we started,
            // but hasn't yet finished.  We have to wait for them to finish
            // pushing before we can update the count.  Note that using a
            // spinloop here isn't really ideal, but except when allocating a
            // new array, the window between reserving space and using it is
            // pretty small, so contention will hopefully be rare, and having a
            // context switch during that interval will hopefully be vanishingly
            // unlikely.
            std::hint::spin_loop();
        }

        idx
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert into a standard `Vec`.
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.len());

        for idx in 0..self.len() {
            let (array, offset) = indices(idx);
            // We use a Relaxed load of the pointer, because the loop above (which
            // ends before `self.len()`) should ensure that the data we want is
            // already visible, since it Acquired `self.count` which synchronizes
            // with the write in `self.push`.
            let ptr = unsafe { *self.data[array as usize].get() };

            // Copy the element value. The copy remaining in the array must not
            // be used again (i.e. make sure we do not drop it).
            let value = unsafe { ptr.add(offset).read() };

            vec.push(value);
        }

        // Prevent dropping the copied-out values by marking the count as 0 before
        // our own drop is run.
        self.count.store(0, Ordering::Relaxed);

        vec
    }
}

impl<T> std::fmt::Debug for PushVec<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Default for PushVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<usize> for PushVec<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        assert!(idx < self.len()); // this includes the required ordering memory barrier
        let (array, offset) = indices(idx);
        // The ptr value below is safe, because the length check above will
        // ensure that the data we want is already visible, since it used
        // Ordering::Acquire on `self.count` which synchronizes with the
        // Ordering::Release write in `self.push`.
        let ptr = unsafe { *self.data[array as usize].get() };
        unsafe { &*ptr.add(offset) }
    }
}

impl<T> Drop for PushVec<T> {
    fn drop(&mut self) {
        // First we'll drop all the `T` in a slightly sloppy way.
        // FIXME: this could be optimized to avoid reloading the `ptr`.
        for idx in 0..self.len() {
            self.free(idx);
        }

        // Now we will free all the arrays.
        self.free_arrays();
    }
}

/// An [`Iterator`] for the values contained in the [`PushVec`].
#[derive(Debug)]
pub struct IntoIter<T> {
    inner: PushVec<T>,
    remaining: std::ops::Range<usize>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.start == self.remaining.end {
            return None;
        }

        let idx = self.remaining.start;
        self.remaining.start += 1;

        let (array, offset) = indices(idx);

        let value = unsafe {
            let ptr = *self.inner.data[array as usize].get();
            // Copy the element value. The copy remaining in the array must not
            // be used again (i.e. make sure we do not drop it).
            ptr.add(offset).read()
        };

        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining.len(), Some(self.remaining.len()))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.remaining.len()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for idx in self.remaining.clone() {
            self.inner.free(idx);
        }

        self.inner.count.store(0, Ordering::Relaxed);
    }
}

impl<T> IntoIterator for PushVec<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    /// Consume the `PushVec` and yield all of its elements in the same order that
    /// they were added. This function and the returned iterator are allocation-free
    /// and copy-free.
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();

        IntoIter {
            inner: self,
            remaining: 0..len,
        }
    }
}

// Internal details ////////////////////////////////////////////////////////////

unsafe impl<T: Send> Send for PushVec<T> {}
unsafe impl<T: Sync + Send> Sync for PushVec<T> {}

const BITS: usize = std::mem::size_of::<usize>() * 8;

#[cfg(target_arch = "x86_64")]
const BITS_USED: usize = 48;
#[cfg(all(not(target_arch = "x86_64"), target_pointer_width = "64"))]
const BITS_USED: usize = 64;
#[cfg(target_pointer_width = "32")]
const BITS_USED: usize = 32;

// This takes an index into a vec, and determines which data array will hold it
// (the first return value), and what the index will be into that data array
// (second return value)
//
// The ith data array holds 1<<i values.
const fn indices(i: usize) -> (u32, usize) {
    let i = i + 8;
    let bin = BITS as u32 - 1 - i.leading_zeros();
    let bin = bin - 3;
    let offset = i - bin_size(bin);
    (bin, offset)
}

const fn bin_size(array: u32) -> usize {
    (1 << 3) << array
}

impl<T> PushVec<T> {
    /// This is a `const` rather than a function so that it can be used to
    /// initialize fixed-length arrays.
    #[allow(clippy::declare_interior_mutable_const)]
    const EMPTY: UnsafeCell<*mut T> = UnsafeCell::new(std::ptr::null_mut());

    /// Index the vec without checking the bounds.
    ///
    /// To use this correctly, you *must* first ensure that the `idx <
    /// self.len()`.  This not only prevents overwriting the bounds, but also
    /// creates the memory barriers to ensure that the data is visible to the
    /// current thread.  In single-threaded code, however, it is not needed to
    /// call `self.len()` explicitly (if e.g. you have counted the number of
    /// elements pushed).
    unsafe fn get_unchecked(&self, idx: usize) -> &T {
        let (array, offset) = indices(idx);
        // We use a Relaxed load of the pointer, because the length check (which
        // was supposed to be performed) should ensure that the data we want is
        // already visible, since self.len() used Ordering::Acquire on
        // `self.count` which synchronizes with the Ordering::Release write in
        // `self.push`.
        let ptr = *self.data[array as usize].get();
        &*ptr.add(offset)
    }

    fn layout(&self, array: u32) -> std::alloc::Layout {
        std::alloc::Layout::array::<T>(bin_size(array)).unwrap()
    }

    fn free(&mut self, idx: usize) {
        let (array, offset) = indices(idx);
        // We use a Relaxed load of the pointer, because the loop above (which
        // ends before `self.len()`) should ensure that the data we want is
        // already visible, since it Acquired `self.count` which synchronizes
        // with the write in `self.push`.
        let ptr = unsafe { *self.data[array as usize].get() };

        unsafe {
            std::ptr::drop_in_place(ptr.add(offset));
        }
    }

    fn free_arrays(&mut self) {
        for array in 0..self.data.len() as u32 {
            // This load is relaxed because no other thread can have a reference
            // to Self because we have a &mut self.
            let ptr = unsafe { *self.data[array as usize].get() };

            if !ptr.is_null() {
                let layout = self.layout(array);
                unsafe { std::alloc::dealloc(ptr as *mut u8, layout) };
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_indices() {
        for i in 0..32 {
            println!("{:3}: {} {}", i, indices(i).0, indices(i).1);
        }
        let mut array = 0;
        let mut offset = 0;
        let mut index = 0;
        while index < 1000 {
            index += 1;
            offset += 1;
            if offset >= bin_size(array) {
                offset = 0;
                array += 1;
            }
            assert_eq!(indices(index), (array, offset));
        }
    }

    #[test]
    fn test_pushing_and_indexing() {
        let v = PushVec::<usize>::new();

        for n in 0..50 {
            let _ = v.push(n);

            assert_eq!(v.len(), n + 1);

            for i in 0..(n + 1) {
                assert_eq!(v[i], i);
            }
        }

        let vec: Vec<usize> = v.iter().copied().collect();
        let ve2: Vec<usize> = (0..50).collect();
        assert_eq!(vec, ve2);
    }

    #[test]
    fn test_parallel_pushing() {
        use std::sync::Arc;
        let v = Arc::new(PushVec::<u64>::new());
        let mut threads = Vec::new();
        const N: u64 = 100;
        for thread_num in 0..N {
            let v = v.clone();
            threads.push(std::thread::spawn(move || {
                let which1 = v.push(thread_num);
                let which2 = v.push(thread_num);
                assert_eq!(v[which1 as usize], thread_num);
                assert_eq!(v[which2 as usize], thread_num);
            }));
        }
        for t in threads {
            t.join().ok();
        }
        for thread_num in 0..N {
            assert_eq!(2, v.iter().copied().filter(|&x| x == thread_num).count());
        }
    }

    #[test]
    fn test_into_vec() {
        struct SafeToDrop(bool);

        impl Drop for SafeToDrop {
            fn drop(&mut self) {
                assert!(self.0);
            }
        }

        let v = PushVec::new();

        for _ in 0..50 {
            let _ = v.push(SafeToDrop(false));
        }

        let mut v = v.into_vec();

        for i in v.iter_mut() {
            i.0 = true;
        }
    }
}
