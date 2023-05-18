//! VZScript's scannerless, combinator-based parser.

mod common;
mod expr;
mod lit;

#[cfg(test)]
mod test;

use crossbeam::queue::SegQueue;
use doomfront::{
	chumsky::{IterParser, Parser},
	rowan::GreenNode,
	util::{
		builder::{GreenCache, GreenCacheNoop},
		state::ParseState,
	},
};
use parking_lot::Mutex;
use rayon::prelude::*;

use crate::{data::vfs::FileRef, VPath, VPathBuf};

use super::Syn;

pub type Error<'i> = doomfront::ParseError<'i>;

/// Parses with the opportunity for recovery. Unless `source` has no tokens
/// whatsoever, this always emits `Some`, although the returned tree may have
/// errors attached.
///
/// When faced with unexpected input, the parser raises an error and then tries
/// to skip ahead to the next top-level item, carriage return, or newline.
/// All input between the error location and the next valid thing gets wrapped
/// into a token tagged [`Syn::Unknown`].
#[must_use]
pub fn parse_file<'i, C: 'i + GreenCache>(
	source: &'i str,
	path: impl AsRef<VPath>,
	cache: Option<C>,
) -> Option<FileParseTree> {
	let parser = doomfront::chumsky::primitive::choice((common::wsp_ext(), common::annotation()))
		.repeated()
		.collect::<()>();

	let mut state = ParseState::new(cache);

	state.gtb.open(Syn::Root.into());

	let (output, errors) = parser
		.parse_with_state(source, &mut state)
		.into_output_errors();

	output.map(|_| {
		state.gtb.close();

		FileParseTree {
			inner: ParseTree {
				root: state.gtb.finish(),
				errors: errors.into_iter().map(|err| err.into_owned()).collect(),
			},
			path: path.as_ref().to_path_buf(),
		}
	})
}

#[must_use]
pub fn parse_repl<C: GreenCache>(source: &str, cache: Option<C>) -> Option<ParseTree> {
	let parser = doomfront::chumsky::primitive::choice((common::wsp_ext(), expr::expr()));

	let mut state = ParseState::new(cache);

	state.gtb.open(Syn::Root.into());

	let (output, errors) = parser
		.parse_with_state(source, &mut state)
		.into_output_errors();

	output.map(|_| {
		state.gtb.close();

		ParseTree {
			root: state.gtb.finish(),
			errors,
		}
	})
}

#[derive(Debug)]
pub struct ParseTree<'i> {
	pub root: GreenNode,
	pub errors: Vec<Error<'i>>,
}

#[derive(Debug)]
pub struct FileParseTree {
	pub inner: ParseTree<'static>,
	pub path: VPathBuf,
}

impl std::ops::Deref for FileParseTree {
	type Target = ParseTree<'static>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Default)]
pub struct IncludeTree {
	files: Vec<FileParseTree>,
}

impl IncludeTree {
	/// Traverses the virtual directory `root` recursively, parsing all text files
	/// extended with `.vzs`.
	///
	/// Panics if `root` is not a directory.
	/// Mind that it is entirely valid for the returned include tree to be empty.
	#[must_use]
	pub fn new(root: FileRef) -> Self {
		assert!(
			root.is_dir(),
			"Tried to build an include tree from non-directory root: {}",
			root.path().display()
		);

		let dir_q = SegQueue::<VPathBuf>::default();
		let all_files = Mutex::new(vec![]);

		for child in root.child_paths().unwrap() {
			dir_q.push(child.to_path_buf());
		}

		while let Some(dir_path) = dir_q.pop() {
			let dir = root.vfs().get(&dir_path).unwrap();

			dir.child_paths()
				.unwrap()
				.par_bridge()
				.for_each(|child_path| {
					if child_path.is_dir() {
						dir_q.push(child_path.to_path_buf());
						return;
					}

					if child_path
						.extension()
						.filter(|ext| ext.eq_ignore_ascii_case("vzs"))
						.is_none()
					{
						return;
					}

					let child = dir.vfs().get(child_path).unwrap();

					if !child.is_text() {
						return;
					}

					if let Some(ptree) =
						parse_file::<GreenCacheNoop>(child.read_str(), child_path, None)
					{
						all_files.lock().push(ptree);
					}
				});
		}

		IncludeTree {
			files: all_files.into_inner(),
		}
	}

	#[must_use]
	pub fn files(&self) -> &[FileParseTree] {
		&self.files
	}

	#[must_use]
	pub fn into_inner(self) -> Vec<FileParseTree> {
		self.files
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		self.files.iter().any(|ptree| !ptree.errors.is_empty())
	}
}
