//! Combinator-based parsers.

mod common;
mod expr;
mod lit;
#[cfg(test)]
mod test;

use bevy::prelude::warn;
use crossbeam::queue::SegQueue;
use doomfront::{
	chumsky::{primitive, Parser},
	comb,
	ext::{Parser1, ParserOpt, ParserVec},
	rowan::{ast::AstNode, GreenNode},
	ParseError, ParseOut,
};
use parking_lot::Mutex;
use rayon::prelude::*;

use crate::{
	data::{FileRef, VfsError},
	vzs::ast,
	VPath, VPathBuf,
};

use super::{IncludeTree, ParseTree, RawParseTree, Syn};

use self::{common::*, expr::*};

/// Parses with the opportunity for recovery. Unless `source` has no tokens
/// whatsoever, this always emits `Some`, although the returned tree may have
/// errors attached.
///
/// When faced with unexpected input, the parser raises an error and then tries
/// to skip ahead to the next CVar definition, carriage return, or newline.
/// All input between the error location and the next valid thing gets wrapped
/// into a token tagged [`Syn::Unknown`].
pub fn parse(source: &str, repl: bool) -> Option<RawParseTree> {
	let (root, errs) = if !repl {
		file_parser(source).parse_recovery(source)
	} else {
		repl_parser(source).parse_recovery(source)
	};

	root.map(|r| RawParseTree::new(r, errs))
}

/// `mount_path` should be, for example, `/viletech`.
/// `root` should be, for example, `/viletech/script/main.vzs`.
pub fn parse_include_tree(mount_path: &VPath, root: FileRef) -> IncTreeResult {
	fn parse_file(source: &str) -> Result<Option<RawParseTree>, Vec<ParseError>> {
		let ptree = match parse(source, false) {
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
		ptree: &ParseTree,
		mount_path: &VPath,
	) -> Result<Vec<VPathBuf>, Vec<IncTreeError>> {
		let mut includes = vec![];
		let mut errs = vec![];

		for top in ptree.ast() {
			let anno = if let ast::Root::Annotation(a) = top {
				a
			} else {
				continue;
			};

			if anno.resolver().syntax().text() != "include" {
				continue;
			}

			if !anno.is_inner() {
				// When pairing annotations with syntax nodes later,
				// this will get flagged as an error; `include` can't be outer.
				continue;
			}

			let args = if let Some(a) = anno.args() {
				a
			} else {
				continue;
			};

			for arg in args.iter() {
				if arg.label().is_some() {
					warn!("Ignoring labelled `include`: {}", arg.syntax().text());
					continue;
				}

				let lit_expr = if let Some(lit) = arg.expr().into_literal() {
					lit.token()
				} else {
					errs.push(IncTreeError::IllegalArgExpr(arg.expr().syntax().kind()));
					continue;
				};

				let string = if let Some(s) = lit_expr.string() {
					s
				} else {
					errs.push(IncTreeError::IllegalArgLit(lit_expr.syntax().kind()));
					continue;
				};

				let rel = VPath::new(string);
				includes.push([mount_path, rel].iter().collect());
			}
		}

		if errs.is_empty() {
			Ok(includes)
		} else {
			Err(errs)
		}
	}

	let root_src = match root.try_read_str() {
		Ok(src) => src,
		Err(err) => {
			return IncTreeResult {
				vfs_errs: vec![err],
				..Default::default()
			}
		}
	};

	let root_pt = match parse_file(root_src) {
		Ok(rpt_opt) => match rpt_opt {
			Some(rpt) => ParseTree::new(rpt),
			None => return IncTreeResult::default(),
		},
		Err(errs) => {
			return IncTreeResult {
				parse_errs: errs,
				..Default::default()
			}
		}
	};

	let mut stack = match get_includes(&root_pt, mount_path) {
		Ok(incs) => incs,
		Err(errs) => {
			return IncTreeResult {
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
			let fref = match root.catalog().get_file(&inc_path) {
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

			let rptree = match parse_file(src) {
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

		while let Some(rpt) = rptq.pop() {
			let ptree = ParseTree::new(rpt);

			match get_includes(&ptree, mount_path) {
				Ok(mut incs) => stack.append(&mut incs),
				Err(mut errs) => misc_errs.lock().append(&mut errs),
			}

			files.push(ptree);
		}
	}

	let mut ret = IncTreeResult {
		tree: None,
		parse_errs: parse_errs.into_inner(),
		vfs_errs: vfs_errs.into_inner(),
		misc_errs: misc_errs.into_inner(),
	};

	if !ret.any_errors() {
		ret.tree = Some(IncludeTree { files });
	}

	ret
}

#[derive(Debug, Default)]
#[must_use]
pub struct IncTreeResult {
	/// If this is `None`, there was nothing to parse in the include tree's root,
	/// or an error occurred.
	pub tree: Option<IncludeTree>,
	pub parse_errs: Vec<ParseError>,
	/// Raised when a script tries to include a non-existent or unreadable file.
	pub vfs_errs: Vec<VfsError>,
	pub misc_errs: Vec<IncTreeError>,
}

impl IncTreeResult {
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

fn file_parser(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + Clone + '_ {
	primitive::choice((wsp_ext(src), item(src), annotation(src)))
		.repeated()
		.collect_g::<Syn, { Syn::Root as u16 }>()
}

fn repl_parser(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + Clone + '_ {
	primitive::choice((wsp_ext(src), expr(src)))
		.repeated()
		.collect_g::<Syn, { Syn::Root as u16 }>()
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
