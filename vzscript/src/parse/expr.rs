//! Expression parsers.

use doomfront::{
	chumsky::{self, primitive, Parser},
	comb,
	util::builder::GreenCache,
};

use crate::{Syn, TokenStream};

use super::{Extra, ParserBuilder};

impl<C: GreenCache> ParserBuilder<C> {
	pub fn expr<'i>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
		chumsky::recursive::recursive(|expr| {
			primitive::choice((
				self.bin_expr(expr.clone()),
				self.type_expr(expr.clone()),
				self.grouped_expr(expr.clone()),
				self.lit_expr(),
			))
		})
		.boxed()
	}

	/// Builds a [`Syn::Literal`] node.
	pub fn lit_expr<'i>(&self) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone {
		comb::node(
			Syn::Literal.into(),
			primitive::choice((
				comb::just(Syn::StringLit),
				comb::just(Syn::IntLit),
				comb::just(Syn::FloatLit),
				comb::just(Syn::TrueLit),
				comb::just(Syn::FalseLit),
			)),
		)
	}

	/// Builds a [`Syn::BinExpr`] node.
	///
	/// [`ParserBuilder::expr`]'s return value must be passed in to prevent infinite recursion.
	pub fn bin_expr<'i, P>(
		&self,
		expr: P,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
	{
		let other = primitive::choice((
			self.type_expr(expr.clone()),
			self.grouped_expr(expr.clone()),
			self.lit_expr(),
		));

		comb::node(
			Syn::BinExpr.into(),
			primitive::group((
				other.clone(),
				self.trivia_0plus(),
				primitive::choice((comb::just(Syn::Plus), comb::just(Syn::Minus))),
				self.trivia_0plus(),
				expr,
			)),
		)
	}

	/// Builds a [`Syn::GroupedExpr`] node.
	///
	/// [`ParserBuilder::expr`]'s return value must be passed in to prevent infinite recursion.
	pub fn grouped_expr<'i, P>(
		&self,
		expr: P,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
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

	/// Builds a [`Syn::TypeExpr`] node.
	///
	/// [`ParserBuilder::expr`]'s return value must be passed in to prevent infinite recursion.
	pub fn type_expr<'i, P>(
		&self,
		expr: P,
	) -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
	where
		P: 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone,
	{
		let ident_chain = comb::node(Syn::TypeExpr.into(), self.ident_chain());

		let kw = comb::node(
			Syn::TypeExpr.into(),
			primitive::group((
				comb::just(Syn::KwType),
				self.trivia_0plus(),
				comb::just(Syn::ParenL),
				self.trivia_0plus(),
				expr,
				self.trivia_0plus(),
				comb::just(Syn::ParenR),
			)),
		);

		primitive::choice((ident_chain, kw))
	}
}

#[cfg(test)]
mod test {
	use doomfront::{testing::*, util::builder::GreenCacheNoop};

	use crate::Version;

	use super::*;

	#[test]
	fn smoke_lit_expr() {
		const SOURCE: &str = "2";

		let vers = Version::new(0, 0, 0);
		let builder = ParserBuilder::<GreenCacheNoop>::new(vers);
		let parser = builder.lit_expr();
		let stream = Syn::stream(SOURCE, vers);
		let ptree = doomfront::parse(parser, None, Syn::ReplRoot.into(), SOURCE, stream);

		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_bin_expr() {
		const SOURCE: &str = "2 + 2";

		let vers = Version::new(0, 0, 0);
		let builder = ParserBuilder::<GreenCacheNoop>::new(vers);
		let parser = builder.bin_expr(builder.expr());
		let stream = Syn::stream(SOURCE, vers);
		let ptree = doomfront::parse(parser, None, Syn::ReplRoot.into(), SOURCE, stream);

		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_type_expr() {
		const SOURCE: &str = "type((0 - 1) + 2 + 3)";

		let vers = Version::new(0, 0, 0);
		let builder = ParserBuilder::<GreenCacheNoop>::new(vers);
		let parser = builder.type_expr(builder.expr());
		let stream = Syn::stream(SOURCE, vers);
		let ptree = doomfront::parse(parser, None, Syn::ReplRoot.into(), SOURCE, stream);

		assert_no_errors(&ptree);
	}
}
