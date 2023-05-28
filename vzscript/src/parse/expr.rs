//! Expression parsers.

use doomfront::{
	chumsky::{self, primitive, Parser},
	comb,
	util::builder::GreenCache,
};

use crate::{
	lex::{Token, TokenStream},
	Syn,
};

use super::{Extra, ParserBuilder};

impl ParserBuilder {
	pub fn expr<'i, C>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		let ret = chumsky::recursive::recursive(|expr| {
			primitive::choice((
				self.atom_expr(),
				self.bin_expr(expr.clone()),
				self.grouped_expr(expr.clone()),
			))
		});

		#[cfg(any(debug_assertions, test))]
		{
			ret.boxed()
		}
		#[cfg(not(any(debug_assertions, test)))]
		{
			ret
		}
	}

	pub fn atom_expr<'i, C>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		let ret = comb::node(
			Syn::Literal.into(),
			primitive::choice((
				comb::just(Token::StringLit, Syn::StringLit.into()),
				comb::just(Token::IntLit, Syn::IntLit.into()),
				comb::just(Token::FloatLit, Syn::FloatLit.into()),
				comb::just(Token::TrueLit, Syn::TrueLit.into()),
				comb::just(Token::FalseLit, Syn::FalseLit.into()),
				comb::just(Token::VoidLit, Syn::VoidLit.into()),
			)),
		);

		#[cfg(any(debug_assertions, test))]
		{
			ret.boxed()
		}
		#[cfg(not(any(debug_assertions, test)))]
		{
			ret
		}
	}

	/// [`ParserBuilder::expr`]'s return value must be passed in to prevent infinite recursion.
	pub fn bin_expr<'i, C, P>(
		&self,
		expr: P,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
		P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
	{
		let ret = comb::node(
			Syn::BinExpr.into(),
			primitive::choice((
				comb::checkpointed(primitive::group((
					expr.clone(),
					self.trivia_0plus(),
					comb::just(Token::Plus, Syn::Plus.into()),
					self.trivia_0plus(),
					expr.clone(),
				))),
				comb::checkpointed(primitive::group((
					expr.clone(),
					self.trivia_0plus(),
					comb::just(Token::Minus, Syn::Minus.into()),
					self.trivia_0plus(),
					expr,
				))),
			))
			.map(|_| ()),
		);

		#[cfg(any(debug_assertions, test))]
		{
			ret.boxed()
		}
		#[cfg(not(any(debug_assertions, test)))]
		{
			ret
		}
	}

	pub fn grouped_expr<'i, C, P>(
		&self,
		expr: P,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
		P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
	{
		let ret = comb::node(
			Syn::GroupedExpr.into(),
			primitive::group((
				comb::just(Token::ParenL, Syn::ParenL.into()),
				self.trivia_0plus(),
				expr,
				self.trivia_0plus(),
				comb::just(Token::ParenR, Syn::ParenR.into()),
			)),
		);

		#[cfg(any(debug_assertions, test))]
		{
			ret.boxed()
		}
		#[cfg(not(any(debug_assertions, test)))]
		{
			ret
		}
	}
}

#[cfg(test)]
mod test {
	use doomfront::util::{builder::GreenCacheNoop, testing::*};

	use crate::Version;

	use super::*;

	#[test]
	fn smoke_atom_expr() {
		const SOURCE: &str = "2";

		let vers = Version::new(0, 0, 0);
		let builder = ParserBuilder::new(vers);
		let atom_expr = builder.atom_expr::<GreenCacheNoop>();
		let stream = Token::stream(SOURCE, vers);
		let ptree = doomfront::parse(atom_expr, None, Syn::ReplRoot.into(), SOURCE, stream);

		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_bin_expr() {
		const SOURCE: &str = "2 + 2";

		let vers = Version::new(0, 0, 0);
		let builder = ParserBuilder::new(vers);
		let expr = builder.expr::<GreenCacheNoop>();
		let expr_bin = builder.bin_expr(expr);
		let stream = Token::stream(SOURCE, vers);
		let ptree = doomfront::parse(expr_bin, None, Syn::ReplRoot.into(), SOURCE, stream);

		assert_no_errors(&ptree);
	}
}
