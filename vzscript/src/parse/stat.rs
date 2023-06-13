//! Statement parsers.

use doomfront::{
	chumsky::{self, primitive, Parser},
	comb,
	gcache::GreenCache,
	parser_t,
	parsing::*,
	rowan::GreenNode,
};

use crate::Syn;

use super::ParserBuilder;

impl<C: GreenCache> ParserBuilder<C> {
	pub fn statement<'i>(&self) -> parser_t!(Syn, GreenNode) {
		chumsky::recursive::recursive(|_| {
			let s_binding = primitive::group((
				primitive::group((comb::just(Syn::KwCeval), self.trivia_1plus())).or_not(),
				primitive::choice((comb::just(Syn::KwLet), comb::just(Syn::KwConst))),
				self.trivia_1plus(),
				self.ident(),
				self.trivia_0plus(),
				self.type_spec(),
				comb::just(Syn::Eq),
				self.trivia_0plus(),
				self.expr(),
				self.trivia_0plus(),
				comb::just(Syn::Semicolon),
			))
			.map(|group| coalesce_node(group, Syn::BindStat));

			let s_break = primitive::group((
				comb::just(Syn::KwBreak),
				self.trivia_0plus(),
				self.block_label().or_not(),
				self.trivia_0plus(),
				comb::just(Syn::Semicolon),
			))
			.map(|group| coalesce_node(group, Syn::BreakStat));

			let s_continue = primitive::group((
				comb::just(Syn::KwContinue),
				self.trivia_0plus(),
				self.block_label().or_not(),
				self.trivia_0plus(),
				comb::just(Syn::Semicolon),
			))
			.map(|group| coalesce_node(group, Syn::ContinueStat));

			let s_expr =
				primitive::group((self.expr(), self.trivia_0plus(), comb::just(Syn::Semicolon)))
					.map(|group| coalesce_node(group, Syn::ExprStat));

			let s_ret = primitive::group((
				comb::just(Syn::KwReturn),
				primitive::group((self.trivia_1plus(), self.expr())).or_not(),
				self.trivia_0plus(),
				comb::just(Syn::Semicolon),
			))
			.map(|group| coalesce_node(group, Syn::ReturnStat));

			primitive::choice((s_expr, s_continue, s_break, s_ret, s_binding)).boxed()
		})
	}
}
