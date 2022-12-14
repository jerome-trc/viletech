//! Sparse set and vector data structures.
//!
//! This is lifted directly from the
//! [`sparse_set`](https://docs.rs/sparse_set/latest/sparse_set/) crate, but has
//! undergone minor modifications to not rely on unstable features `allocator_api`
//! and `type_alias_impl_trait`. Unit tests were also stripped, since the upstream
//! tests prove the soundness of this code themselves.
//!
//! The code is provided under the
//! [Apache 2.0 License](https://github.com/sgodwincs/sparse_set/blob/master/LICENSE-APACHE)
//! and [MIT License](https://github.com/sgodwincs/sparse_set/blob/master/LICENSE-MIT) and
//! specifically used under the terms of the former. See inside the source file
//! iself to find both notices attached.
//!
//! This module is to be removed immediately in the event of one of the following:
//! - Both aforementioned features stabilize. In that case, link to this crate remotely.
//! - Another sparse set implementation is published. In that case, use it.
//! - This crate stops depending on unstable features. In that case, link to it remotely.

#![allow(unsafe_code)]

/*

							  Apache License
						Version 2.0, January 2004
					 http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
	   Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
	   stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
	   that You distribute, all copyright, patent, trademark, and
	   attribution notices from the Source form of the Work,
	   excluding those notices that do not pertain to any part of
	   the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
	   distribution, then any Derivative Works that You distribute must
	   include a readable copy of the attribution notices contained
	   within such NOTICE file, excluding those notices that do not
	   pertain to any part of the Derivative Works, in at least one
	   of the following places: within a NOTICE text file distributed
	   as part of the Derivative Works; within the Source form or
	   documentation, if provided along with the Derivative Works; or,
	   within a display generated by the Derivative Works, if and
	   wherever such third-party notices normally appear. The contents
	   of the NOTICE file are for informational purposes only and
	   do not modify the License. You may add Your own attribution
	   notices within Derivative Works that You distribute, alongside
	   or as an addendum to the NOTICE text from the Work, provided
	   that such additional attribution notices cannot be construed
	   as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS

*/

/*

MIT License

Copyright (c) 2022 Scott Godwin

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

*/

use std::{
	collections::TryReserveError,
	fmt::{self, Debug, Formatter},
	hash::{Hash, Hasher},
	marker::PhantomData,
	mem,
	num::NonZeroUsize,
	ops::{Deref, DerefMut, Index, IndexMut},
};

/// A sparsely populated vector, written `SparseVec<I, T>`, where `I` is the index type and `T` is the value type.
///
/// For operation complexity notes, *n* is the number of values in the sparse vec and *m* is the value of the largest
/// index in the sparse vec. Note that *m* will always be at least as large as *n*.
pub struct SparseVec<I, T> {
	values: Vec<Option<T>>,
	_marker: PhantomData<I>,
}

impl<I, T> SparseVec<I, T> {
	/// Constructs a new, empty `SparseVec<I, T>`.
	///
	/// The sparse vec will not allocate until elements are inserted into it.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// # #[allow(unused_mut)]
	/// let mut vec: SparseVec<usize, u32> = SparseVec::new();
	/// ```
	#[must_use]
	pub const fn new() -> Self {
		Self {
			values: Vec::new(),
			_marker: PhantomData,
		}
	}

	/// Constructs a new, empty `SparseVec<I, T>` with the specified capacity.
	///
	/// The sparse vec will be able to hold exactly `capacity` elements without reallocating. If `capacity` is 0, the
	/// sparse vec will not allocate.
	///
	/// It is important to note that although the returned sparse vec has the *capacity* specified, the sparse vec will
	/// have a zero *length*.
	///
	/// # Panics
	///
	/// Panics if the new capacity exceeds `isize::MAX` bytes.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::with_capacity(10);
	///
	/// // The sparse vec contains no items, even though it has capacity for more.
	/// assert_eq!(vec.len(), 0);
	/// assert_eq!(vec.capacity(), 10);
	///
	/// // These are all done without reallocating...
	/// for i in 0..10 {
	///   vec.insert(i, i);
	/// }
	///
	/// assert_eq!(vec.len(), 10);
	/// assert_eq!(vec.capacity(), 10);
	///
	/// // ...but this will make the sparse vec reallocate.
	/// vec.insert(10, 10);
	/// vec.insert(11, 11);
	/// assert_eq!(vec.len(), 12);
	/// assert!(vec.capacity() >= 12);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			values: Vec::with_capacity(capacity),
			_marker: PhantomData,
		}
	}
}

impl<I, T> SparseVec<I, T> {
	/// Extracts a slice containing the entire underlying buffer.
	#[must_use]
	pub fn as_slice(&self) -> &[Option<T>] {
		&self.values
	}

	/// Extracts a mutable slice of the entire underlying buffer.
	#[must_use]
	pub fn as_mut_slice(&mut self) -> &mut [Option<T>] {
		&mut self.values
	}

	/// Returns a raw pointer to the buffer, or a dangling raw pointer valid for zero sized reads if the sparse vec didn't
	/// allocate.
	///
	/// The caller must ensure that the sparse vec outlives the pointer this function returns, or else it will end up
	/// pointing to garbage. Modifying the sparse vec may cause its buffer to be reallocated, which would also make any
	/// pointers to it invalid.
	///
	/// The caller must also ensure that the memory the pointer (non-transitively) points to is never written to (except
	/// inside an `UnsafeCell`) using this pointer or any pointer derived from it.
	#[must_use]
	pub fn as_ptr(&self) -> *const Option<T> {
		self.values.as_ptr()
	}

	/// Returns an unsafe mutable pointer to the sparse vec's buffer.
	///
	/// The caller must ensure that the sparse vec outlives the pointer this function returns, or else it will end up
	/// pointing to garbage. Modifying the sparse vec may cause its buffer to be reallocated, which would also make any
	/// pointers to it invalid.
	///
	/// Swapping elements in the mutable slice changes which indices point to which values.
	#[must_use]
	pub fn as_mut_ptr(&mut self) -> *mut Option<T> {
		self.values.as_mut_ptr()
	}

	/// Returns the number of elements the sparse vec can hold without reallocating.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let vec: SparseVec<usize, i32> = SparseVec::with_capacity(10);
	/// assert_eq!(vec.capacity(), 10);
	/// ```
	#[must_use]
	pub fn capacity(&self) -> usize {
		self.values.capacity()
	}

	/// Clears the sparse vec, removing all values.
	///
	/// Note that this method has no effect on the allocated capacity of the sparse vec.
	/// # Examples
	///
	/// This operation is *O*(*m*).
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	///
	/// vec.insert(0, 1);
	/// vec.insert(1, 2);
	/// vec.insert(2, 3);
	///
	/// vec.clear();
	///
	/// assert!(vec.is_empty());
	/// ```
	pub fn clear(&mut self) {
		self.values.clear();
	}

	/// Returns `true` if the sparse vec contains no elements.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	/// assert!(vec.is_empty());
	///
	/// vec.insert(0, 1);
	/// assert!(!vec.is_empty());
	/// ```
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns an iterator over the sparse vec's values.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	///
	/// Consuming the iterator is an *O*(*m*) operation.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	/// vec.insert(0, 1);
	/// vec.insert(1, 2);
	/// vec.insert(2, 3);
	///
	/// let mut iterator = vec.iter();
	///
	/// assert_eq!(iterator.next(), Some(&Some(1)));
	/// assert_eq!(iterator.next(), Some(&Some(2)));
	/// assert_eq!(iterator.next(), Some(&Some(3)));
	/// ```
	pub fn iter(&self) -> impl Iterator<Item = &Option<T>> {
		self.values.iter()
	}

	/// Returns an iterator that allows modifying each value.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	/// vec.insert(0, 1);
	/// vec.insert(1, 2);
	/// vec.insert(2, 3);
	///
	/// for elem in vec.iter_mut() {
	///     *elem = elem.map(|value| value + 2);
	/// }
	///
	/// assert!(vec.iter().eq(&[Some(3), Some(4), Some(5)]));
	/// ```
	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<T>> {
		self.values.iter_mut()
	}

