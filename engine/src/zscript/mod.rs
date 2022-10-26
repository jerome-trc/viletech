//! End-to-end ZScript-to-LuaJIT transpilation.

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

pub mod parser;

use parser::{
	error::ParsingError,
	fs::{File, FileSystem, Files},
	manager::{parse_filesystem, FileIndexAndAst},
};

use crate::utils::string::line_from_char_index;

pub struct ParseOutput {
	pub files: Files,
	pub errors: Vec<ParsingError>,
	pub asts: Vec<FileIndexAndAst>,
}

#[must_use]
pub fn parse(fs: impl FileSystem) -> ParseOutput {
	let mut errors = Default::default();
	let mut files = Default::default();
	let asts = parse_filesystem(fs, &mut files, &mut errors).asts;

	ParseOutput {
		files,
		errors,
		asts,
	}
}

pub fn prettify_error(namespace: &str, file: &File, error: &ParsingError) -> String {
	let start = error.main_spans[0].get_start();
	let end = error.main_spans[0].get_end();
	let (line, line_index) = line_from_char_index(file.text(), start).unwrap();
	let line = line.trim();
	let line_start = file.text().find(line).unwrap();

	let mut indicators = String::with_capacity(line.len());
	indicators.push('\t');

	for _ in line_start..start {
		indicators.push(' ');
	}

	for _ in 0..(end - start) {
		indicators.push('^');
	}

	format!(
		"/{}/{}:{}:{}\r\n\r\n\t{}\r\n{}\r\n\tDetails: {}.\r\n",
		namespace,
		file.filename(),
		line_index + 1,
		start - line_start,
		line,
		indicators,
		error.msg
	)
}