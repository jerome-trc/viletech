//! Combinator-based parsers.

use doomfront::{
	chumsky::{primitive, recovery, Parser},
	comb, help,
	rowan::GreenNode,
	ParseError, ParseOut,
};

use super::Syn;

#[must_use = "combinator parsers are lazy and do nothing unless consumed"]
pub fn file_parser(src: &str) -> impl Parser<char, GreenNode, Error = ParseError> + '_ {
	primitive::choice((wsp_ext(src),))
		.recover_with(recovery::skip_then_retry_until([]))
		.labelled("item, annotation, or preprocessor directive")
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

#[must_use]
fn wsp_ext(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	comb::wsp_ext::<Syn, _>(src, comb::c_cpp_comment::<Syn>(src))
}