	/// Returns the number of elements in the sparse vec, also referred to as its 'len'.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	/// vec.insert(0, 1);
	/// vec.insert(1, 2);
	/// vec.insert(2, 3);
	///
	/// assert_eq!(vec.len(), 3);
	/// ```
	#[must_use]
	pub fn len(&self) -> usize {
		self.values.len()
	}

	/// Reserves capacity for at least `additional` more elements to be inserted in the given `SparseVec<I, T>`.
	///
	/// The collection may reserve more space to avoid frequent reallocations. After calling `reserve`, the capacity will
	/// be greater than or equal to `self.len() + additional`. Does nothing if capacity is already sufficient.
	///
	/// # Panics
	///
	/// Panics if the new capacity exceeds `isize::MAX` bytes.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	/// vec.insert(0, 1);
	/// vec.reserve(10);
	/// assert!(vec.capacity() >= 11);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn reserve(&mut self, additional: usize) {
		self.values.reserve(additional);
	}

	/// Reserves the minimum capacity for exactly `additional` more elements to be inserted in the given
	/// `SparseSet<I, T>`'.
	///
	/// After calling `reserve_exact`, the capacity will be greater than or equal to `self.len() + additional`. Does
	/// nothing if the capacity is already sufficient.
	///
	/// Note that the allocator may give the collection more space than it requests. Therefore, capacity can not be relied
	/// upon to be precisely minimal. Prefer [`reserve`] if future insertions are expected.
	///
	/// [`reserve`]: SparseVec::reserve
	///
	/// # Panics
	///
	/// Panics if the new capacity exceeds `isize::MAX` bytes.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	/// vec.insert(0, 1);
	/// vec.reserve_exact(10);
	/// assert!(vec.capacity() >= 11);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn reserve_exact(&mut self, additional: usize) {
		self.values.reserve_exact(additional);
	}

	/// Tries to reserve capacity for at least `additional` more elements to be inserted in the given `SparseVec<I, T>`.
	///
	/// The collection may reserve more space to avoid frequent reallocations. After calling `try_reserve`, capacity will
	/// be greater than or equal to `self.len() + additional`. Does nothing if capacity is already sufficient.
	///
	/// # Errors
	///
	/// If the capacity overflows, or the allocator reports a failure, then an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use std::collections::TryReserveError;
	///
	/// # use sparse_set::SparseVec;
	///
	/// fn process_data(data: &[u32]) -> Result<SparseVec<usize, u32>, TryReserveError> {
	///   let mut output = SparseVec::new();
	///
	///   // Pre-reserve the memory, exiting if we can't.
	///   output.try_reserve(data.len())?;
	///
	///   // Now we know this can't OOM in the middle of our complex work.
	///   for (index, value) in data.iter().cloned().enumerate() {
	///     output.insert(index, value);
	///   }
	///
	///   Ok(output)
	/// }
	/// # process_data(&[1, 2, 3]).expect("why is the test harness OOMing on 12 bytes?");
	/// ```
	pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.values.try_reserve(additional)
	}

	/// Tries to reserve the minimum capacity for exactly `additional` elements to be inserted in the given
	/// `SparseVec<T>`.
	///
	/// After calling `try_reserve_exact`, capacity will be greater than or equal to `self.len() + additional` if it
	/// returns `Ok(())`. Does nothing if the capacity is already sufficient.
	///
	/// Note that the allocator may give the collection more space than it requests. Therefore, capacity can not be relied
	/// upon to be precisely minimal. Prefer [`try_reserve`] if future insertions are expected.
	///
	/// [`try_reserve`]: SparseVec::try_reserve
	///
	/// # Errors
	///
	/// If the capacity overflows, or the allocator reports a failure, then an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use std::collections::TryReserveError;
	///
	/// # use sparse_set::SparseVec;
	///
	/// fn process_data(data: &[u32]) -> Result<SparseVec<usize, u32>, TryReserveError> {
	///   let mut output = SparseVec::new();
	///
	///   // Pre-reserve the memory, exiting if we can't.
	///   output.try_reserve_exact(data.len())?;
	///
	///   // Now we know this can't OOM in the middle of our complex work.
	///   for (index, value) in data.iter().cloned().enumerate() {
	///     output.insert(index, value);
	///   }
	///
	///   Ok(output)
	/// }
	/// # process_data(&[1, 2, 3]).expect("why is the test harness OOMing on 12 bytes?");
	/// ```
	pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.values.try_reserve_exact(additional)
	}

	/// Shrinks the capacity of the sparse vec as much as possible.
	///
	/// It will drop down as close as possible to the length but the allocator may still inform the sparse vec that
	/// there is space for a few more elements.
	///
	/// This operation is *O*(*m*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::with_capacity(10);
	/// vec.insert(0, 1);
	/// vec.insert(1, 2);
	/// assert_eq!(vec.capacity(), 10);
	///
	/// vec.shrink_to_fit();
	/// assert!(vec.capacity() >= 2);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn shrink_to_fit(&mut self) {
		self.values.truncate(self.max_index());
		self.values.shrink_to_fit();
	}

	/// Shrinks the capacity of the sparse vec with a lower bound.
	///
	/// This will also reduce `len` as any empty indices after the maximum index will be removed.
	///
	/// The capacity will remain at least as large as both the length and the supplied value.
	///
	/// If the current capacity is less than the lower limit, this is a no-op.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::with_capacity(10);
	/// vec.insert(0, 1);
	/// vec.insert(1, 2);
	/// assert_eq!(vec.capacity(), 10);
	/// vec.shrink_to(4);
	/// assert!(vec.capacity() >= 4);
	/// vec.shrink_to(0);
	/// assert!(vec.capacity() >= 2);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn shrink_to(&mut self, min_capacity: usize) {
		let len = self.max_index();

		if min_capacity < len {
			self.values.truncate(len);
		}

		self.values.shrink_to(min_capacity);
	}

	/// Returns the largest index in the sparse vec, or None if it is empty.
	///
	/// This operation is *O*(*m*).
	#[must_use]
	fn max_index(&self) -> usize {
		for (index, value) in self.values.iter().rev().enumerate() {
			if value.is_some() {
				return self.values.len() - index;
			}
		}

		0
	}
}

impl<I: SparseSetIndex, T> SparseVec<I, T> {
	/// Returns `true` if the sparse vec contains an element at the given index.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	///
	/// vec.insert(0, 1);
	/// vec.insert(1, 2);
	/// vec.insert(2, 3);
	///
	/// assert!(vec.contains(0));
	/// assert!(!vec.contains(100));
	/// ```
	#[must_use]
	pub fn contains(&self, index: I) -> bool {
		self.get(index).is_some()
	}

	/// Returns a reference to an element pointed to by the index, if it exists.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	///
	/// vec.insert(0, 1);
	/// vec.insert(1, 2);
	/// vec.insert(2, 3);
	/// assert_eq!(Some(&2), vec.get(1));
	/// assert_eq!(None, vec.get(3));
	///
	/// vec.remove(1);
	/// assert_eq!(None, vec.get(1));
	/// ```
	#[must_use]
	pub fn get(&self, index: I) -> Option<&T> {
		self.values.get(index.into()).and_then(Option::as_ref)
	}

	/// Returns a mutable reference to an element pointed to by the index, if it exists.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	///
	/// vec.insert(0, 1);
	/// vec.insert(1, 2);
	/// vec.insert(2, 3);
	///
	/// if let Some(elem) = vec.get_mut(1) {
	///   *elem = 42;
	/// }
	///
	/// assert!(vec.iter().eq(&[Some(1), Some(42), Some(3)]));
	/// ```
	#[must_use]
	pub fn get_mut(&mut self, index: I) -> Option<&mut T> {
		self.values.get_mut(index.into()).and_then(Option::as_mut)
	}

