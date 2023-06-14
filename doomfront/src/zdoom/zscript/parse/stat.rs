//! Statement parsers.

use chumsky::{primitive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{zscript::Syn, Token},
	GreenElement,
};

use super::ParserBuilder;

impl ParserBuilder {
	/// The returned parser emits a node tagged with one of the following:
	/// - [`Syn::AssignStat`]
	/// - [`Syn::BreakStat`]
	/// - [`Syn::CaseStat`]
	/// - [`Syn::CompoundStat`]
	/// - [`Syn::ContinueStat`]
	/// - [`Syn::DeclAssignStat`]
	/// - [`Syn::DefaultStat`]
	/// - [`Syn::DoUntilStat`]
	/// - [`Syn::DoWhileStat`]
	/// - [`Syn::EmptyStat`]
	/// - [`Syn::ExprStat`]
	/// - [`Syn::ForStat`]
	/// - [`Syn::ForEachStat`]
	/// - [`Syn::LocalStat`]
	/// - [`Syn::MixinStat`]
	/// - [`Syn::ReturnStat`]
	/// - [`Syn::StaticConstStat`]
	/// - [`Syn::SwitchStat`]
	/// - [`Syn::UntilStat`]
	/// - [`Syn::WhileStat`]
	pub fn statement<'i>(&self) -> parser_t!(GreenNode) {
		chumsky::recursive::recursive(|stat| {
			let s_assign = primitive::group((
				comb::just_ts(Token::BracketL, Syn::BracketL),
				self.trivia_0plus(),
				self.expr_list(self.expr()),
				self.trivia_0plus(),
				comb::just_ts(Token::BracketR, Syn::BracketR),
				self.trivia_0plus(),
				comb::just_ts(Token::Eq, Syn::Eq),
				self.trivia_0plus(),
				self.expr(),
			))
			.map(|group| coalesce_node(group, Syn::AssignStat));

			let s_break = primitive::group((
				comb::just_ts(Token::KwBreak, Syn::KwBreak),
				self.trivia_0plus(),
				comb::just_ts(Token::Semicolon, Syn::Semicolon),
			))
			.map(|group| coalesce_node(group, Syn::BreakStat));

			let s_case = primitive::group((
				comb::just_ts(Token::KwCase, Syn::KwCase),
				self.trivia_1plus(),
				self.expr(),
				self.trivia_0plus(),
				comb::just_ts(Token::Colon, Syn::Colon),
			))
			.map(|group| coalesce_node(group, Syn::CaseStat));

			let s_compound = self.compound_stat(stat.clone());

			let s_condloop = primitive::group((
				primitive::choice((
					comb::just_ts(Token::KwWhile, Syn::KwWhile),
					comb::just_ts(Token::KwUntil, Syn::KwUntil),
				)),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenL, Syn::ParenL),
				self.trivia_0plus(),
				self.expr(),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenR, Syn::ParenR),
				stat.clone(),
			))
			.map(|group| {
				let syn = if group.0.kind() == Syn::KwWhile.into() {
					Syn::WhileStat
				} else if group.0.kind() == Syn::KwUntil.into() {
					Syn::UntilStat
				} else {
					unreachable!()
				};

				coalesce_node(group, syn)
			});

			let s_continue = primitive::group((
				comb::just_ts(Token::KwContinue, Syn::KwContinue),
				self.trivia_0plus(),
				comb::just_ts(Token::Semicolon, Syn::Semicolon),
			))
			.map(|group| coalesce_node(group, Syn::ContinueStat));

			let s_declassign = primitive::group((
				comb::just_ts(Token::KwLet, Syn::KwLet),
				self.trivia_0plus(),
				comb::just_ts(Token::BracketL, Syn::BracketL),
				self.trivia_0plus(),
				self.ident_list(),
				self.trivia_0plus(),
				comb::just_ts(Token::BracketR, Syn::BracketR),
				self.trivia_0plus(),
				comb::just_ts(Token::Eq, Syn::Eq),
				self.trivia_0plus(),
				self.expr(),
			))
			.map(|group| coalesce_node(group, Syn::DeclAssignStat));

