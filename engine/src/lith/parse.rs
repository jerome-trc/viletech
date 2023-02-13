//! Combinator-based parsers.

use crossbeam::queue::SegQueue;
use doomfront::{
	chumsky::{primitive, recovery, Parser},
	comb, help,
	rowan::{ast::AstNode, GreenNode},
	ParseError, ParseOut,
};
use log::warn;
use parking_lot::Mutex;
use rayon::prelude::*;

use crate::{data::FileRef, data::VfsError, lith::ast, VPath, VPathBuf};

use super::{IncludeTree, ParseTree, RawParseTree, Syn};

mod common;
mod expr;
mod lit;
#[cfg(test)]
mod test;

use common::*;

/// If `repl` is true, look for expressions and statements at the top level instead
/// of items, annotations, and macro invocations.
/// If `tolerant` is true, unexpected tokens are lexed as [`Syn::Unknown`] instead
/// of raising an error.
/// Either way, the only condition for a parse tree not to be returned is if the
/// source contains no tokens.
#[must_use]
pub fn parse(source: &str, repl: bool, tolerant: bool) -> Option<RawParseTree> {
	let (root, errs) = if !repl {
		if !tolerant {
			#[cfg(not(test))]
			let pt = file_parser(source).parse_recovery(source);
			#[cfg(test)]
			let pt = file_parser(source).parse_recovery_verbose(source);

			pt
		} else {
			#[cfg(not(test))]
			let pt = file_parser_tolerant(source).parse_recovery(source);
			#[cfg(test)]
			let pt = file_parser(source).parse_recovery_verbose(source);

			pt
		}
	} else if !tolerant {
		repl_parser(source).parse_recovery(source)
	} else {
		repl_parser_tolerant(source).parse_recovery(source)
	};

	root.map(|r| RawParseTree::new(r, errs))
}

/// `mount_path` should be, for example, `/viletech`.
/// `root` should be, for example, `/viletech/script/main.lith`.
pub fn parse_include_tree(mount_path: &VPath, root: FileRef) -> IncTreeResult {
	fn parse_file(source: &str) -> Result<Option<RawParseTree>, Vec<ParseError>> {
		let ptree = match parse(source, false, false) {
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
				// this will get flagged as an error; `include` can't be outer
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

pub fn file_parser(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + '_ {
	primitive::choice((wsp_ext(src), item(src), annotation(src)))
		.recover_with(recovery::skip_then_retry_until([]))
		.labelled("item or annotation")
		.repeated()
		.map(help::map_finish::<Syn>(Syn::Root))
}

pub fn file_parser_tolerant(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + '_ {
	primitive::choice((wsp_ext(src), item(src), annotation(src), unknown(src)))
		.repeated()
		.map(help::map_finish::<Syn>(Syn::Root))
}

pub fn repl_parser(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + '_ {
	primitive::choice((wsp_ext(src),))
		.recover_with(recovery::skip_then_retry_until([]))
		.labelled("expression or statement")
		.repeated()
		.map(help::map_finish::<Syn>(Syn::Root))
}

pub fn repl_parser_tolerant(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + '_ {
	primitive::choice((wsp_ext(src), unknown(src)))
		.repeated()
		.map(help::map_finish::<Syn>(Syn::Root))
}

fn item(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::choice((func_decl(src),))
}

fn func_decl(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	decl_quals(src)
		.or_not()
		.map(help::map_nvec_opt())
		.then(return_types(src))
		.map(help::map_push())
		.then(wsp_ext(src))
		.map(help::map_push())
		.then(name(src))
		.map(help::map_push())
		.then(comb::just::<Syn>("(", Syn::LParen))
		.map(help::map_push())
		.then(comb::just::<Syn>(")", Syn::RParen))
		.map(help::map_push())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(primitive::choice((
			comb::just::<Syn>(";", Syn::Semicolon),
			block(),
		)))
		.map(help::map_push())
		.map(help::map_collect::<Syn>(Syn::FunctionDecl))
}

fn return_types(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	let rep1 = wsp_ext(src)
		.or_not()
		.map(help::map_nvec_opt())
		.then(comb::just::<Syn>(",", Syn::Comma))
		.map(help::map_push())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(type_ref(src))
		.map(help::map_push());

	type_ref(src)
		.map(help::map_nvec())
		.then(rep1.repeated())
		.map(|(mut first, mut others)| {
			others.iter_mut().for_each(|vec| first.append(vec));
			first
		})
		.map(help::map_collect::<Syn>(Syn::ReturnTypes))
}

fn type_ref(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::choice((resolver(src),)).map(help::map_node::<Syn>(Syn::ExprType))
}

fn unknown(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::any().map_with_span(help::map_tok::<Syn, _>(src, Syn::Unknown))
}