	/// Inserts an element at position `index` within the sparse vec.
	///
	/// If a value already existed at `index`, it will be replaced and returned.
	///
	/// If `index` is greater than `capacity`, then an allocation will take place.
	///
	/// This operation is amortized *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	///
	/// vec.insert(0, 1);
	/// vec.insert(1, 4);
	/// vec.insert(2, 2);
	/// vec.insert(3, 3);
	///
	/// assert!(vec.iter().eq(&[Some(1), Some(4), Some(2), Some(3)]));
	/// vec.insert(5, 5);
	/// assert!(vec.iter().eq(&[Some(1), Some(4), Some(2), Some(3), None, Some(5)]));
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn insert(&mut self, index: I, value: T) -> Option<T> {
		let index = index.into();

		if index >= self.len() {
			self.values.resize_with(index + 1, || None);
		}

		unsafe { self.values.get_unchecked_mut(index) }.replace(value)
	}

	/// Removes and returns the element at position `index` within the sparse vec, if it exists.
	///
	/// This does not change the length of the sparse vec as the value is replaced with `None`.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseVec;
	/// #
	/// let mut vec = SparseVec::new();
	/// vec.insert(0, 1);
	/// vec.insert(1, 2);
	/// vec.insert(2, 3);
	///
	/// assert_eq!(vec.remove(1), Some(2));
	/// assert!(vec.iter().eq(&[Some(1), None, Some(3)]));
	/// ```
	#[must_use]
	pub fn remove(&mut self, index: I) -> Option<T> {
		let index = index.into();
		self.values.get_mut(index).and_then(Option::take)
	}
}

impl<I, T> AsRef<Self> for SparseVec<I, T> {
	fn as_ref(&self) -> &Self {
		self
	}
}

impl<I, T> AsMut<Self> for SparseVec<I, T> {
	fn as_mut(&mut self) -> &mut Self {
		self
	}
}

impl<I, T> AsRef<[Option<T>]> for SparseVec<I, T> {
	fn as_ref(&self) -> &[Option<T>] {
		&self.values
	}
}

impl<I, T> AsMut<[Option<T>]> for SparseVec<I, T> {
	fn as_mut(&mut self) -> &mut [Option<T>] {
		&mut self.values
	}
}

impl<I, T: Clone + Clone> Clone for SparseVec<I, T> {
	fn clone(&self) -> Self {
		Self {
			values: self.values.clone(),
			_marker: PhantomData,
		}
	}
}
impl<I, T> Default for SparseVec<I, T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<I, T> Deref for SparseVec<I, T> {
	type Target = [Option<T>];

	fn deref(&self) -> &[Option<T>] {
		&self.values
	}
}

impl<I, T> DerefMut for SparseVec<I, T> {
	fn deref_mut(&mut self) -> &mut [Option<T>] {
		&mut self.values
	}
}

impl<I, T: fmt::Debug> fmt::Debug for SparseVec<I, T> {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.values.fmt(formatter)
	}
}

#[cfg(not(no_global_oom_handling))]
impl<'a, I: SparseSetIndex, T: Copy + 'a + 'a> Extend<(I, &'a T)> for SparseVec<I, T> {
	fn extend<Iter: IntoIterator<Item = (I, &'a T)>>(&mut self, iter: Iter) {
		for (index, &value) in iter {
			let _ = self.insert(index, value);
		}
	}
}

#[cfg(not(no_global_oom_handling))]
impl<I: SparseSetIndex, T> Extend<(I, T)> for SparseVec<I, T> {
	fn extend<Iter: IntoIterator<Item = (I, T)>>(&mut self, iter: Iter) {
		for (index, value) in iter {
			let _ = self.insert(index, value);
		}
	}
}

#[cfg(not(no_global_oom_handling))]
impl<I: SparseSetIndex, T, const N: usize> From<[(I, T); N]> for SparseVec<I, T> {
	fn from(slice: [(I, T); N]) -> Self {
		let mut vec = Self::with_capacity(slice.len());

		for (index, value) in slice {
			let _ = vec.insert(index, value);
		}

		vec
	}
}

#[cfg(not(no_global_oom_handling))]
impl<I: SparseSetIndex, T> FromIterator<(I, T)> for SparseVec<I, T> {
	fn from_iter<Iter: IntoIterator<Item = (I, T)>>(iter: Iter) -> Self {
		let iter = iter.into_iter();
		let capacity = iter
			.size_hint()
			.1
			.map_or_else(|| iter.size_hint().0, |size_hint| size_hint);
		let mut vec = Self::with_capacity(capacity);

		for (index, value) in iter {
			let _ = vec.insert(index, value);
		}

		vec
	}
}

impl<I, T: Hash> Hash for SparseVec<I, T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.values.hash(state);
	}
}

impl<I: SparseSetIndex, T> Index<I> for SparseVec<I, T> {
	type Output = T;

	fn index(&self, index: I) -> &Self::Output {
		self.get(index).unwrap()
	}
}

impl<I: SparseSetIndex, T> IndexMut<I> for SparseVec<I, T> {
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		self.get_mut(index).unwrap()
	}
}

impl<I, T: PartialEq> PartialEq for SparseVec<I, T> {
	fn eq(&self, other: &Self) -> bool {
		self.values == other.values
	}
}

impl<I, T: Eq> Eq for SparseVec<I, T> {}

/// A type with this trait indicates it can be used as an index into a `SparseSet`.
///
/// Two indices may the same index if they are unequal, but if equal they must return the same index.
pub trait SparseSetIndex: Copy + Into<usize> {}

impl SparseSetIndex for usize {}

/// A sparsely populated set, written `SparseSet<I, T>`, where `I` is the index type and `T` is the value type.
///
/// For operation complexity notes, *n* is the number of values in the sparse set and *m* is the value of the largest
/// index in the sparse set. Note that *m* will always be at least as large as *n*.
#[derive(Clone)]
pub struct SparseSet<I, T> {
	/// The dense buffer, i.e., the buffer containing the actual data values of type `T`.
	dense: Vec<T>,

	/// The sparse buffer, i.e., the buffer where each index may correspond to an index into `dense`.
	sparse: SparseVec<I, NonZeroUsize>,

	/// All the existing indices in `sparse`.
	///
	/// The indices here will always be in order based on the `dense` buffer.
	indices: Vec<I>,
}

impl<I, T> SparseSet<I, T> {
	/// Constructs a new, empty `SparseSet<I, T>`.
	///
	/// The sparse set will not allocate until elements are inserted into it.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// # #[allow(unused_mut)]
	/// let mut set: SparseSet<usize, u32> = SparseSet::new();
	/// ```
	#[must_use]
	pub const fn new() -> Self {
		Self {
			dense: Vec::new(),
			sparse: SparseVec::new(),
			indices: Vec::new(),
		}
	}

	/// Constructs a new, empty `SparseSet<I, T>` with the specified capacity.
	///
	/// The sparse set will be able to hold exactly `capacity` elements without reallocating. If `capacity` is 0, the
	/// sparse set will not allocate.
	///
	/// It is important to note that although the returned sparse set has the *capacity* specified, the sparse set will
	/// have a zero *length*.
	///
	/// # Panics
	///
	/// Panics if the new capacity exceeds `isize::MAX` bytes.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::with_capacity(11, 10);
	///
	/// // The sparse set contains no items, even though it has capacity for more.
	/// assert_eq!(set.len(), 0);
	/// assert_eq!(set.dense_capacity(), 10);
	/// assert_eq!(set.sparse_capacity(), 11);
	///
	/// // These are all done without reallocating...
	/// for i in 0..10 {
	///   set.insert(i, i);
	/// }
	///
	/// assert_eq!(set.len(), 10);
	/// assert_eq!(set.dense_capacity(), 10);
	/// assert_eq!(set.sparse_capacity(), 11);
	///
	/// // ...but this will make the sparse set reallocate.
	/// set.insert(10, 10);
	/// set.insert(11, 11);
	/// assert_eq!(set.dense_len(), 12);
	/// assert!(set.dense_capacity() >= 12);
	/// assert!(set.sparse_capacity() >= 12);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	#[must_use]
	pub fn with_capacity(sparse_capacity: usize, dense_capacity: usize) -> Self {
		assert!(
			sparse_capacity >= dense_capacity,
			"Sparse capacity must be at least as large as the dense capacity."
		);
		Self::with_capacity(sparse_capacity, dense_capacity)
	}
}

