use super::Entry;
use super::EntryId;
use super::Wad;
use super::WadSlice;

pub struct EntryIterator<'a> {
	index: usize,
	wad: &'a Wad,
}

impl<'a> EntryIterator<'a> {
	pub(super) fn new(wad: &Wad) -> EntryIterator {
		EntryIterator { index: 0, wad }
	}
}

impl<'a> Iterator for EntryIterator<'a> {
	type Item = Entry<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.wad.len() {
			self.index += 1;
			Some(unsafe {
				// This is safe because entry_unchecked only elides the bounds
				// check, and we do bounds checking in this function
				self.wad.entry_unchecked(self.index - 1).unwrap()
			})
		} else {
			None
		}
	}
}

pub struct IdIterator<'a> {
	index: usize,
	wad: &'a Wad,
}

impl<'a> IdIterator<'a> {
	pub(super) fn new(wad: &Wad) -> IdIterator {
		IdIterator { index: 0, wad }
	}
}

impl<'a> Iterator for IdIterator<'a> {
	type Item = EntryId;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.wad.len() {
			self.index += 1;
			Some(unsafe {
				// This is safe because entry_unchecked only elides the bounds
				// check, and we do bounds checking in this function
				self.wad.entry_id_unchecked(self.index - 1)
			})
		} else {
			None
		}
	}
}

pub struct SliceEntryIterator<'a> {
	index: usize,
	wad: &'a WadSlice<'a>,
}

impl<'a> SliceEntryIterator<'a> {
	pub(super) fn new<'b>(wad: &'b WadSlice) -> SliceEntryIterator<'b> {
		SliceEntryIterator { index: 0, wad }
	}
}

impl<'a> Iterator for SliceEntryIterator<'a> {
	type Item = Entry<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.wad.len() {
			self.index += 1;
			Some(unsafe {
				// This is safe because entry_unchecked only elides the bounds
				// check, and we do bounds checking in this function
				self.wad.entry_unchecked(self.index - 1).unwrap()
			})
		} else {
			None
		}
	}
}

pub struct SliceIdIterator<'a> {
	index: usize,
	wad: &'a WadSlice<'a>,
}

impl<'a> SliceIdIterator<'a> {
	pub(super) fn new<'b>(wad: &'b WadSlice) -> SliceIdIterator<'b> {
		SliceIdIterator { index: 0, wad }
	}
}

impl<'a> Iterator for SliceIdIterator<'a> {
	type Item = EntryId;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.wad.len() {
			self.index += 1;
			Some(unsafe {
				// This is safe because entry_unchecked only elides the bounds
				// check, and we do bounds checking in this function
				self.wad.entry_id_unchecked(self.index - 1)
			})
		} else {
			None
		}
	}
}
