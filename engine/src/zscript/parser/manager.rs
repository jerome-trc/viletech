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

use rayon::prelude::*;

use super::{
	ast::top::{TopLevel, TopLevelDefinitionKind},
	core::Parser,
	error::ParsingError,
	fs::{FileIndex, FileSystem, Files},
};

#[derive(serde::Serialize, Debug)]
pub struct FileIndexAndAst {
	pub file: FileIndex,
	pub ast: TopLevel,
}

#[derive(serde::Serialize, Debug)]
pub struct FileSystemParseResult {
	pub asts: Vec<FileIndexAndAst>,
}

pub struct ParseFileSystemConfig<'a> {
	pub root_name: &'a str,
}

impl<'a> Default for ParseFileSystemConfig<'a> {
	fn default() -> Self {
		Self {
			root_name: "zscript",
		}
	}
}

pub fn parse_filesystem<F: FileSystem>(
	filesystem: F,
	files: &mut Files,
	errs: &mut Vec<ParsingError>,
) -> FileSystemParseResult {
	parse_filesystem_config(filesystem, files, errs, &ParseFileSystemConfig::default())
}

pub fn parse_filesystem_config<F: FileSystem>(
	mut filesystem: F,
	files: &mut Files,
	errs: &mut Vec<ParsingError>,
	config: &ParseFileSystemConfig,
) -> FileSystemParseResult {
	let mut ret = FileSystemParseResult { asts: vec![] };
	let root_scripts = filesystem.get_files_no_ext(config.root_name);
	let mut needed_files = vec![];
	for r in root_scripts {
		let f = files.add(r);
		needed_files.push(f);
	}
	while !needed_files.is_empty() {
		let iter = needed_files.par_iter();
		let res: Vec<_> = iter
			.map(|&f| Parser::new(f, files[f].text()).parse())
			.collect();

		needed_files.clear();
		for r in res {
			for d in r.ast.definitions.iter() {
				if let TopLevelDefinitionKind::Include(s) = &d.kind {
					let file = {
						let filename = s.symbol.string();
						filesystem.get_file(&filename).unwrap()
					};
					let f = files.add(file);
					needed_files.push(f);
				}
			}
			for e in r.errs {
				errs.push(e);
			}
			ret.asts.push(FileIndexAndAst {
				file: r.file,
				ast: r.ast,
			});
		}
	}
	ret
}
