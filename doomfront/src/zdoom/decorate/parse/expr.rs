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
/// See [`primary_expr`] for a relevant caveat.
pub fn expr<'i, C>(
	property: bool,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	chumsky::recursive::recursive(|expr| {
		let prim = primary_expr(expr.clone());

		primitive::choice((
			bin_expr(expr.clone(), property),
			prim.clone(),
			unary_expr_prefix(prim.clone()),
			unary_expr_postfix(prim),
		))
	})
	.boxed()
}

/// Builds a [`Syn::BinExpr`] node.
/// [`expr`]'s return value must be passed in to prevent infinite recursion.
pub fn bin_expr<'i, C, P>(
	expr: P,
	property: bool,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
	P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
{
	let prim = primary_expr(expr.clone());

	let lhs = primitive::choice((
		prim.clone(),
		unary_expr_prefix(prim.clone()),
		unary_expr_postfix(prim),
	));

	let op_common = primitive::choice((
		comb::just_ts(Token::Ampersand2, Syn::Ampersand2.into()),
		comb::just_ts(Token::Ampersand, Syn::Ampersand.into()),
		comb::just_ts(Token::Pipe, Syn::Pipe.into()),
		comb::just_ts(Token::Eq2, Syn::Eq2.into()),
		comb::just_ts(Token::AngleLEq, Syn::AngleLEq.into()),
		comb::just_ts(Token::AngleREq, Syn::AngleREq.into()),
		comb::just_ts(Token::AngleL, Syn::AngleL.into()),
		comb::just_ts(Token::AngleR, Syn::AngleR.into()),
		comb::just_ts(Token::Slash, Syn::Slash.into()),
		comb::just_ts(Token::Asterisk, Syn::Asterisk.into()),
		comb::just_ts(Token::Caret, Syn::Caret.into()),
	));

	if !property {
		let op = primitive::choice((
			op_common,
			comb::just_ts(Token::Plus, Syn::Plus.into()),
			comb::just_ts(Token::Minus, Syn::Minus.into()),
		));

		comb::node(
			Syn::BinExpr.into(),
			primitive::group((lhs, trivia_0plus(), op, trivia_0plus(), expr)),
		)
	} else {
		comb::node(
			Syn::BinExpr.into(),
			primitive::group((lhs, trivia_0plus(), op_common, trivia_0plus(), expr)),
		)
	}
}

/// Builds a [`Syn::CallExpr`] node.
/// Note that this does not require an argument list.
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
			comb::just_ts(Token::Ident, Syn::Ident.into()),
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
			comb::just_ts(Token::BracketL, Syn::BracketL.into()),
			trivia_0plus(),
			comb::just_ts(Token::Ident, Syn::Ident.into()),
			trivia_0plus(),
			comb::just_ts(Token::BracketR, Syn::BracketR.into()),
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
			comb::just_ts(Token::ParenL, Syn::ParenL.into()),
			trivia_0plus(),
			expr_list(expr),
			trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR.into()),
		)),
	)
}

/// Builds a [`Syn::GroupedExpr`] node.
/// [`expr`]'s return value must be passed in to prevent infinite recursion.
pub fn grouped_expr<'i, C, P>(
	expr: P,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
	P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
{
	comb::node(
		Syn::GroupedExpr.into(),
		primitive::group((
			comb::just_ts(Token::ParenL, Syn::ParenL.into()),
			trivia_0plus(),
			expr,
			trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR.into()),
		)),
	)
}

/// Builds a [`Syn::IdentExpr`] node.
pub fn ident_expr<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(Syn::IdentExpr.into(), ident_chain())
}

/// Builds a [`Syn::IndexExpr`] node.
/// [`expr`]'s return value must be passed in to prevent infinite recursion.
pub fn index_expr<'i, C, P>(
	expr: P,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
	P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
{
	comb::node(
		Syn::IndexExpr.into(),
		primitive::group((
			ident_expr(),
			trivia_0plus(),
			comb::just_ts(Token::BracketL, Syn::BracketL.into()),
			trivia_0plus(),
			expr,
			trivia_0plus(),
			comb::just_ts(Token::BracketR, Syn::BracketR.into()),
		)),
	)
}