			let s_default = primitive::group((
				comb::just_ts(Token::KwDefault, Syn::KwDefault),
				self.trivia_0plus(),
				comb::just_ts(Token::Colon, Syn::Colon),
			))
			.map(|group| coalesce_node(group, Syn::DefaultStat));

			let s_doloop = primitive::group((
				comb::just_ts(Token::KwDo, Syn::KwDo),
				self.trivia_1plus(),
				stat.clone(),
				self.trivia_0plus(),
				primitive::choice((
					comb::just_ts(Token::KwWhile, Syn::KwWhile),
					comb::just_ts(Token::KwUntil, Syn::KwUntil),
				)),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenL, Syn::ParenL),
				self.trivia_0plus(),
				self.expr(),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenR, Syn::ParenR),
			))
			.map(|group| {
				let syn = if group.4.kind() == Syn::KwWhile.into() {
					Syn::DoWhileStat
				} else if group.4.kind() == Syn::KwUntil.into() {
					Syn::DoUntilStat
				} else {
					unreachable!()
				};

				coalesce_node(group, syn)
			});

			let s_empty = comb::just_ts(Token::Semicolon, Syn::Semicolon)
				.map(|gtok| GreenNode::new(Syn::EmptyStat.into(), [gtok.into()]));

			let s_expr = primitive::group((
				self.expr(),
				self.trivia_0plus(),
				comb::just_ts(Token::Semicolon, Syn::Semicolon),
			))
			.map(|group| coalesce_node(group, Syn::ExprStat));

			let for_cond = self
				.expr()
				.map(|node| GreenNode::new(Syn::ForCond.into(), [node.into()]));

			let for_iter = self
				.expr()
				.map(|gnode| vec![gnode.into()])
				.foldl(
					primitive::group((
						self.trivia_0plus(),
						comb::just_ts(Token::Comma, Syn::Comma),
						self.trivia_0plus(),
						self.expr(),
					))
					.repeated(),
					|mut lhs, (mut t0, comma, mut t1, ident)| {
						lhs.append(&mut t0);
						lhs.push(comma.into());
						lhs.append(&mut t1);
						lhs.push(ident.into());
						lhs
					},
				)
				.map(|group| coalesce_node(group, Syn::ForIter));

			let for_init = primitive::choice((self.local_var(), for_iter.clone()))
				.map(|node| GreenNode::new(Syn::ForInit.into(), [node.into()]));

			let s_for = primitive::group((
				comb::just_ts(Token::KwFor, Syn::KwFor),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenL, Syn::ParenL),
				self.trivia_0plus(),
				// This is only grouped to shrink the emitted tuple, so as not
				// to have to macro-generate more impls than we already are.
				primitive::group((
					for_init,
					self.trivia_0plus(),
					comb::just_ts(Token::Semicolon, Syn::Semicolon),
					self.trivia_0plus(),
					for_cond.or_not(),
					self.trivia_0plus(),
					comb::just_ts(Token::Semicolon, Syn::Semicolon),
					self.trivia_0plus(),
					for_iter.or_not(),
				)),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenR, Syn::ParenR),
				self.trivia_0plus(),
				stat.clone(),
			))
			.map(|group| coalesce_node(group, Syn::ForStat));

			let s_foreach = primitive::group((
				comb::just_ts(Token::KwForeach, Syn::KwForEach),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenL, Syn::ParenL),
				self.trivia_0plus(),
				self.var_name(),
				self.trivia_0plus(),
				comb::just_ts(Token::Colon, Syn::Colon),
				self.trivia_0plus(),
				self.expr(),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenR, Syn::ParenR),
				self.trivia_0plus(),
				stat.clone(),
			))
			.map(|group| coalesce_node(group, Syn::ForEachStat));

			let s_local = primitive::group((
				self.local_var(),
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
			))
			.map(|group| coalesce_node(group, Syn::LocalStat));

			let s_ret = primitive::group((
				comb::just_ts(Token::KwReturn, Syn::KwReturn),
				primitive::group((self.trivia_1plus(), self.expr_list(self.expr()))).or_not(),
				self.trivia_0plus(),
				comb::just_ts(Token::Semicolon, Syn::Semicolon),
			))
			.map(|group| coalesce_node(group, Syn::ReturnStat));

			let s_switch = primitive::group((
				comb::just_ts(Token::KwSwitch, Syn::KwSwitch),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenL, Syn::ParenL),
				self.trivia_0plus(),
				self.expr(),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenR, Syn::ParenR),
				self.trivia_0plus(),
				stat,
			))
			.map(|group| coalesce_node(group, Syn::SwitchStat));

			primitive::choice((
				s_empty,
				s_case,
				s_default,
				s_compound,
				s_expr,
				s_switch,
				s_condloop,
				s_doloop,
				s_for,
				s_foreach,
				s_continue,
				s_break,
				s_ret,
				s_assign,
				s_declassign,
				s_local,
				self.static_array(),
			))
			.boxed()
		})
	}

	/// The returned parser emits a [`Syn::CompoundStat`] node. The return value
	/// of [`Self::statement`] must be passed in to prevent infinite recursion.
	pub(super) fn compound_stat<'i>(&self, stat: parser_t!(GreenNode)) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::BraceL, Syn::BraceL),
			self.trivia_0plus(),
			stat.repeated().collect::<Vec<_>>(),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::CompoundStat))
	}

	fn local_var<'i>(&self) -> parser_t!(GreenNode) {
		let ident_only = self
			.ident()
			.map(|gtok| GreenNode::new(Syn::LocalVarInit.into(), [gtok.into()]));

		let ident_arraylen =
			primitive::group((self.ident(), self.trivia_0plus(), self.array_len()))
				.map(|group| coalesce_node(group, Syn::LocalVarInit));

		let ident_eq_expr = primitive::group((
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::Eq, Syn::Eq),
			self.trivia_0plus(),
			self.expr(),
		))
		.map(|group| coalesce_node(group, Syn::LocalVarInit));

		let ident_braced_exprs = primitive::group((
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL),
			self.trivia_0plus(),
			self.expr_list(self.expr()),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::LocalVarInit));

		let ident_eq_braced_exprs = primitive::group((
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::Eq, Syn::Eq),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL),
			self.trivia_0plus(),
			self.expr_list(self.expr()),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::LocalVarInit));

		let var_init = primitive::choice((
			ident_only,
			ident_arraylen,
			ident_eq_expr,
			ident_braced_exprs,
			ident_eq_braced_exprs,
		));

		let var_list_with_init = var_init
			.clone()
			.map(|gnode| vec![GreenElement::from(gnode)])
			.foldl(
				primitive::group((
					self.trivia_0plus(),
					comb::just_ts(Token::Comma, Syn::Comma),
					self.trivia_0plus(),
					var_init,
				))
				.repeated(),
				|mut lhs, (mut t0, comma, mut t1, v_init)| {
					lhs.append(&mut t0);
					lhs.push(comma.into());
					lhs.append(&mut t1);
					lhs.push(v_init.into());
					lhs
				},
			);

		primitive::group((self.type_ref(), self.trivia_1plus(), var_list_with_init))
			.map(|group| coalesce_node(group, Syn::LocalVar))
	}

	fn static_array<'i>(&self) -> parser_t!(GreenNode) {
		let base = primitive::group((
			comb::just_ts(Token::KwStatic, Syn::KwStatic),
			self.trivia_1plus(),
			comb::just_ts(Token::KwConst, Syn::KwConst),
			self.trivia_1plus(),
			self.type_ref(),
		));

		let ident_brackets = primitive::group((
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::BracketL, Syn::BracketL),
			self.trivia_0plus(),
			comb::just_ts(Token::BracketR, Syn::BracketR),
		))
		.map(coalesce_vec);

		let brackets_ident = primitive::group((
			self.trivia_0plus(),
			comb::just_ts(Token::BracketL, Syn::BracketL),
			self.trivia_0plus(),
			comb::just_ts(Token::BracketR, Syn::BracketR),
			self.trivia_0plus(),
			self.ident(),
		))
		.map(coalesce_vec);

		let init = primitive::group((
			comb::just_ts(Token::Eq, Syn::Eq),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceR),
			self.trivia_0plus(),
			self.expr_list(self.expr()),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
			self.trivia_0plus(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon),
		));

		primitive::group((
			base,
			self.trivia_1plus(),
			primitive::choice((ident_brackets, brackets_ident)),
			init,
		))
		.map(|group| coalesce_node(group, Syn::StaticConstStat))
	}
}
