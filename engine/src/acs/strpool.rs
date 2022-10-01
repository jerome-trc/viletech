/*
Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::collections::HashMap;

use fasthash::metro;

const NUM_BUCKETS: usize = 251;

/// [GZ]:
/// Programmatically generated strings (e.g. those returned by strparam) are stored here.
/// PCD_TAGSTRING also now stores strings in this table instead of simply
/// tagging strings with their library ID.
///
/// Identical strings map to identical string identifiers.
///
/// When the string table needs to grow to hold more strings, a garbage
/// collection is first attempted to see if more room can be made to store
/// strings without growing. A string is considered in use if any value
/// in any of these variable blocks contains a valid ID in the global string
/// table:
///   * The active area of the ACS stack
///   * All running scripts' local variables
///   * All map variables
///   * All world variables
///   * All global variables
/// It's not important whether or not they are really used as strings, only
/// that they might be. A string is also considered in use if its lock count
/// is non-zero, even if none of the above variable blocks referenced it.
///
/// To keep track of local and map variables for nonresident maps in a hub,
/// when a map's state is archived, all strings found in its local and map
/// variables are locked. When a map is revisited in a hub, all strings found
/// in its local and map variables are unlocked. Locking and unlocking are
/// cumulative operations.
///
/// What this all means is that:
///   * Strings returned by strparam last indefinitely. No longer do they
///     disappear at the end of the tic they were generated.
///   * You can pass library strings around freely without having to worry
///     about always having the same libraries loaded in the same order on
///     every map that needs to use those strings.
pub(super) struct StringPool {
	pool: Vec<Entry>,
	buckets: [usize; NUM_BUCKETS],
	first_free_entry: usize,
}

#[derive(Default)]
struct Entry {
	string: String,
	hash: usize,
	next: usize,
	marked: bool,
	locks: Vec<u16>,
}

impl Entry {
	fn lock(&mut self, level_num: u16) {
		match self.locks.iter().find(|l| **l == level_num) {
			Some(_) => {}
			None => {
				self.locks.push(level_num);
			}
		}
	}

	fn unlock(&mut self, level_num: u16) {
		if let Some(pos) = self.locks.iter().position(|l| *l == level_num) {
			self.locks.swap_remove(pos);
		}
	}
}

impl Default for StringPool {
	fn default() -> Self {
		StringPool {
			pool: Default::default(),
			buckets: [255usize; 251],
			first_free_entry: 0,
		}
	}
}

impl StringPool {
	const FREE_ENTRY: usize = usize::MAX - 1;
	const NO_ENTRY: usize = usize::MAX;
	const MIN_GC_SIZE: usize = 100;

	const LIBID_MASK: usize = 0xFFF00000;
	const LIBID_SHIFT: usize = 20;

	const LIBID: usize = (i32::MAX as usize) >> Self::LIBID_SHIFT;
	const LIBID_OR: usize = Self::LIBID << Self::LIBID_SHIFT;

	fn clear(&mut self) {
		self.pool.clear();
		self.buckets = [255usize; 251];
		self.first_free_entry = 0;
	}

	fn add(&mut self, string: &str) -> Option<usize> {
		let hash = metro::hash64(string) as usize;
		let bucket = hash % NUM_BUCKETS;

		match self.find(string, hash, bucket) {
			Some(i) => Some(i | Self::LIBID_OR),
			None => self.insert(string, hash, bucket),
		}
	}

	fn get(&self, mut index: usize) -> Option<&str> {
		debug_assert_eq!(index & Self::LIBID_MASK, Self::LIBID_OR);

		index &= !Self::LIBID_MASK;

		if index >= self.pool.len() {
			return None;
		}

		if self.pool[index].next == Self::FREE_ENTRY {
			return None;
		}

		Some(&self.pool[index].string)
	}

	fn insert(&mut self, string: &str, hash: usize, bucket: usize) -> Option<usize> {
		let mut index = self.first_free_entry;

		if index >= Self::MIN_GC_SIZE && index == self.pool.capacity() {
			// Array needs to grow. Try a GC first
			self.gc_all();
			index = self.first_free_entry;
		}

		if self.first_free_entry >= Self::LIBID_OR {
			// Any higher will collide with the library ID marker
			return None;
		}

		if index == self.pool.len() {
			self.pool
				.resize_with(self.pool.len() + Self::MIN_GC_SIZE, Entry::default);

			self.first_free_entry += 1;
		} else {
			self.first_free_entry = self.find_first_free_entry(self.first_free_entry + 1);
		}

		let entry = &mut self.pool[index];
		entry.string = string.to_owned();
		entry.hash = hash;
		entry.next = self.buckets[bucket];
		entry.marked = false;
		entry.locks.clear();

		self.buckets[bucket] = index;

		Some(index | Self::LIBID_OR)
	}

	fn find(&self, string: &str, hash: usize, bucket: usize) -> Option<usize> {
		let mut i = self.buckets[bucket];

		while i != Self::NO_ENTRY {
			let entry = &self.pool[i];

			debug_assert!(entry.next != Self::FREE_ENTRY);

			if entry.hash == hash && entry.string == string {
				return Some(i);
			}

			i = entry.next;
		}

		None
	}

	fn find_first_free_entry(&self, mut base: usize) -> usize {
		while base < self.pool.len() && self.pool[base].next != Self::FREE_ENTRY {
			base += 1;
		}

		base
	}

	fn purge(&mut self) {
		// Clear hash buckets; rebuild them while choosing
		// which strings to keep and which will be purged
		self.buckets = [255usize; NUM_BUCKETS];

		for (i, entry) in self.pool.iter_mut().enumerate() {
			if entry.next == Self::FREE_ENTRY {
				continue;
			}

			if entry.locks.is_empty() && !entry.marked {
				entry.next = Self::FREE_ENTRY;

				if i < self.first_free_entry {
					self.first_free_entry = i;
				}

				entry.string = String::default();
			} else {
				let bytes = bytemuck::bytes_of(&entry.hash);
				let hash = metro::hash64(bytes) as usize;
				let hash = hash % NUM_BUCKETS;
				entry.next = self.buckets[hash];
				self.buckets[hash] = i;
				entry.marked = false;
			}
		}
	}

	fn lock(&mut self, str_num: usize, level_num: u16) {
		let index = self.pool_index(str_num);
		self.pool[index].lock(level_num);
	}

	fn lock_array(&mut self, level_num: u16, str_nums: Vec<usize>) {
		for str_num in str_nums {
			if (str_num & Self::LIBID_MASK) != Self::LIBID_OR {
				continue;
			}

			let index = str_num & !Self::LIBID_MASK;

			if index >= self.pool.len() {
				continue;
			}

			self.pool[index].lock(level_num);
		}
	}

	fn unlock_all(&mut self) {
		for entry in &mut self.pool {
			entry.marked = false;
			entry.locks.clear();
		}
	}

	fn unlock_for_level(&mut self, level_num: u16) {
		for entry in &mut self.pool {
			if entry.next == Self::FREE_ENTRY {
				continue;
			}

			match entry.locks.iter().position(|l| *l == level_num) {
				None => {}
				Some(ndx) => {
					entry.locks.swap_remove(ndx);
				}
			}
		}
	}

	fn gc_mark(&mut self, str_num: usize) {
		let index = self.pool_index(str_num);
		self.pool[index].marked = true;
	}

	fn gc_mark_array(&mut self, str_nums: Vec<usize>) {
		for str_num in str_nums {
			if (str_num & Self::LIBID_MASK) != Self::LIBID_OR {
				continue;
			}

			let index = str_num & !Self::LIBID_MASK;

			if index >= self.pool.len() {
				continue;
			}

			self.pool[index].marked = true;
		}
	}

	fn gc_mark_map(&mut self, world_globals: HashMap<i32, i32>) {
		for (_, val) in world_globals {
			let str_num = val as usize;

			if (str_num & Self::LIBID_MASK) != Self::LIBID_OR {
				continue;
			}

			let index = str_num & !Self::LIBID_MASK;

			if index >= self.pool.len() {
				continue;
			}

			self.pool[index].marked |= true;
		}
	}

	fn gc_all(&mut self) {
		// TODO:
		// - Mark all string arrays in stack
		// - Mark all level var strings in each level's behaviors
		// - Mark world var strings
		// - Mark global var strings
		self.purge();
		todo!();
	}

	fn pool_index(&self, str_num: usize) -> usize {
		debug_assert_eq!(str_num & Self::LIBID_MASK, Self::LIBID_OR);
		let ret = str_num & !Self::LIBID_MASK;
		debug_assert!(ret < self.pool.len());
		ret
	}
}