/// Builds a [`Syn::Literal`] node.
pub fn lit_expr<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::Literal.into(),
		primitive::choice((
			comb::just_ts(Token::StringLit, Syn::StringLit.into()),
			comb::just_ts(Token::IntLit, Syn::IntLit.into()),
			comb::just_ts(Token::FloatLit, Syn::FloatLit.into()),
			comb::just_ts(Token::KwTrue, Syn::KwTrue.into()),
			comb::just_ts(Token::KwFalse, Syn::KwFalse.into()),
		)),
	)
}

/// Builds a node tagged [`Syn::IdentExpr`], [`Syn::Literal`], [`Syn::GroupedExpr`],
/// or [`Syn::UnaryExpr`] (postfix only).
/// [`expr`]'s return value must be passed in to prevent infinite recursion.
///
/// Parsing DECORATE expressions is tricky. In the reference implementation, a
/// function call does not require a parenthesis-delimited argument list, making
/// them ambiguous with all other identifiers. As such, this uses a special
/// variation on [`call_expr`] that requires the argument list; all other identifiers
/// are assumed to be atoms, and must be handled by DoomFront's callers.
pub fn primary_expr<'i, C, P>(
	expr: P,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
	P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
{
	primitive::choice((
		comb::node(
			Syn::CallExpr.into(),
			primitive::group((
				comb::just_ts(Token::Ident, Syn::Ident.into()),
				trivia_0plus(),
				rng_spec().or_not(),
				trivia_0plus(),
				arg_list(expr.clone()),
			)),
		),
		index_expr(expr.clone()),
		grouped_expr(expr),
		ident_expr(),
		lit_expr(),
	))
}

/// Builds a [`Syn::UnaryExpr`] node.
/// [`expr`]'s return value must be passed in to prevent infinite recursion.
pub fn unary_expr<'i, C, P>(
	expr: P,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
	P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
{
	primitive::choice((unary_expr_prefix(expr.clone()), unary_expr_postfix(expr)))
}

/// Builds a [`Syn::PrefixExpr`] node.
/// [`primary_expr`]'s return value must be passed in to prevent infinite recursion.
pub fn unary_expr_prefix<'i, C, P>(
	pex: P,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
	P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
{
	comb::node(
		Syn::PrefixExpr.into(),
		primitive::group((
			primitive::choice((
				comb::just_ts(Token::Plus2, Syn::Plus2.into()),
				comb::just_ts(Token::Minus2, Syn::Minus2.into()),
				comb::just_ts(Token::Plus, Syn::Plus.into()),
				comb::just_ts(Token::Minus, Syn::Minus.into()),
			)),
			trivia_0plus(),
			pex,
		)),
	)
}

/// Builds a [`Syn::PostfixExpr`] node.
/// [`primary_expr`]'s return value must be passed in to prevent infinite recursion.
pub fn unary_expr_postfix<'i, C, P>(
	pex: P,
) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
	P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
{
	comb::node(
		Syn::PostfixExpr.into(),
		primitive::group((
			pex,
			trivia_0plus(),
			primitive::choice((
				comb::just_ts(Token::Plus2, Syn::Plus2.into()),
				comb::just_ts(Token::Minus2, Syn::Minus2.into()),
			)),
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
			comb::just_ts(Token::Comma, Syn::Comma.into()),
			trivia_0plus(),
			expr,
		)))
		.repeated()
		.collect::<()>(),
	))
	.map(|_| ())
}

#[cfg(test)]
mod test {
	use crate::{testing::*, util::builder::GreenCacheNoop};

	use super::*;

	#[test]
	fn smoke() {
		const SOURCE: &str = "x ^ ((a * b) + (c / d)) | y & z && foo";

		let parser = expr(false);

		let ptree = crate::parse(
			parser,
			Some(GreenCacheNoop),
			Syn::Root.into(),
			SOURCE,
			Token::stream(SOURCE),
		);

		assert_no_errors(&ptree);
	}
}