impl<I, T> SparseSet<I, T> {
	/// Extracts a slice containing the entire dense buffer.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	#[must_use]
	pub fn as_dense_slice(&self) -> &[T] {
		&self.dense
	}

	/// Extracts a mutable slice of the entire dense buffer.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	///
	/// # Safety
	///
	/// The order of value in the dense buffer must be kept in sync with the order of indices in the index buffer.
	#[must_use]
	pub unsafe fn as_dense_mut_slice(&mut self) -> &mut [T] {
		&mut self.dense
	}

	/// Returns a raw pointer to the dense buffer, or a dangling raw pointer valid for zero sized reads if the sparse set
	/// didn't allocate.
	///
	/// The caller must ensure that the sparse set outlives the pointer this function returns, or else it will end up
	/// pointing to garbage. Modifying the sparse set may cause its buffer to be reallocated, which would also make any
	/// pointers to it invalid.
	///
	/// The caller must also ensure that the memory the pointer (non-transitively) points to is never written to (except
	/// inside an `UnsafeCell`) using this pointer or any pointer derived from it.
	#[must_use]
	pub fn as_dense_ptr(&self) -> *const T {
		self.dense.as_ptr()
	}

	/// Returns an unsafe mutable pointer to the sparse set's dense buffer.
	///
	/// The caller must ensure that the sparse set outlives the pointer this function returns, or else it will end up
	/// pointing to garbage. Modifying the sparse set may cause its buffer to be reallocated, which would also make any
	/// pointers to it invalid.
	///
	/// # Safety
	///
	/// The order of value in the dense buffer must be kept in sync with the order of indices in the index buffer.
	#[must_use]
	pub unsafe fn as_dense_mut_ptr(&mut self) -> *mut T {
		self.dense.as_mut_ptr()
	}

	/// Returns a slice over the sparse set's indices.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// assert_eq!(set.as_indices_slice(), &[0, 1, 2]);
	/// ```
	#[must_use]
	pub fn as_indices_slice(&self) -> &[I] {
		&self.indices
	}

	/// Extracts a mutable slice of the entire index buffer.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	///
	/// # Safety
	///
	/// The order of value in the dense buffer must be kept in sync with the order of indices in the index buffer.
	#[must_use]
	pub unsafe fn as_indices_mut_slice(&mut self) -> &mut [I] {
		&mut self.indices
	}

	/// Returns a raw pointer to the index buffer, or a dangling raw pointer valid for zero sized reads if the sparse set
	/// didn't allocate.
	///
	/// The caller must ensure that the sparse set outlives the pointer this function returns, or else it will end up
	/// pointing to garbage. Modifying the sparse set may cause its buffer to be reallocated, which would also make any
	/// pointers to it invalid.
	///
	/// The caller must also ensure that the memory the pointer (non-transitively) points to is never written to (except
	/// inside an `UnsafeCell`) using this pointer or any pointer derived from it.
	#[must_use]
	pub fn as_indices_ptr(&self) -> *const I {
		self.indices.as_ptr()
	}

	/// Returns an unsafe mutable pointer to the index buffer, or a dangling raw pointer valid for zero sized reads if the
	/// sparse set didn't allocate.
	///
	/// The caller must ensure that the sparse set outlives the pointer this function returns, or else it will end up
	/// pointing to garbage. Modifying the sparse set may cause its buffer to be reallocated, which would also make any
	/// pointers to it invalid.
	///
	/// # Safety
	///
	/// The order of value in the dense buffer must be kept in sync with the order of indices in the index buffer.
	#[must_use]
	pub unsafe fn as_indices_mut_ptr(&mut self) -> *mut I {
		self.indices.as_mut_ptr()
	}

	/// Returns the number of elements the dense buffer can hold without reallocating.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let set: SparseSet<usize, i32> = SparseSet::with_capacity(15, 10);
	/// assert_eq!(set.dense_capacity(), 10);
	/// ```
	#[must_use]
	pub fn dense_capacity(&self) -> usize {
		self.dense.capacity()
	}

	/// Returns the number of elements the sparse buffer can hold without reallocating.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let set: SparseSet<usize, i32> = SparseSet::with_capacity(15, 10);
	/// assert_eq!(set.sparse_capacity(), 15);
	/// ```
	#[must_use]
	pub fn sparse_capacity(&self) -> usize {
		self.sparse.capacity()
	}

	/// Clears the sparse set, removing all values.
	///
	/// Note that this method has no effect on the allocated capacity of the sparse set.
	///
	/// This operation is *O*(*m*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	///
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// set.clear();
	///
	/// assert!(set.is_empty());
	/// ```
	pub fn clear(&mut self) {
		self.dense.clear();
		self.indices.clear();
		self.sparse.clear();
	}

