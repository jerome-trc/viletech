//! Combinators used by multiple other combinators.

use doomfront::{
	chumsky::{primitive, text, Parser},
	comb, help, ParseError, ParseOut,
};

use crate::lith::parse::expr::*;

use super::Syn;

pub(super) fn annotation(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	comb::just::<Syn>("#", Syn::Pound)
		.map(help::map_nvec())
		.then(comb::just::<Syn>("!", Syn::Bang).or_not())
		.map(help::map_push_opt())
		.then(comb::just::<Syn>("[", Syn::LBracket))
		.map(help::map_push())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(resolver(src))
		.map(help::map_push())
		.then(arg_list(src).or_not())
		.map(help::map_push_opt())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(comb::just::<Syn>("]", Syn::RBracket))
		.map(help::map_push())
		.map(help::map_collect::<Syn>(Syn::Annotation))
}

/// Includes delimiting parentheses.
pub(super) fn arg_list(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	let anon = || expr(src).map(help::map_node::<Syn>(Syn::Argument));

	let labelled = || {
		label(src)
			.map(help::map_nvec())
			.then(comb::just::<Syn>(":", Syn::Colon))
			.map(help::map_push())
			.then(wsp_ext(src).or_not())
			.map(help::map_push_opt())
			.then(expr(src))
			.map(help::map_push())
			.map(help::map_collect::<Syn>(Syn::Argument))
	};

	let rep = comb::just::<Syn>(",", Syn::Comma)
		.map(help::map_nvec())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(primitive::choice((labelled(), anon())))
		.map(help::map_push())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt());

	comb::just::<Syn>("(", Syn::LParen)
		.map(help::map_nvec())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(primitive::choice((labelled(), anon())).or_not())
		.map(help::map_push_opt())
		.then(wsp_ext(src).or_not())
		.map(help::map_push_opt())
		.then(rep.repeated())
		.map(|(mut vec, mut v_v)| {
			v_v.iter_mut().for_each(|v| vec.append(v));
			vec
		})
		.then(comb::just::<Syn>(")", Syn::RParen))
		.map(help::map_push())
		.map(help::map_collect::<Syn>(Syn::ArgList))
}

pub(super) fn block() -> impl Parser<char, ParseOut, Error = ParseError> {
	comb::just::<Syn>("{", Syn::LBrace)
		.map(help::map_nvec())
		.then(comb::just::<Syn>("}", Syn::RBrace))
		.map(help::map_push())
		.map(help::map_collect::<Syn>(Syn::Block))
}

pub(super) fn decl_quals(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	let rep = wsp_ext(src).then(decl_qual(src));

	decl_qual(src)
		.map(help::map_nvec())
		.then(rep.repeated())
		.map(|(mut vec, others)| {
			for (wsp, qual) in others {
				vec.push(wsp);
				vec.push(qual);
			}

			vec
		})
		.then(wsp_ext(src))
		.map(help::map_push())
		.map(help::map_collect::<Syn>(Syn::DeclQualifiers))
}

pub(super) fn decl_qual(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::choice((
		text::keyword("abstract").map_with_span(help::map_tok::<Syn, _>(src, Syn::KwAbstract)),
		text::keyword("ceval").map_with_span(help::map_tok::<Syn, _>(src, Syn::KwCEval)),
		text::keyword("final").map_with_span(help::map_tok::<Syn, _>(src, Syn::KwFinal)),
		text::keyword("override").map_with_span(help::map_tok::<Syn, _>(src, Syn::KwOverride)),
		text::keyword("private").map_with_span(help::map_tok::<Syn, _>(src, Syn::KwPrivate)),
		text::keyword("protected").map_with_span(help::map_tok::<Syn, _>(src, Syn::KwProtected)),
		text::keyword("static").map_with_span(help::map_tok::<Syn, _>(src, Syn::KwStatic)),
		text::keyword("virtual").map_with_span(help::map_tok::<Syn, _>(src, Syn::KwVirtual)),
	))
}

pub(super) fn ident(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	text::ident().map_with_span(help::map_tok::<Syn, _>(src, Syn::Ident))
}

pub(super) fn label(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	ident(src).map(help::map_node::<Syn>(Syn::Label))
}

pub(super) fn name(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	ident(src).map(help::map_node::<Syn>(Syn::Name))
}

pub(super) fn resolver(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	let rep = comb::just::<Syn>("::", Syn::Colon2).then(name(src));

	comb::just::<Syn>("::", Syn::Colon2)
		.or_not()
		.map(|opt| match opt {
			Some(n_or_t) => vec![n_or_t],
			None => vec![],
		})
		.then(name(src))
		.map(help::map_push())
		.then(rep.repeated())
		.map(|(mut vec, parts)| {
			for (p_sep, p_ident) in parts {
				vec.push(p_sep);
				vec.push(p_ident);
			}

			vec
		})
		.map(help::map_collect::<Syn>(Syn::Resolver))
}

pub(super) fn wsp_ext(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	comb::wsp_ext::<Syn, _>(src, comb::c_cpp_comment::<Syn>(src))
}
