//! Gutawer's ZScript parser, customised for Impure to generalise over DECORATE
//! as well. Used to generate one of two kinds of AST to be converted to Regolith.

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

mod ast;
mod core;
pub mod error;
pub mod fs;
mod helper;
mod hir;
mod interner;
mod ir;
pub(super) mod manager;
mod tokenizer;

use std::ops::Range;

use serde::Serialize;

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
	file: fs::FileIndex,
	start: usize,
	end: usize,
}

impl Span {
	#[inline(always)]
	fn combine(self, other: Span) -> Span {
		debug_assert!(self.file == other.file);

		Span {
			start: self.start.min(other.start),
			end: self.end.max(other.end),
			file: self.file,
		}
	}

	pub fn get_file(&self) -> fs::FileIndex {
		self.file
	}

	pub fn get_start(&self) -> usize {
		self.start
	}

	pub fn get_end(&self) -> usize {
		self.end
	}
}

fn get_lines(source: &str) -> Vec<Range<usize>> {
	let source_ptr = source.as_ptr() as usize;
	source
		.split('\n')
		.map(|l| {
			let l_ptr = l.as_ptr() as usize;
			let offset = l_ptr - source_ptr;
			offset..(offset + l.len())
		})
		.collect()
}
