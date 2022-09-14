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

use crate::vfs::ZsProxyFs;

// use ::zsparse::filesystem::File as ZsFile;
// use ::zsparse::filesystem::FileSystem as ZsFileSystem;
use ::zsparse::filesystem::Files as ZsFiles;
use zsparse::err::ParsingError as ZsParsingError;
// use zsparse::err::ParsingErrorLevel as ZsParsingErrorLevel;
use zsparse::parser_manager::parse_filesystem as zs_parse_filesystem;
use zsparse::parser_manager::FileIndexAndAst as ZsFileIndexAndAst;

pub struct ParseOutput {
	pub files: ZsFiles,
	pub errors: Vec<ZsParsingError>,
	pub asts: Vec<ZsFileIndexAndAst>
}

pub fn parse(vfs: ZsProxyFs) -> ParseOutput {
	let mut errors = Default::default();
	let mut files = Default::default();
	let asts = zs_parse_filesystem(vfs, &mut files, &mut errors).asts;

	ParseOutput {
		files,
		errors,
		asts
	}
}
