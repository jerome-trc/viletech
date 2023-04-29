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

use crate::{
	data::{vfs::FileRef, VfsError},
	VPath, VPathBuf,
};

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
pub fn parse_file(source: &str) -> Option<FileParseTree> {
	let (root, errs) = primitive::choice((wsp_ext(source), item(source), annotation(source)))
		.repeated()
		.collect_g::<Syn, { Syn::Root as u16 }>()
		.parse_recovery(source);

	root.map(|r| ParseTree {
		root: r,
		errors: errs,
	})
}

#[must_use]
pub fn parse_repl(source: &str) -> Option<ReplParseTree> {
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
pub struct ParseTree<const REPL: bool> {
	root: GreenNode,
	errors: Vec<ParseError>,
}

impl<const REPL: bool> ParseTree<REPL> {
	#[must_use]
	pub fn root(&self) -> &GreenNode {
		&self.root
	}

	/// Were any errors encountered when parsing a token stream?
	#[must_use]
	pub fn any_errors(&self) -> bool {
		!self.errors.is_empty()
	}

	/// Errors encountered while parsing a token stream.
	#[must_use]
	pub fn errors(&self) -> &[ParseError] {
		&self.errors
	}

	#[must_use]
	pub fn into_errors(self) -> Vec<ParseError> {
		self.errors
	}
}

pub type FileParseTree = ParseTree<false>;
pub type ReplParseTree = ParseTree<true>;

#[derive(Debug, Default)]
pub struct IncludeTree {
	/// Element 0 is always the script root.
	pub(super) files: Vec<FileParseTree>,
	pub(super) parse_errs: Vec<ParseError>,
	/// Raised when a script tries to include a non-existent or unreadable file.
	pub(super) vfs_errs: Vec<VfsError>,
	pub(super) misc_errs: Vec<IncTreeError>,
}

impl IncludeTree {
	/// `mount_path` should be, for example, `/viletech`.
	/// `root` should be, for example, `/viletech/script/main.vzs`.
	/// Mind that the returned include tree can be empty if the root file has no
	/// tokens in it.
	pub fn new(mount_path: &VPath, root: FileRef) -> Self {
		fn try_parse(source: &str) -> Result<Option<FileParseTree>, Vec<ParseError>> {
			let ptree = match parse_file(source) {
				Some(pt) => pt,
				None => return Ok(None),
			};

			if ptree.any_errors() {
				Err(ptree.into_errors())
			} else {
				Ok(Some(ptree))
			}
		}

		fn get_includes(
			_ptree: &FileParseTree,
			_mount_path: &VPath,
		) -> Result<Vec<VPathBuf>, Vec<IncTreeError>> {
			unimplemented!("New include tree system pending.")
		}

		let root_src = match root.try_read_str() {
			Ok(src) => src,
			Err(err) => {
				return IncludeTree {
					vfs_errs: vec![err],
					..Default::default()
				}
			}
		};

		let root_pt = match try_parse(root_src) {
			Ok(fpt_opt) => match fpt_opt {
				Some(rpt) => rpt,
				None => return IncludeTree::default(),
			},
			Err(errs) => {
				return IncludeTree {
					parse_errs: errs,
					..Default::default()
				}
			}
		};

		let mut stack = match get_includes(&root_pt, mount_path) {
			Ok(incs) => incs,
			Err(errs) => {
				return IncludeTree {
					misc_errs: errs,
					..Default::default()
				}
			}
		};

		let rptq = SegQueue::default();
		let mut files = vec![];
		let parse_errs = Mutex::new(vec![]);
		let vfs_errs = Mutex::new(vec![]);
		let misc_errs = Mutex::new(vec![]);

		while !stack.is_empty() {
			stack.par_drain(..).for_each(|inc_path| {
				let fref = match root.vfs().get(&inc_path) {
					Some(f) => f,
					None => {
						vfs_errs.lock().push(VfsError::NotFound(inc_path));
						return;
					}
				};

				let src = match fref.try_read_str() {
					Ok(src) => src,
					Err(err) => {
						vfs_errs.lock().push(err);
						return;
					}
				};

				let rptree = match try_parse(src) {
					Ok(rpt_opt) => match rpt_opt {
						Some(rpt) => rpt,
						None => return,
					},
					Err(mut errs) => {
						parse_errs.lock().append(&mut errs);
						return;
					}
				};

				rptq.push(rptree);
			});

			while let Some(ptree) = rptq.pop() {
				match get_includes(&ptree, mount_path) {
					Ok(mut incs) => stack.append(&mut incs),
					Err(mut errs) => misc_errs.lock().append(&mut errs),
				}

				files.push(ptree);
			}
		}

		IncludeTree {
			files,
			parse_errs: parse_errs.into_inner(),
			vfs_errs: vfs_errs.into_inner(),
			misc_errs: misc_errs.into_inner(),
		}
	}

	#[must_use]
	pub fn into_inner(self) -> Vec<FileParseTree> {
		self.files
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		!self.parse_errs.is_empty() || !self.vfs_errs.is_empty() || !self.misc_errs.is_empty()
	}
}

/// Things that can go wrong when traversing an include tree that are not
/// [`VfsError`]s or [`ParseError`]s.
#[derive(Debug)]
pub enum IncTreeError {
	/// An `include` annotation received a non-literal argument.
	IllegalArgExpr(Syn),
	/// An `include` annotation received a non-string literal argument.
	IllegalArgLit(Syn),
}

impl std::error::Error for IncTreeError {}

impl std::fmt::Display for IncTreeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IllegalArgExpr(syn) => write!(
				f,
				"`include` annotation expected a literal expression argument, but found: {syn:#?}"
			),
			Self::IllegalArgLit(syn) => write!(
				f,
				"`include` annotation expected a string literal argument, but found: {syn:#?}"
			),
		}
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