	/// Returns `true` if the sparse set contains no elements.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// assert!(set.is_empty());
	///
	/// set.insert(0, 1);
	/// assert!(!set.is_empty());
	/// ```
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.dense_len() == 0
	}

	/// Returns the number of elements in the dense set, also referred to as its '`dense_len`'.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// assert_eq!(set.dense_len(), 3);
	/// ```
	#[must_use]
	pub fn dense_len(&self) -> usize {
		self.dense.len()
	}

	/// Returns the number of elements in the sparse set, also referred to as its '`sparse_len`'.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(200, 3);
	///
	/// assert_eq!(set.sparse_len(), 201);
	/// ```
	#[must_use]
	pub fn sparse_len(&self) -> usize {
		self.sparse.len()
	}

	/// Clears the sparse set, returning all `(index, value)` pairs as an iterator.
	///
	/// The allocated memory is kept for reuse.
	///
	/// If the returned iterator is dropped before fully consumed, it drops the remaining `(index, value)` pairs. The
	/// returned iterator keeps a mutable borrow on the sparse set to optimize its implementation.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	///
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// assert!(set.drain().eq([(0, 1), (1, 2), (2, 3)]));
	/// ```
	pub fn drain(
		&mut self,
	) -> impl Iterator<Item = (I, T)> + DoubleEndedIterator + ExactSizeIterator + '_ {
		self.sparse.clear();
		self.indices.drain(..).zip(self.dense.drain(..))
	}

	/// Reserves capacity for at least `additional` more elements to be inserted in the given `SparseSet<I, T>`'s dense
	/// buffer.
	///
	/// The collection may reserve more space to avoid frequent reallocations. After calling `reserve`, the dense capacity
	/// will be greater than or equal to `self.dense_len() + additional`. Does nothing if capacity is already sufficient.
	///
	/// # Panics
	///
	/// Panics if the new capacity exceeds `isize::MAX` bytes.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.reserve_dense(10);
	/// assert!(set.dense_capacity() >= 11);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn reserve_dense(&mut self, additional: usize) {
		self.dense.reserve(additional);
	}

	/// Reserves capacity for at least `additional` more elements to be inserted in the given `SparseSet<I, T>`'s sparse
	/// buffer.
	///
	/// The collection may reserve more space to avoid frequent reallocations. After calling `reserve`, the sparse
	/// capacity will be greater than or equal to `self.sparse_len() + additional`. Does nothing if capacity is already
	/// sufficient.
	///
	/// # Panics
	///
	/// Panics if the new capacity exceeds `isize::MAX` bytes.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.reserve_sparse(10);
	/// assert!(set.sparse_capacity() >= 11);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn reserve_sparse(&mut self, additional: usize) {
		self.sparse.reserve(additional);
	}

	/// Reserves the minimum capacity for exactly `additional` more elements to be inserted in the given
	/// `SparseSet<I, T>`'s dense buffer.
	///
	/// After calling `reserve_exact`, the dense capacity will be greater than or equal to
	/// `self.dense_len() + additional`. Does nothing if the capacity is already sufficient.
	///
	/// Note that the allocator may give the collection more space than it requests. Therefore, capacity can not be relied
	/// upon to be precisely minimal. Prefer [`reserve_dense`] if future insertions are expected.
	///
	/// [`reserve_dense`]: SparseSet::reserve_dense
	///
	/// # Panics
	///
	/// Panics if the new capacity exceeds `isize::MAX` bytes.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.reserve_exact_dense(10);
	/// assert!(set.dense_capacity() >= 11);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn reserve_exact_dense(&mut self, additional: usize) {
		self.dense.reserve_exact(additional);
	}

	/// Reserves the minimum capacity for exactly `additional` more elements to be inserted in the given
	/// `SparseSet<I, T>`'s sparse buffer.
	///
	/// After calling `reserve_exact`, the sparse capacity will be greater than or equal to
	/// `self.sparse_len() + additional`. Does nothing if the capacity is already sufficient.
	///
	/// Note that the allocator may give the collection more space than it requests. Therefore, capacity can not be relied
	/// upon to be precisely minimal. Prefer [`reserve_sparse`] if future insertions are expected.
	///
	/// [`reserve_sparse`]: SparseSet::reserve_sparse
	///
	/// # Panics
	///
	/// Panics if the new capacity exceeds `isize::MAX` bytes.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.reserve_exact_sparse(10);
	/// assert!(set.sparse_capacity() >= 11);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn reserve_exact_sparse(&mut self, additional: usize) {
		self.sparse.reserve_exact(additional);
	}

	/// Tries to reserve capacity for at least `additional` more elements to be inserted in the given `SparseSet<I, T>`'s
	/// dense buffer.
	///
	/// The collection may reserve more space to avoid frequent reallocations. After calling `try_reserve_dense`, capacity
	/// will be greater than or equal to `self.dense_len() + additional`. Does nothing if capacity is already sufficient.
	///
	/// # Errors
	///
	/// If the capacity overflows, or the allocator reports a failure, then an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use std::collections::TryReserveError;
	///
	/// # use sparse_set::SparseSet;
	///
	/// fn process_data(data: &[u32]) -> Result<SparseSet<usize, u32>, TryReserveError> {
	///   let mut output = SparseSet::new();
	///
	///   // Pre-reserve the memory, exiting if we can't.
	///   output.try_reserve_dense(data.len())?;
	///
	///   // Now we know this can't OOM in the middle of our complex work.
	///   for (index, value) in data.iter().cloned().enumerate() {
	///     output.insert(index, value);
	///   }
	///
	///   Ok(output)
	/// }
	/// # process_data(&[1, 2, 3]).expect("why is the test harness OOMing on 12 bytes?");
	/// ```
	pub fn try_reserve_dense(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.dense.try_reserve(additional)
	}

	/// Tries to reserve capacity for at least `additional` more elements to be inserted in the given `SparseSet<I, T>`'s
	/// sparse buffer.
	///
	/// The collection may reserve more space to avoid frequent reallocations. After calling `try_reserve_sparse`,
	/// capacity will be greater than or equal to `self.sparse_len() + additional`. Does nothing if capacity is already
	/// sufficient.
	///
	/// # Errors
	///
	/// If the capacity overflows, or the allocator reports a failure, then an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use std::collections::TryReserveError;
	///
	/// # use sparse_set::SparseSet;
	///
	/// fn process_data(data: &[u32]) -> Result<SparseSet<usize, u32>, TryReserveError> {
	///   let mut output = SparseSet::new();
	///
	///   // Pre-reserve the memory, exiting if we can't.
	///   output.try_reserve_sparse(data.len())?;
	///
	///   // Now we know this can't OOM in the middle of our complex work.
	///   for (index, value) in data.iter().cloned().enumerate() {
	///     output.insert(index, value);
	///   }
	///
	///   Ok(output)
	/// }
	/// # process_data(&[1, 2, 3]).expect("why is the test harness OOMing on 12 bytes?");
	/// ```
	pub fn try_reserve_sparse(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.sparse.try_reserve(additional)
	}

	/// Tries to reserve the minimum capacity for exactly `additional` elements to be inserted in the given
	/// `SparseSet<T>`'s dense buffer.
	///
	/// After calling `try_reserve_exact_dense`, capacity will be greater than or equal to `self.dense_len() + additional`
	/// if it returns `Ok(())`. Does nothing if the capacity is already sufficient.
	///
	/// Note that the allocator may give the collection more space than it requests. Therefore, capacity can not be relied
	/// upon to be precisely minimal. Prefer [`try_reserve_dense`] if future insertions are expected.
	///
	/// [`try_reserve_dense`]: SparseSet::try_reserve_dense
	///
	/// # Errors
	///
	/// If the capacity overflows, or the allocator reports a failure, then an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use std::collections::TryReserveError;
	///
	/// # use sparse_set::SparseSet;
	///
	/// fn process_data(data: &[u32]) -> Result<SparseSet<usize, u32>, TryReserveError> {
	///   let mut output = SparseSet::new();
	///
	///   // Pre-reserve the memory, exiting if we can't.
	///   output.try_reserve_exact_dense(data.len())?;
	///
	///   // Now we know this can't OOM in the middle of our complex work.
	///   for (index, value) in data.iter().cloned().enumerate() {
	///     output.insert(index, value);
	///   }
	///
	///   Ok(output)
	/// }
	/// # process_data(&[1, 2, 3]).expect("why is the test harness OOMing on 12 bytes?");
	/// ```
	pub fn try_reserve_exact_dense(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.dense.try_reserve_exact(additional)
	}

	/// Tries to reserve the minimum capacity for exactly `additional` elements to be inserted in the given
	/// `SparseSet<T>`'s sparse buffer.
	///
	/// After calling `try_reserve_exact_sparse`, capacity will be greater than or equal to
	/// `self.sparse_len() + additional` if it returns `Ok(())`. Does nothing if the capacity is already sufficient.
	///
	/// Note that the allocator may give the collection more space than it requests. Therefore, capacity can not be relied
	/// upon to be precisely minimal. Prefer [`try_reserve_sparse`] if future insertions are expected.
	///
	/// [`try_reserve_sparse`]: SparseSet::try_reserve_sparse
	///
	/// # Errors
	///
	/// If the capacity overflows, or the allocator reports a failure, then an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use std::collections::TryReserveError;
	///
	/// # use sparse_set::SparseSet;
	///
	/// fn process_data(data: &[u32]) -> Result<SparseSet<usize, u32>, TryReserveError> {
	///   let mut output = SparseSet::new();
	///
	///   // Pre-reserve the memory, exiting if we can't.
	///   output.try_reserve_exact_sparse(data.len())?;
	///
	///   // Now we know this can't OOM in the middle of our complex work.
	///   for (index, value) in data.iter().cloned().enumerate() {
	///     output.insert(index, value);
	///   }
	///
	///   Ok(output)
	/// }
	/// # process_data(&[1, 2, 3]).expect("why is the test harness OOMing on 12 bytes?");
	/// ```
	pub fn try_reserve_exact_sparse(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.sparse.try_reserve_exact(additional)
	}

	/// Shrinks the dense capacity of the sparse set as much as possible.
	///
	/// It will drop down as close as possible to the length but the allocator may still inform the sparse set that
	/// there is space for a few more elements.
	///
	/// This operation is *O*(*n*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::with_capacity(10, 10);
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// assert_eq!(set.dense_capacity(), 10);
	///
	/// set.shrink_to_fit_dense();
	/// assert!(set.dense_capacity() >= 2);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn shrink_to_fit_dense(&mut self) {
		self.dense.shrink_to_fit();
	}

	/// Shrinks the sparse capacity of the sparse set as much as possible.
	///
	/// Unlike [`shrink_to_fit_dense`](Self::shrink_to_fit_dense), this can also
	/// reduce `sparse_len` as any empty indices after the maximum index are removed.
	///
	/// It will drop down as close as possible to the length but the allocator may still inform the sparse set that
	/// there is space for a few more elements.
	///
	/// This operation is *O*(*m*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::with_capacity(10, 10);
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// assert_eq!(set.sparse_capacity(), 10);
	///
	/// set.shrink_to_fit_dense();
	/// assert!(set.sparse_capacity() >= 2);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn shrink_to_fit_sparse(&mut self) {
		self.sparse.shrink_to_fit();
	}

	/// Shrinks the dense capacity of the sparse set with a lower bound.
	///
	/// The capacity will remain at least as large as both the length and the supplied value.
	///
	/// If the current capacity is less than the lower limit, this is a no-op.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::with_capacity(10, 10);
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// assert_eq!(set.dense_capacity(), 10);
	/// set.shrink_to_dense(4);
	/// assert!(set.dense_capacity() >= 4);
	/// set.shrink_to_dense(0);
	/// assert!(set.dense_capacity() >= 2);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn shrink_to_dense(&mut self, min_capacity: usize) {
		self.dense.shrink_to(min_capacity);
	}

	/// Shrinks the sparse capacity of the sparse set with a lower bound.
	///
	/// Unlike [`shrink_to_dense`](Self::shrink_to_dense), this can also reduce
	/// `sparse_len` as any empty indices after the maximum index are removed.
	///
	/// The capacity will remain at least as large as both the length and the supplied value.
	///
	/// If the current capacity is less than the lower limit, this is a no-op.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::with_capacity(10, 10);
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// assert_eq!(set.sparse_capacity(), 10);
	/// set.shrink_to_sparse(4);
	/// assert!(set.sparse_capacity() >= 4);
	/// set.shrink_to_sparse(0);
	/// assert!(set.sparse_capacity() >= 2);
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn shrink_to_sparse(&mut self, min_capacity: usize) {
		self.sparse.shrink_to(min_capacity);
	}

	/// Returns an iterator over the sparse set's values.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	///
	/// Consuming the iterator is an *O*(*n*) operation.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// let mut iterator = set.values();
	///
	/// assert!(set.values().eq(&[1, 2, 3]));
	/// ```
	pub fn values(&self) -> impl Iterator<Item = &T> + DoubleEndedIterator + ExactSizeIterator {
		self.dense.iter()
	}

	/// Returns an iterator that allows modifying each value.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	///
	/// Consuming the iterator is an *O*(*n*) operation.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// for elem in set.values_mut() {
	///     *elem += 2;
	/// }
	///
	/// assert!(set.values().eq(&[3, 4, 5]));
	/// ```
	pub fn values_mut(
		&mut self,
	) -> impl Iterator<Item = &mut T> + DoubleEndedIterator + ExactSizeIterator {
		self.dense.iter_mut()
	}
}

