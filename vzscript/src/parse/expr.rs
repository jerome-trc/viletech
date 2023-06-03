//! Expression parsers.

use doomfront::{
	chumsky::{self, primitive, Parser},
	comb,
	util::builder::GreenCache,
};

use crate::{Syn, TokenStream};

use super::{Extra, ParserBuilder};

impl ParserBuilder {
	pub fn expr<'i, C>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		chumsky::recursive::recursive(|expr| {
			primitive::choice((
				self.atom_expr(),
				self.bin_expr(expr.clone()),
				self.grouped_expr(expr.clone()),
			))
		})
		.boxed()
	}

	pub fn atom_expr<'i, C>(
		&self,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
	{
		comb::node(
			Syn::Literal.into(),
			primitive::choice((
				comb::just(Syn::StringLit),
				comb::just(Syn::IntLit),
				comb::just(Syn::FloatLit),
				comb::just(Syn::TrueLit),
				comb::just(Syn::FalseLit),
				comb::just(Syn::VoidLit),
			)),
		)
		.boxed()
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
		comb::node(
			Syn::BinExpr.into(),
			primitive::choice((
				comb::checkpointed(primitive::group((
					expr.clone(),
					self.trivia_0plus(),
					comb::just(Syn::Plus),
					self.trivia_0plus(),
					expr.clone(),
				))),
				comb::checkpointed(primitive::group((
					expr.clone(),
					self.trivia_0plus(),
					comb::just(Syn::Minus),
					self.trivia_0plus(),
					expr,
				))),
			))
			.map(|_| ()),
		)
	}

	pub fn grouped_expr<'i, C, P>(
		&self,
		expr: P,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		C: GreenCache,
		P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
	{
		comb::node(
			Syn::GroupedExpr.into(),
			primitive::group((
				comb::just(Syn::ParenL),
				self.trivia_0plus(),
				expr,
				self.trivia_0plus(),
				comb::just(Syn::ParenR),
			)),
		)
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
		let stream = Syn::stream(SOURCE, vers);
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
		let stream = Syn::stream(SOURCE, vers);
		let ptree = doomfront::parse(expr_bin, None, Syn::ReplRoot.into(), SOURCE, stream);

		assert_no_errors(&ptree);
	}
}
