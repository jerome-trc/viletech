//! Expression parsers.

use doomfront::{
	chumsky::{self, primitive, recursive::Recursive, Parser},
	comb,
	gcache::GreenCache,
	parser_t,
	parsing::*,
	rowan::GreenNode,
};

use crate::Syn;

use super::ParserBuilder;

impl<C: GreenCache> ParserBuilder<C> {
	/// The returned parser emits a node tagged with one of the following:
	/// - [`Syn::ArrayExpr`]
	/// - [`Syn::BinExpr`]
	/// - [`Syn::CallExpr`]
	/// - [`Syn::GroupedExpr`]
	/// - [`Syn::IdentExpr`]
	/// - [`Syn::IndexExpr`]
	/// - [`Syn::PostfixExpr`]
	/// - [`Syn::PrefixExpr`]
	pub fn expr<'i>(&self) -> parser_t!(Syn, GreenNode) {
		chumsky::recursive::recursive(|expr: Recursive<dyn Parser<'_, _, GreenNode, _>>| {
			let ident = self
				.ident()
				.map(|gtok| GreenNode::new(Syn::IdentExpr.into(), [gtok.into()]));

			let literal = primitive::choice((
				comb::just(Syn::FloatLit),
				comb::just(Syn::IntLit),
				comb::just(Syn::StringLit),
				comb::just(Syn::FalseLit),
				comb::just(Syn::TrueLit),
			))
			.map(|gtok| GreenNode::new(Syn::Literal.into(), [gtok.into()]));

			let grouped = primitive::group((
				comb::just(Syn::ParenL),
				self.trivia_0plus(),
				expr.clone(),
				self.trivia_0plus(),
				comb::just(Syn::ParenR),
			))
			.map(|group| coalesce_node(group, Syn::GroupedExpr));

			let _atom = primitive::choice((grouped, literal, ident));

			primitive::todo()
		})
	}
}