impl<I: SparseSetIndex, T> SparseSet<I, T> {
	/// Returns `true` if the sparse set contains an element at the given index.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	///
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// assert!(set.contains(0));
	/// assert!(!set.contains(100));
	/// ```
	#[must_use]
	pub fn contains(&self, index: I) -> bool {
		self.get(index).is_some()
	}

	/// Returns the raw `usize` index into the dense buffer from the given index.
	///
	/// This is meant to help with usecases of storing additional data outside of the sparse set in the same order.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	///  ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	///
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(20, 3);
	/// assert_eq!(Some(1), set.dense_index_of(1));
	/// assert_eq!(Some(2), set.dense_index_of(20));
	/// assert_eq!(None, set.dense_index_of(2));
	/// ```
	#[must_use]
	pub fn dense_index_of(&self, index: I) -> Option<usize> {
		self.sparse
			.get(index)
			.map(|dense_index| dense_index.get() - 1)
	}

	/// Gets the given index's corresponding entry in the sparse set for in-place manipulation.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.entry(1).or_insert(0);
	/// assert_eq!(set.get(1), Some(&0));
	/// ```
	#[must_use]
	pub fn entry(&mut self, index: I) -> Entry<'_, I, T> {
		match self.dense_index_of(index) {
			Some(dense_index) => Entry::Occupied(OccupiedEntry {
				dense_index,
				index,
				sparse_set: self,
			}),
			None => Entry::Vacant(VacantEntry {
				index,
				sparse_set: self,
			}),
		}
	}

	/// Returns a reference to an element pointed to by the index, if it exists.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	///
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	/// assert_eq!(Some(&2), set.get(1));
	/// assert_eq!(None, set.get(3));
	///
	/// set.remove(1);
	/// assert_eq!(None, set.get(1));
	/// ```
	#[must_use]
	pub fn get(&self, index: I) -> Option<&T> {
		self.dense_index_of(index)
			.map(|dense_index| unsafe { self.dense.get_unchecked(dense_index) })
	}

	/// Returns a reference to an element pointed to by the index, if it exists along with its index.
	///
	/// This is useful over [`SparseSet::get`] when the index type has additional information that can be used to
	/// distinguish it from other indices.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	///
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	/// assert_eq!(Some((1, &2)), set.get_with_index(1));
	/// assert_eq!(None, set.get_with_index(3));
	///
	/// set.remove(1);
	/// assert_eq!(None, set.get_with_index(1));
	/// ```
	#[must_use]
	pub fn get_with_index(&self, index: I) -> Option<(I, &T)> {
		self.dense_index_of(index).map(|dense_index| {
			(
				*unsafe { self.indices.get_unchecked(dense_index) },
				unsafe { self.dense.get_unchecked(dense_index) },
			)
		})
	}

	/// Returns a mutable reference to an element pointed to by the index, if it exists.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	///
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// if let Some(elem) = set.get_mut(1) {
	///   *elem = 42;
	/// }
	///
	/// assert!(set.values().eq(&[1, 42, 3]));
	/// ```
	#[must_use]
	pub fn get_mut(&mut self, index: I) -> Option<&mut T> {
		self.dense_index_of(index)
			.map(|dense_index| unsafe { self.dense.get_unchecked_mut(dense_index) })
	}

	/// Returns a mutable reference to an element pointed to by the index,, if it exists along with its index.
	///
	/// This is useful over [`SparseSet::get_mut`] when the index type has additional information that can be used to
	/// distinguish it from other indices.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	///
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// if let Some((index, elem)) = set.get_mut_with_index(1) {
	///   *elem = 42;
	/// }
	///
	/// assert!(set.values().eq(&[1, 42, 3]));
	/// ```
	#[must_use]
	pub fn get_mut_with_index(&mut self, index: I) -> Option<(I, &mut T)> {
		self.dense_index_of(index).map(|dense_index| {
			(
				*unsafe { self.indices.get_unchecked_mut(dense_index) },
				unsafe { self.dense.get_unchecked_mut(dense_index) },
			)
		})
	}

	/// Returns an iterator over the sparse set's indices.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	///
	/// Consuming the iterator is an *O*(*n*) operation.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// let mut iterator = set.indices();
	///
	/// assert_eq!(iterator.next(), Some(0));
	/// assert_eq!(iterator.next(), Some(1));
	/// assert_eq!(iterator.next(), Some(2));
	/// assert_eq!(iterator.next(), None);
	/// ```
	pub fn indices(
		&self,
	) -> impl Iterator<Item = I> + DoubleEndedIterator + ExactSizeIterator + '_ {
		self.indices.iter().cloned()
	}

	/// Returns an iterator over the sparse set's indices and values as pairs.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	///
	/// Consuming the iterator is an *O*(*n*) operation.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// let mut iterator = set.values();
	///
	/// assert!(set.iter().eq([(0, &1), (1, &2), (2, &3)]));
	/// ```
	pub fn iter(&self) -> impl Iterator<Item = (I, &T)> + DoubleEndedIterator + ExactSizeIterator {
		self.indices.iter().cloned().zip(self.dense.iter())
	}

	/// Returns an iterator that allows modifying each value as an `(index, value)` pair.
	///
	/// Do not rely on the order being consistent across insertions and removals.
	///
	/// Consuming the iterator is an *O*(*n*) operation.
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// assert!(set.iter_mut().eq([(0, &mut 1), (1, &mut 2), (2, &mut 3)]));
	/// ```
	pub fn iter_mut(
		&mut self,
	) -> impl Iterator<Item = (I, &mut T)> + DoubleEndedIterator + ExactSizeIterator {
		self.indices.iter().cloned().zip(self.dense.iter_mut())
	}

	/// Inserts an element at position `index` within the sparse set.
	///
	/// If a value already existed at `index`, it will be replaced and returned. The corresponding index will also be
	/// replaced with the given index allowing indices to store additional information outside of their indexing behavior.
	///
	/// If `index` is greater than `sparse_capacity`, then an allocation will take place.
	///
	/// This operation is amortized *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	///
	/// set.insert(0, 1);
	/// set.insert(1, 4);
	/// set.insert(2, 2);
	/// set.insert(3, 3);
	///
	/// assert!(set.values().eq(&[1, 4, 2, 3]));
	/// set.insert(20, 5);
	/// assert!(set.values().eq(&[1, 4, 2, 3, 5]));
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn insert(&mut self, index: I, value: T) -> Option<T> {
		self.insert_with_index(index, value).map(|(_, value)| value)
	}

	/// Inserts an element at position `index` within the sparse set.
	///
	/// If a value already existed at `index`, it will be replaced and returned. The corresponding index will also be
	/// replaced with the given index allowing indices to store additional information outside of their indexing behavior.
	///
	/// If `index` is greater than `sparse_capacity`, then an allocation will take place.
	///
	/// This is useful over [`SparseSet::insert`] when the index type has additional information that can be used to
	/// distinguish it from other indices.
	///
	/// This operation is amortized *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	///
	/// set.insert_with_index(0, 1);
	/// set.insert_with_index(1, 4);
	/// set.insert_with_index(2, 2);
	/// set.insert_with_index(3, 3);
	///
	/// assert!(set.values().eq(&[1, 4, 2, 3]));
	/// set.insert_with_index(20, 5);
	/// assert!(set.values().eq(&[1, 4, 2, 3, 5]));
	/// ```
	#[cfg(not(no_global_oom_handling))]
	pub fn insert_with_index(&mut self, mut index: I, mut value: T) -> Option<(I, T)> {
		match self.dense_index_of(index) {
			Some(dense_index) => {
				mem::swap(&mut index, unsafe {
					self.indices.get_unchecked_mut(dense_index)
				});
				mem::swap(&mut value, unsafe {
					self.dense.get_unchecked_mut(dense_index)
				});
				Some((index, value))
			}
			None => {
				self.dense.push(value);
				self.indices.push(index);
				let _ = self.sparse.insert(index, unsafe {
					NonZeroUsize::new_unchecked(self.dense_len())
				});
				None
			}
		}
	}

	/// Removes and returns the element at position `index` within the sparse set, if it exists.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// assert_eq!(set.remove(1), Some(2));
	/// assert!(set.values().eq(&[1, 3]));
	/// ```
	#[must_use]
	pub fn remove(&mut self, index: I) -> Option<T> {
		self.remove_with_index(index).map(|(_, value)| value)
	}

	/// Removes and returns the element at position `index` within the sparse set, if it exists along with its index.
	///
	/// This is useful over [`SparseSet::remove`] when the index type has additional information that can be used to
	/// distinguish it from other indices.
	///
	/// This operation is *O*(*1*).
	///
	/// # Examples
	///
	/// ```
	/// # use sparse_set::SparseSet;
	/// #
	/// let mut set = SparseSet::new();
	/// set.insert(0, 1);
	/// set.insert(1, 2);
	/// set.insert(2, 3);
	///
	/// assert_eq!(set.remove_with_index(1), Some((1, 2)));
	/// assert!(set.values().eq(&[1, 3]));
	/// ```
	#[must_use]
	pub fn remove_with_index(&mut self, index: I) -> Option<(I, T)> {
		self.sparse
			.remove(index)
			.map(|dense_index| unsafe { self.remove_at_dense_index(dense_index.get() - 1) })
	}

	unsafe fn remove_at_dense_index(&mut self, dense_index: usize) -> (I, T) {
		let index = self.indices.swap_remove(dense_index);
		let value = self.dense.swap_remove(dense_index);

		if dense_index != self.dense.len() {
			let swapped_index: usize = (*unsafe { self.indices.get_unchecked(dense_index) }).into();
			*unsafe { self.sparse.get_unchecked_mut(swapped_index) } =
				Some(unsafe { NonZeroUsize::new_unchecked(dense_index + 1) });
		}

		(index, value)
	}
}

