//! Combinator-based parsers.

mod common;
mod expr;
mod lit;
#[cfg(test)]
mod test;

use crossbeam::queue::SegQueue;
use doomfront::{
	chumsky::{primitive, Parser},
	comb,
	ext::{Parser1, ParserOpt, ParserVec},
	rowan::GreenNode,
	ParseError, ParseOut,
};
use parking_lot::Mutex;
use rayon::prelude::*;

use crate::{data::vfs::FileRef, VPath, VPathBuf};

use super::Syn;

use self::{common::*, expr::*};

/// Parses with the opportunity for recovery. Unless `source` has no tokens
/// whatsoever, this always emits `Some`, although the returned tree may have
/// errors attached.
///
/// When faced with unexpected input, the parser raises an error and then tries
/// to skip ahead to the next top-level item, carriage return, or newline.
/// All input between the error location and the next valid thing gets wrapped
/// into a token tagged [`Syn::Unknown`].
#[must_use]
pub fn parse_file(source: &str, path: impl AsRef<VPath>) -> Option<FileParseTree> {
	let (root, errs) = primitive::choice((wsp_ext(source), item(source), annotation(source)))
		.repeated()
		.collect_g::<Syn, { Syn::Root as u16 }>()
		.parse_recovery(source);

	root.map(|r| FileParseTree {
		inner: ParseTree {
			root: r,
			errors: errs,
		},
		path: path.as_ref().to_path_buf(),
	})
}

#[must_use]
pub fn parse_repl(source: &str) -> Option<ParseTree> {
	let (root, errs) = primitive::choice((wsp_ext(source), expr(source)))
		.repeated()
		.collect_g::<Syn, { Syn::Root as u16 }>()
		.parse_recovery(source);

	root.map(|r| ParseTree {
		root: r,
		errors: errs,
	})
}

#[derive(Debug)]
pub struct ParseTree {
	pub root: GreenNode,
	pub errors: Vec<ParseError>,
}

impl ParseTree {
	/// Were any errors encountered when parsing a token stream?
	#[must_use]
	pub fn any_errors(&self) -> bool {
		!self.errors.is_empty()
	}
}

#[derive(Debug)]
pub struct FileParseTree {
	pub inner: ParseTree,
	pub path: VPathBuf,
}

impl std::ops::Deref for FileParseTree {
	type Target = ParseTree;

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

					if let Some(ptree) = parse_file(child.read_str(), child_path) {
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
		self.files.iter().any(|ptree| ptree.any_errors())
	}
}

fn item(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	primitive::choice((func_decl(src),))
}

fn func_decl(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	decl_quals(src)
		.or_not()
		.start_vec()
		.chain_push(return_types(src))
		.chain_append(wsp_ext(src).repeated().at_least(1))
		.chain_push(name(src))
		.chain_push(comb::just::<Syn, _>('(', Syn::ParenL, src))
		// TODO: Parameter list
		.chain_push(comb::just::<Syn, _>(')', Syn::ParenL, src))
		.chain_append(wsp_ext(src).repeated())
		.chain_push(primitive::choice((
			comb::just::<Syn, _>(';', Syn::Semicolon, src),
			block(src),
		)))
		.collect_n::<Syn, { Syn::FunctionDecl as u16 }>()
}

fn return_types(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	let rep1 = wsp_ext(src)
		.repeated()
		.chain_push(comb::just::<Syn, _>(',', Syn::Comma, src))
		.chain_append(wsp_ext(src).repeated())
		.chain_push(type_ref(src));

	type_ref(src)
		.start_vec()
		.then(rep1.repeated())
		.map(|(mut first, mut others)| {
			others.iter_mut().for_each(|vec| first.append(vec));
			first
		})
		.collect_n::<Syn, { Syn::ReturnTypes as u16 }>()
}
