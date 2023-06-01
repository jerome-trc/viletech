use chumsky::{primitive, IterParser, Parser};

use crate::{
	comb,
	util::builder::GreenCache,
	zdoom::{
		decorate::Syn,
		lex::{Token, TokenStream},
		Extra,
	},
};

use super::common::*;

/// Builds expression nodes recursively.
///
/// Parsing DECORATE expressions is tricky. In the reference implementation,
/// a function call does not require a parenthesis-delimited argument list,
/// making them ambiguous with all other identifiers. As such, it uses a special
/// variation on `call_expr` that requires the argument list; all other identifiers
/// are assumed to be atoms, and must be handled by DoomFront's callers.
pub fn expr<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let ident = comb::node(
		Syn::IdentExpr.into(),
		comb::just(Token::Ident, Syn::Ident.into()),
	);

	let atom = primitive::choice((lit_expr(), ident));

	chumsky::recursive::recursive(|expr| primitive::choice((atom, call_expr(expr)))).boxed()
}

pub fn lit_expr<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::Literal.into(),
		primitive::choice((
			comb::just(Token::StringLit, Syn::StringLit.into()),
			comb::just(Token::IntLit, Syn::IntLit.into()),
			comb::just(Token::FloatLit, Syn::FloatLit.into()),
			comb::just(Token::KwTrue, Syn::KwTrue.into()),
			comb::just(Token::KwFalse, Syn::KwFalse.into()),
		)),
	)
}

/// Builds a [`Syn::CallExpr`] node. Note that this requires an argument list
/// [`expr`]'s return value must be passed in to prevent infinite recursion.
pub fn call_expr<'i, C, P>(
	expr: P,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
	P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
{
	comb::node(
		Syn::CallExpr.into(),
		primitive::group((
			comb::just(Token::Ident, Syn::Ident.into()),
			trivia_0plus(),
			rng_spec().or_not(),
			trivia_0plus(),
			arg_list(expr).or_not(),
		)),
	)
}

/// Builds a [`Syn::RngSpec`] node.
/// Can be part of a function call between the identifier and argument list.
pub fn rng_spec<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::RngSpec.into(),
		primitive::group((
			comb::just(Token::BracketL, Syn::BracketL.into()),
			trivia_0plus(),
			comb::just(Token::Ident, Syn::Ident.into()),
			trivia_0plus(),
			comb::just(Token::BracketR, Syn::BracketR.into()),
		)),
	)
}

/// Builds a [`Syn::ArgList`] node.
/// [`expr`]'s return value must be passed in to prevent infinite recursion.
pub fn arg_list<'i, C, P>(
	expr: P,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
	P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
{
	comb::node(
		Syn::ArgList.into(),
		primitive::group((
			comb::just(Token::ParenL, Syn::ParenL.into()),
			trivia_0plus(),
			expr_list(expr),
			trivia_0plus(),
			comb::just(Token::ParenR, Syn::ParenR.into()),
		)),
	)
}

/// Builds a series of expression nodes (separated by commas).
/// [`expr`]'s return value must be passed in to prevent infinite recursion.
pub fn expr_list<'i, C, P>(
	expr: P,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
	P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
{
	primitive::group((
		expr.clone(),
		comb::checkpointed(primitive::group((
			trivia_0plus(),
			comb::just(Token::Comma, Syn::Comma.into()),
			trivia_0plus(),
			expr,
		)))
		.repeated()
		.collect::<()>(),
	))
	.map(|_| ())
}
