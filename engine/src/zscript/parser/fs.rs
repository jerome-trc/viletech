/*

Copyright (C) 2021-2022 Jessica "Gutawer" Russell

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

use std::ops::Range;

use serde::Serialize;

use super::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct File {
	pub(super) filename: String,
	pub(super) data: Vec<u8>,
	pub(super) text: String,
	pub(super) lines: Vec<Range<usize>>,
}

impl File {
	pub fn new(filename: String, data: Vec<u8>) -> Self {
		let text = String::from_utf8_lossy(&data).to_string();
		let lines = super::get_lines(&text);
		Self {
			filename,
			data,
			text,
			lines,
		}
	}

	pub fn filename(&self) -> &str {
		&self.filename
	}

	pub fn data(&self) -> &[u8] {
		&self.data
	}

	pub fn text(&self) -> &str {
		&self.text
	}
}

#[derive(Serialize, Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileIndex(pub(super) usize);

#[derive(Debug, Default)]
pub struct Files(Vec<File>);

impl std::ops::Index<FileIndex> for Files {
	type Output = File;

	fn index(&self, index: FileIndex) -> &Self::Output {
		&self.0[index.0]
	}
}

impl Files {
	pub fn add(&mut self, file: File) -> FileIndex {
		self.0.push(file);
		FileIndex(self.0.len() - 1)
	}

	pub fn text_from_span(&self, span: Span) -> &str {
		let file = &self[span.file];
		&file.text()[span.start..span.end]
	}
}

pub trait FileSystem {
	fn get_file(&mut self, filename: &str) -> Option<File>;
	fn get_files_no_ext(&mut self, filename: &str) -> Vec<File>;
}
