//! Combinators used by multiple other combinators.

use doomfront::{
	chumsky::{primitive, text, Parser},
	comb,
	ext::{Parser1, ParserVec},
	help, ParseError, ParseOut,
};

use crate::vzs::parse::expr::*;

use super::Syn;

pub(super) fn annotation(
	src: &str,
) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	comb::just::<Syn, _>('#', Syn::Pound, src)
		.start_vec()
		.chain_push_opt(comb::just::<Syn, _>('!', Syn::Bang, src).or_not())
		.chain_push(comb::just::<Syn, _>('[', Syn::BracketL, src))
		.chain_append(wsp_ext(src).repeated())
		.chain_push(resolver(src))
		.chain_push_opt(arg_list(src).or_not())
		.chain_append(wsp_ext(src).repeated())
		.chain_push(comb::just::<Syn, _>(']', Syn::BracketR, src))
		.collect_n::<Syn, { Syn::Annotation as u16 }>()
}

/// Includes delimiting parentheses.
pub(super) fn arg_list(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	let anon = expr(src).map(help::map_node::<Syn>(Syn::Argument));

	let labelled = label(src)
		.start_vec()
		.chain_push(comb::just::<Syn, _>(':', Syn::Colon, src))
		.chain_append(wsp_ext(src).repeated())
		.chain_push(expr(src))
		.collect_n::<Syn, { Syn::Argument as u16 }>();

	let rep = comb::just::<Syn, _>(',', Syn::Comma, src)
		.start_vec()
		.chain_append(wsp_ext(src).repeated())
		.chain_push(primitive::choice((labelled.clone(), anon.clone())))
		.chain_append(wsp_ext(src).repeated());

	comb::just::<Syn, _>('(', Syn::ParenL, src)
		.start_vec()
		.chain_append(wsp_ext(src).repeated())
		.chain_push_opt(primitive::choice((labelled.clone(), anon.clone())).or_not())
		.chain_append(wsp_ext(src).repeated())
		.then(rep.repeated())
		.map(|(mut vec, mut v_v)| {
			v_v.iter_mut().for_each(|v| vec.append(v));
			vec
		})
		.chain_push(comb::just::<Syn, _>(')', Syn::ParenR, src))
		.collect_n::<Syn, { Syn::ArgList as u16 }>()
}

pub(super) fn block(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	comb::just::<Syn, _>('{', Syn::BraceL, src)
		.start_vec()
		.chain_push(comb::just::<Syn, _>('}', Syn::BraceR, src))
		.collect_n::<Syn, { Syn::Block as u16 }>()
}

pub(super) fn decl_quals(
	src: &str,
) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	let rep = wsp_ext(src).then(decl_qual(src));

	decl_qual(src)
		.start_vec()
		.then(rep.repeated())
		.map(|(mut vec, others)| {
			for (wsp, qual) in others {
				vec.push(wsp);
				vec.push(qual);
			}

			vec
		})
		.chain_append(wsp_ext(src).repeated().at_least(1))
		.collect_n::<Syn, { Syn::DeclQualifiers as u16 }>()
}

pub(super) fn decl_qual(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
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

pub(super) fn ident(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	text::ident().map_with_span(help::map_tok::<Syn, _>(src, Syn::Ident))
}

pub(super) fn label(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	ident(src).map(help::map_node::<Syn>(Syn::Label))
}

pub(super) fn name(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	ident(src).map(help::map_node::<Syn>(Syn::Name))
}

pub(super) fn resolver(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	let rep = comb::just::<Syn, _>("::", Syn::Colon2, src).then(name(src));

	comb::just::<Syn, _>("::", Syn::Colon2, src)
		.or_not()
		.map(|opt| match opt {
			Some(n_or_t) => vec![n_or_t],
			None => vec![],
		})
		.chain_push(name(src))
		.then(rep.repeated())
		.map(|(mut vec, parts)| {
			for (p_sep, p_ident) in parts {
				vec.push(p_sep);
				vec.push(p_ident);
			}

			vec
		})
		.collect_n::<Syn, { Syn::Resolver as u16 }>()
}

pub(super) fn type_ref(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	primitive::choice((resolver(src),)).map(help::map_node::<Syn>(Syn::TypeRef))
}

pub(super) fn wsp_ext(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + Clone + '_ {
	comb::wsp_ext::<Syn, _>(comb::c_cpp_comment::<Syn>(src), src)
}
