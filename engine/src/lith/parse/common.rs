//! Combinators used by multiple other combinators.

use doomfront::{
	chumsky::{primitive, text, Parser},
	comb, help, ParseError, ParseOut,
};

use super::Syn;

#[must_use]
pub(super) fn block(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	primitive::just('{')
		.map_with_span(help::map_tok::<Syn, _>(src, Syn::LBrace))
		.map(help::map_nvec())
		.then(primitive::just('}').map_with_span(help::map_tok::<Syn, _>(src, Syn::LBrace)))
		.map(help::map_push())
		.map(help::map_collect::<Syn>(Syn::Block))
}

#[must_use]
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

#[must_use]
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

#[must_use]
pub(super) fn ident(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	text::ident().map_with_span(help::map_tok::<Syn, _>(src, Syn::Identifier))
}

#[must_use]
pub(super) fn resolver(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	let rep = comb::just::<Syn>("::", Syn::Colon2).then(ident(src));

	comb::just::<Syn>("::", Syn::Colon2)
		.or_not()
		.map(|opt| match opt {
			Some(n_or_t) => vec![n_or_t],
			None => vec![],
		})
		.then(ident(src))
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

#[must_use]
pub(super) fn wsp_ext(src: &str) -> impl Parser<char, ParseOut, Error = ParseError> + '_ {
	comb::wsp_ext::<Syn, _>(src, comb::c_cpp_comment::<Syn>(src))
}
