//! Gutawer's ZScript parser, customised for Impure to generalise over DECORATE
//! as well. Used to generate one of two kinds of AST to be converted to Regolith.

/*

Original code within provided under the MIT license:

////////////////////////////////////////////////////////////////////////////////

Copyright 2022 Jessica Russell

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in the
Software without restriction, including without limitation the rights to use,
copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the
Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

////////////////////////////////////////////////////////////////////////////////

Modifications made to the parser are covered by the GPL3 license, notice below.

////////////////////////////////////////////////////////////////////////////////

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

pub(super) mod ast;
mod core;
pub mod fs;
mod helper;
mod hir;
pub(super) mod interner;
pub(super) mod ir;
pub mod issue;
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