impl<I, T> AsRef<Self> for SparseSet<I, T> {
	fn as_ref(&self) -> &Self {
		self
	}
}

impl<I, T> AsMut<Self> for SparseSet<I, T> {
	fn as_mut(&mut self) -> &mut Self {
		self
	}
}

impl<I, T> AsRef<[T]> for SparseSet<I, T> {
	fn as_ref(&self) -> &[T] {
		&self.dense
	}
}

impl<I, T> AsMut<[T]> for SparseSet<I, T> {
	fn as_mut(&mut self) -> &mut [T] {
		&mut self.dense
	}
}

impl<I, T> Default for SparseSet<I, T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<I, T> Deref for SparseSet<I, T> {
	type Target = [T];

	fn deref(&self) -> &[T] {
		&self.dense
	}
}

impl<I, T> DerefMut for SparseSet<I, T> {
	fn deref_mut(&mut self) -> &mut [T] {
		&mut self.dense
	}
}

impl<I: Debug + SparseSetIndex, T: Debug> Debug for SparseSet<I, T> {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		formatter.debug_map().entries(self.iter()).finish()
	}
}

#[cfg(not(no_global_oom_handling))]
impl<'a, I: SparseSetIndex, T: Copy + 'a> Extend<(I, &'a T)> for SparseSet<I, T> {
	fn extend<Iter: IntoIterator<Item = (I, &'a T)>>(&mut self, iter: Iter) {
		for (index, &value) in iter {
			let _ = self.insert(index, value);
		}
	}
}

#[cfg(not(no_global_oom_handling))]
impl<I: SparseSetIndex, T> Extend<(I, T)> for SparseSet<I, T> {
	fn extend<Iter: IntoIterator<Item = (I, T)>>(&mut self, iter: Iter) {
		for (index, value) in iter {
			let _ = self.insert(index, value);
		}
	}
}

#[cfg(not(no_global_oom_handling))]
impl<I: SparseSetIndex, T, const N: usize> From<[(I, T); N]> for SparseSet<I, T> {
	fn from(slice: [(I, T); N]) -> Self {
		let mut set = Self::with_capacity(slice.len(), slice.len());

		for (index, value) in slice {
			let _ = set.insert(index, value);
		}

		set
	}
}

#[cfg(not(no_global_oom_handling))]
impl<I: SparseSetIndex, T> FromIterator<(I, T)> for SparseSet<I, T> {
	fn from_iter<Iter: IntoIterator<Item = (I, T)>>(iter: Iter) -> Self {
		let iter = iter.into_iter();
		let capacity = iter
			.size_hint()
			.1
			.map_or_else(|| iter.size_hint().0, |size_hint| size_hint);
		let mut set = Self::with_capacity(capacity, capacity);

		for (index, value) in iter {
			let _ = set.insert(index, value);
		}

		set
	}
}

impl<I: Hash + SparseSetIndex, T: Hash> Hash for SparseSet<I, T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		for index in self.sparse.iter().flatten() {
			unsafe { self.sparse.get_unchecked(index.get() - 1) }.hash(state);
			unsafe { self.indices.get_unchecked(index.get() - 1) }.hash(state);
			unsafe { self.dense.get_unchecked(index.get() - 1) }.hash(state);
		}
	}
}

impl<I: SparseSetIndex, T> Index<I> for SparseSet<I, T> {
	type Output = T;

	fn index(&self, index: I) -> &Self::Output {
		self.get(index).unwrap()
	}
}

impl<I: SparseSetIndex, T> IndexMut<I> for SparseSet<I, T> {
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		self.get_mut(index).unwrap()
	}
}

