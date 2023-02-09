//! Combinator-based parsers.

use doomfront::{
	chumsky::{primitive, recovery, Parser},
	comb, help,
	rowan::GreenNode,
	ParseError, ParseOut, ParseTree,
};

use super::Syn;

mod common;
mod expr;
#[cfg(test)]
mod test;

use common::*;
use expr::*;

/// If `repl` is true, look for expressions and statements at the top level instead
/// of items, annotations, and macro invocations.
/// If `tolerant` is true, unexpected tokens are lexed as [`Syn::Unknown`] instead
/// of raising an error.
/// Either way, the only condition for a parse tree not to be returned is if the
/// source contains no tokens.
#[must_use]
pub fn parse(source: &str, repl: bool, tolerant: bool) -> Option<ParseTree<Syn>> {
	let (root, errs) = if !repl {
		if !tolerant {
			file_parser(source).parse_recovery(source)
		} else {
			file_parser_tolerant(source).parse_recovery(source)
		}
	} else if !tolerant {
		repl_parser(source).parse_recovery(source)
	} else {
		repl_parser_tolerant(source).parse_recovery(source)
	};

	root.map(|r| ParseTree::new(r, errs))
}

#[must_use = "combinator parsers are lazy and do nothing unless consumed"]
pub fn file_parser(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + '_ {
	primitive::choice((wsp_ext(src), item(src)))
		.recover_with(recovery::skip_then_retry_until([]))
		.labelled("item, annotation, or preprocessor directive")
		.repeated()
		.map(help::map_finish::<Syn>(Syn::Root))
}

#[must_use = "combinator parsers are lazy and do nothing unless consumed"]
pub fn file_parser_tolerant(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + '_ {
	primitive::choice((wsp_ext(src), item(src), unknown(src)))
		.repeated()
		.map(help::map_finish::<Syn>(Syn::Root))
}

#[must_use = "combinator parsers are lazy and do nothing unless consumed"]
pub fn repl_parser(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + '_ {
	primitive::choice((wsp_ext(src),))
		.recover_with(recovery::skip_then_retry_until([]))
		.labelled("expression or statement")
		.repeated()
		.map(help::map_finish::<Syn>(Syn::Root))
}

#[must_use = "combinator parsers are lazy and do nothing unless consumed"]
pub fn repl_parser_tolerant(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + '_ {
	primitive::choice((wsp_ext(src), unknown(src)))
		.repeated()
		.map(help::map_finish::<Syn>(Syn::Root))
}

#[must_use]
fn item(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::choice((func_decl(src),))
}

#[must_use]
fn func_decl(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	decl_quals(src)
		.or_not()
		.map(help::map_nvec_opt())
		.then(return_types(src))
		.map(help::map_push())
		.then(wsp_ext(src))
		.map(help::map_push())
		.then(ident(src))
		.map(help::map_push())
		.then(comb::just::<Syn>("(", Syn::LParen))
		.map(help::map_push())
		.then(comb::just::<Syn>(")", Syn::RParen))
		.map(help::map_push())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(primitive::choice((
			comb::just::<Syn>(";", Syn::Semicolon),
			block(src),
		)))
		.map(help::map_push())
		.map(help::map_collect::<Syn>(Syn::FunctionDecl))
}

#[must_use]
fn return_types(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	let rep1 = wsp_ext(src)
		.or_not()
		.map(help::map_nvec_opt())
		.then(comb::just::<Syn>(",", Syn::Comma))
		.map(help::map_push())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(type_expr(src))
		.map(help::map_push());

	type_expr(src)
		.map(help::map_nvec())
		.then(rep1.repeated())
		.map(|(mut first, mut others)| {
			others.iter_mut().for_each(|vec| first.append(vec));
			first
		})
		.map(help::map_collect::<Syn>(Syn::ReturnTypes))
}

#[must_use]
fn unknown(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::any().map_with_span(help::map_tok::<Syn, _>(src, Syn::Unknown))
}