impl<I: PartialEq + SparseSetIndex, T: PartialEq> PartialEq for SparseSet<I, T> {
	fn eq(&self, other: &Self) -> bool {
		if self.indices.len() != other.indices.len() {
			return false;
		}

		for index in &self.indices {
			match (self.dense_index_of(*index), other.dense_index_of(*index)) {
				(Some(index), Some(other_index)) => {
					if unsafe { self.indices.get_unchecked(index) }
						!= unsafe { other.indices.get_unchecked(other_index) }
					{
						return false;
					}

					if unsafe { self.dense.get_unchecked(index) }
						!= unsafe { other.dense.get_unchecked(other_index) }
					{
						return false;
					}
				}
				(None, None) => {}
				_ => {
					return false;
				}
			}
		}

		true
	}
}

impl<I: Eq + SparseSetIndex, T: Eq> Eq for SparseSet<I, T> {}

/// A view into a single entry in a sparse set, which may be either vacant or occupied.
///
/// This is constructed from the [`SparseSet::entry`] function.
pub enum Entry<'a, I, T> {
	/// A vacant entry.
	Vacant(VacantEntry<'a, I, T>),

	/// An occupied entry.
	Occupied(OccupiedEntry<'a, I, T>),
}

impl<'a, I: SparseSetIndex, T> Entry<'a, I, T> {
	/// Provides in-place mutable access to an occupied entry before any potential inserts into the sparse set.
	#[must_use]
	pub fn and_modify<F: FnOnce(&mut T)>(self, function: F) -> Self {
		match self {
			Entry::Vacant(entry) => Entry::Vacant(entry),
			Entry::Occupied(mut entry) => {
				function(entry.get_mut());
				Entry::Occupied(entry)
			}
		}
	}

	/// The index used to create this entry.
	#[must_use]
	pub fn entry_index(&self) -> I {
		match self {
			Entry::Vacant(entry) => entry.entry_index(),
			Entry::Occupied(entry) => entry.entry_index(),
		}
	}

	/// Ensures a value is in the entry by inserting the default if empty, and returns a mutable reference to the value in
	/// the entry.
	pub fn or_insert(self, default: T) -> &'a mut T {
		match self {
			Entry::Vacant(entry) => entry.insert(default),
			Entry::Occupied(entry) => entry.into_mut(),
		}
	}

	/// Ensures a value is in the entry by inserting the result of the default function if empty, and returns a mutable
	/// reference to the value in the entry.
	pub fn or_insert_with<F: FnOnce() -> T>(self, default: F) -> &'a mut T {
		match self {
			Entry::Vacant(entry) => entry.insert(default()),
			Entry::Occupied(entry) => entry.into_mut(),
		}
	}

	/// Sets the value of the entry, and returns an [`OccupiedEntry`].
	pub fn insert_entry(self, value: T) -> OccupiedEntry<'a, I, T> {
		match self {
			Entry::Vacant(entry) => entry.insert_entry(value),
			Entry::Occupied(mut entry) => {
				let _ = entry.insert(value);
				entry
			}
		}
	}
}

impl<I: Debug + SparseSetIndex, T: Debug> Debug for Entry<'_, I, T> {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Entry::Vacant(entry) => formatter.debug_tuple("Entry").field(entry).finish(),
			Entry::Occupied(entry) => formatter.debug_tuple("Entry").field(entry).finish(),
		}
	}
}

/// A view into a vacant entry in a sparse set.
pub struct VacantEntry<'a, I, T> {
	/// The index this entry was created from.
	index: I,

	/// A reference to the sparse set this entry was created for.
	sparse_set: &'a mut SparseSet<I, T>,
}

impl<'a, I: SparseSetIndex, T> VacantEntry<'a, I, T> {
	/// The index used to create this entry.
	#[must_use]
	pub fn entry_index(&self) -> I {
		self.index
	}

	/// Inserts the given value into this entry, returning a mutable reference to it.
	pub fn insert(mut self, value: T) -> &'a mut T {
		let dense_index = self.insert_raw(value);
		unsafe { self.sparse_set.dense.get_unchecked_mut(dense_index) }
	}

	/// Inserts the given value into this entry, returning an occupied entry.
	pub fn insert_entry(mut self, value: T) -> OccupiedEntry<'a, I, T> {
		let dense_index = self.insert_raw(value);
		OccupiedEntry {
			dense_index,
			index: self.index,
			sparse_set: self.sparse_set,
		}
	}

	/// Inserts the given value into this entry without consuming it.
	#[must_use]
	fn insert_raw(&mut self, value: T) -> usize {
		self.sparse_set.dense.push(value);
		self.sparse_set.indices.push(self.index);
		let _ = self.sparse_set.sparse.insert(self.index, unsafe {
			NonZeroUsize::new_unchecked(self.sparse_set.dense_len())
		});
		self.sparse_set.dense_len() - 1
	}
}

impl<I: Debug, T> Debug for VacantEntry<'_, I, T> {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		formatter
			.debug_tuple("VacantEntry")
			.field(&self.index)
			.finish()
	}
}

/// A view into an occupied entry in a sparse set.
pub struct OccupiedEntry<'a, I, T> {
	/// The raw `usize` index into the dense buffer for this entry.
	dense_index: usize,

	/// The index this entry was created from.
	index: I,

	/// A reference to the sparse set this entry was created for.
	sparse_set: &'a mut SparseSet<I, T>,
}

impl<'a, I: SparseSetIndex, T> OccupiedEntry<'a, I, T> {
	/// Returns the raw `usize` index into the dense buffer for this entry.
	#[must_use]
	pub fn dense_index(&self) -> usize {
		self.dense_index
	}

	/// Returns an immutable reference to the value for this entry.
	#[must_use]
	pub fn get(&self) -> &T {
		unsafe { self.sparse_set.dense.get_unchecked(self.dense_index) }
	}

	/// Returns an mutable reference to the value for this entry.
	#[must_use]
	pub fn get_mut(&mut self) -> &mut T {
		unsafe { self.sparse_set.dense.get_unchecked_mut(self.dense_index) }
	}

	/// Consumes the entry, returning a reference to the entry's value tied to the lifetime of the sparse set.
	#[must_use]
	pub fn into_mut(self) -> &'a mut T {
		unsafe { self.sparse_set.dense.get_unchecked_mut(self.dense_index) }
	}

	/// The index used to create this entry.
	///
	/// This index may be different from the one currently stored (see [`OccupiedEntry::stored_index`]), but both will
	/// have the same behavior with respect to [`SparseSetIndex`].
	#[must_use]
	pub fn entry_index(&self) -> I {
		self.index
	}

	/// The index stored for this index..
	///
	/// This index may be different from the one used to create this entry (see [`OccupiedEntry::entry_index`]), but both
	/// will have the same behavior with respect to [`SparseSetIndex`].
	#[must_use]
	pub fn stored_index(&self) -> I {
		*unsafe { self.sparse_set.indices.get_unchecked(self.dense_index) }
	}

	/// Inserts the given value into this entry, returning the existing value.
	pub fn insert(&mut self, value: T) -> T {
		self.insert_with_index(value).1
	}

	/// Inserts the given value into this entry, returning the existing value and its index.
	pub fn insert_with_index(&mut self, mut value: T) -> (I, T) {
		let mut index = self.index;
		mem::swap(&mut index, unsafe {
			self.sparse_set.indices.get_unchecked_mut(self.dense_index)
		});
		mem::swap(&mut value, unsafe {
			self.sparse_set.dense.get_unchecked_mut(self.dense_index)
		});
		(index, value)
	}

	/// Removes and returns the value associated with this entry, consuming it.
	pub fn remove(self) -> T {
		self.remove_with_index().1
	}

	/// Removes and returns the value associated with this entry along with its index, consuming it.
	pub fn remove_with_index(self) -> (I, T) {
		let _ = self.sparse_set.sparse.remove(self.index);
		unsafe { self.sparse_set.remove_at_dense_index(self.dense_index) }
	}
}

impl<I: Debug + SparseSetIndex, T: Debug> Debug for OccupiedEntry<'_, I, T> {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		formatter
			.debug_struct("OccupiedEntry")
			.field("index", &self.entry_index())
			.field("value", self.get())
			.finish()
	}
}
