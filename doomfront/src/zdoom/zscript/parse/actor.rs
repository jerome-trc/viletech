//! Parsers for parts of definitions for classes inheriting from `Actor`.

use chumsky::{primitive, IterParser, Parser};
use rowan::{GreenNode, GreenToken};

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{zscript::Syn, Token},
	GreenElement, ParseState,
};

use super::ParserBuilder;

impl ParserBuilder {
	/// The returned parser emits a [`Syn::FlagDef`] node.
	pub fn flag_def<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwFlagdef, Syn::KwFlagdef),
			self.trivia_1plus(),
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::Colon, Syn::Colon),
			self.trivia_0plus(),
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma),
			self.trivia_0plus(),
			comb::just_ts(Token::IntLit, Syn::IntLit),
			self.trivia_0plus(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon),
		))
		.map(|group| coalesce_node(group, Syn::FlagDef))
	}

	/// The returned parser emits a [`Syn::PropertyDef`] node.
	pub fn property_def<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwProperty, Syn::KwProperty),
			self.trivia_1plus(),
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::Colon, Syn::Colon),
			self.trivia_0plus(),
			self.ident_list(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon),
		))
		.map(|group| coalesce_node(group, Syn::PropertyDef))
	}

	// Default blocks //////////////////////////////////////////////////////////////

	/// The returned parser emits a [`Syn::DefaultBlock`] node.
	pub fn default_block<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwDefault, Syn::KwDefault),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL),
			primitive::choice((
				self.property_setting().map(GreenElement::from),
				self.flag_setting().map(GreenElement::from),
				self.trivia(),
			))
			.repeated()
			.collect::<Vec<_>>(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::DefaultBlock))
	}

	/// The returned parser emits a [`Syn::FlagSetting`] node.
	fn flag_setting<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			primitive::choice((
				comb::just_ts(Token::Plus, Syn::Plus),
				comb::just_ts(Token::Minus, Syn::Minus),
			)),
			self.trivia_0plus(),
			self.ident_chain(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon).or_not(),
		))
		.map(|group| coalesce_node(group, Syn::FlagSetting))
	}

	/// The returned parser emits a [`Syn::PropertySetting`] node.
	fn property_setting<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			self.ident_chain(),
			self.trivia_1plus(),
			self.expr_list(self.expr()),
			comb::just_ts(Token::Semicolon, Syn::Semicolon),
		))
		.map(|group| coalesce_node(group, Syn::PropertySetting))
	}

	// State machine definitions ///////////////////////////////////////////////////

	/// The returned parser emits a [`Syn::StatesBlock`] node.
	pub fn states_block<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwStates, Syn::KwStates),
			self.trivia_0plus(),
			self.states_usage().or_not(),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL),
			primitive::choice((
				self.trivia(),
				self.state_label().map(GreenElement::from),
				self.state_flow().map(GreenElement::from),
				self.state_def().map(GreenElement::from),
			))
			.repeated()
			.collect::<Vec<_>>(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::StatesBlock))
		.boxed()
	}

	/// The returned parser emits a [`Syn::StatesUsage`] node.
	fn states_usage<'i>(&self) -> parser_t!(GreenNode) {
		let kw = primitive::choice((
			comb::string_nc(Token::Ident, "actor", Syn::Ident),
			comb::string_nc(Token::Ident, "item", Syn::Ident),
			comb::string_nc(Token::Ident, "overlay", Syn::Ident),
			comb::string_nc(Token::Ident, "weapon", Syn::Ident),
		));

		let rep = primitive::group((
			self.trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma),
			self.trivia_0plus(),
			kw.clone(),
		));

		primitive::group((
			comb::just_ts(Token::ParenL, Syn::ParenL),
			kw.map(|gtok| vec![GreenElement::from(gtok)]).foldl(
				rep.repeated(),
				|mut lhs, (mut t0, comma, mut t1, i)| {
					lhs.append(&mut t0);
					lhs.push(comma.into());
					lhs.append(&mut t1);
					lhs.push(i.into());
					lhs
				},
			),
			comb::just_ts(Token::ParenR, Syn::ParenR),
		))
		.map(|group| coalesce_node(group, Syn::StatesUsage))
	}

	/// The returned parser emits a [`Syn::StateLabel`] node.
	fn state_label<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::Colon, Syn::Colon),
		))
		.map(|group| coalesce_node(group, Syn::StateLabel))
	}

	/// The returned parser emits a [`Syn::StateFlow`] node.
	fn state_flow<'i>(&self) -> parser_t!(GreenNode) {
		let kw = primitive::group((
			primitive::choice((
				comb::just_ts(Token::KwFail, Syn::KwFail),
				comb::just_ts(Token::KwLoop, Syn::KwLoop),
				comb::just_ts(Token::KwStop, Syn::KwStop),
				comb::just_ts(Token::KwWait, Syn::KwWait),
			)),
			self.trivia_0plus(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon),
		))
		.map(|group| coalesce_node(group, Syn::StateFlow));

		let offset = primitive::group((
			self.trivia_0plus(),
			comb::just_ts(Token::Plus, Syn::Plus),
			self.trivia_0plus(),
			self.expr(),
		))
		.map(|group| coalesce_node(group, Syn::GotoOffset));

		let scope = primitive::group((
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::Colon2, Syn::Colon2),
			self.trivia_0plus(),
		));

		let goto = primitive::group((
			comb::just_ts(Token::KwGoto, Syn::KwGoto),
			self.trivia_1plus(),
			scope.or_not(),
			self.ident_chain(),
			offset.or_not(),
			self.trivia_0plus(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon),
		))
		.map(|group| coalesce_node(group, Syn::StateFlow));

		primitive::choice((kw, goto))
	}

	/// The returned parser emits a [`Syn::StateDef`] node.
	pub fn state_def<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			self.state_sprite(),
			self.trivia_0plus(),
			self.state_frames(),
			self.trivia_0plus(),
			self.expr(),
			self.state_quals().or_not(),
			self.trivia_0plus(),
			self.action_function(),
		))
		.map(|group| coalesce_node(group, Syn::StateDef))
	}

	/// The returned parser emits a [`Syn::StateSprite`] token.
	fn state_sprite<'i>(&self) -> parser_t!(GreenToken) {
		let alphanum = primitive::choice((
			primitive::just(Token::IntLit),
			primitive::just(Token::Ident),
		))
		.repeated()
		.at_least(1)
		.map_with_state(|_, span: logos::Span, state: &mut ParseState| {
			GreenToken::new(Syn::StateSprite.into(), &state.source[span])
		});

		let sel_hold = comb::just_ts(Token::Pound, Syn::Pound)
			.repeated()
			.exactly(4)
			.map_with_state(|_, span: logos::Span, state: &mut ParseState| {
				GreenToken::new(Syn::StateSprite.into(), &state.source[span])
			});

		let hold = comb::just_ts(Token::Minus2, Syn::Minus2)
			.repeated()
			.exactly(2)
			.map_with_state(|_, span: logos::Span, state: &mut ParseState| {
				GreenToken::new(Syn::StateSprite.into(), &state.source[span])
			});

		let sel_hold_quoted = comb::string_nc(Token::StringLit, "\"####\"", Syn::StateSprite);

		let hold_quoted = comb::string_nc(Token::StringLit, "\"----\"", Syn::StateSprite);

		primitive::choice((hold, sel_hold, hold_quoted, sel_hold_quoted, alphanum))
	}

	/// The returned parser emits a [`Syn::StateFrames`] token.
	fn state_frames<'i>(&self) -> parser_t!(GreenToken) {
		let unquoted = primitive::any()
			.filter(|token: &Token| {
				if token.is_keyword() {
					return true;
				}

				if matches!(
					token,
					Token::Unknown | Token::Ident | Token::BracketL | Token::BracketR
				) {
					return true;
				}

				false
			})
			.repeated()
			.at_least(1)
			.map_with_state(|_, span, state: &mut ParseState| {
				GreenToken::new(Syn::StateFrames.into(), &state.source[span])
			});

		primitive::choice((
			unquoted,
			comb::just_ts(Token::Pound, Syn::StateFrames),
			comb::just_ts(Token::StringLit, Syn::StateFrames),
		))
	}

	fn state_quals<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		let light_spec = primitive::choice((
			comb::just_ts(Token::StringLit, Syn::StringLit),
			comb::just_ts(Token::NameLit, Syn::NameLit),
		));

		let light_rep = primitive::group((
			self.trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma),
			self.trivia_0plus(),
			light_spec.clone(),
		));

		let light_list = light_spec.map(|gtok| vec![GreenElement::from(gtok)]).foldl(
			light_rep.repeated(),
			|mut lhs, (mut t0, comma, mut t1, l)| {
				lhs.append(&mut t0);
				lhs.push(comma.into());
				lhs.append(&mut t1);
				lhs.push(l.into());
				lhs
			},
		);

		let light = primitive::group((
			comb::just_ts(Token::KwLight, Syn::KwLight),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenL, Syn::ParenL),
			self.trivia_0plus(),
			light_list,
			self.trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR),
		))
		.map(|group| coalesce_node(group, Syn::StateLight));

		let offset = primitive::group((
			comb::just_ts(Token::KwOffset, Syn::KwOffset),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenL, Syn::ParenL),
			self.trivia_0plus(),
			self.expr(),
			self.trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma),
			self.trivia_0plus(),
			self.expr(),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR),
		))
		.map(|group| coalesce_node(group, Syn::StateOffset));

		let qual = primitive::choice((
			comb::just_ts(Token::KwBright, Syn::KwBright).map(GreenElement::from),
			comb::just_ts(Token::KwCanRaise, Syn::KwCanRaise).map(GreenElement::from),
			comb::just_ts(Token::KwFast, Syn::KwFast).map(GreenElement::from),
			comb::just_ts(Token::KwNoDelay, Syn::KwNoDelay).map(GreenElement::from),
			comb::just_ts(Token::KwSlow, Syn::KwSlow).map(GreenElement::from),
			light.map(GreenElement::from),
			offset.map(GreenElement::from),
		));

		primitive::group((self.trivia_1plus(), qual.clone()))
			.map(coalesce_vec)
			.foldl(
				primitive::group((self.trivia_1plus(), qual.clone())).repeated(),
				|mut lhs, (mut triv, q)| {
					lhs.append(&mut triv);
					lhs.push(q);
					lhs
				},
			)
	}

	/// The returned parser emits either a [`Syn::ActionFunction`] node or
	/// a [`Syn::Semicolon`] token.
	fn action_function<'i>(&self) -> parser_t!(GreenElement) {
		let empty_block = primitive::group((
			comb::just_ts(Token::BraceL, Syn::BraceL),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::ActionFunction));

		let anon = self
			.compound_stat(self.statement())
			.map(|gnode| GreenNode::new(Syn::ActionFunction.into(), [gnode.into()]));

		let call = primitive::group((
			self.ident(),
			self.trivia_0plus(),
			primitive::group((
				comb::just_ts(Token::ParenL, Syn::ParenL),
				self.trivia_0plus(),
				self.arg_list(self.expr()).or_not(),
				self.trivia_0plus(),
				comb::just_ts(Token::ParenR, Syn::ParenR),
			))
			.or_not(),
			self.trivia_0plus(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon),
		))
		.map(|group| coalesce_node(group, Syn::ActionFunction));

		primitive::choice((
			comb::just_ts(Token::Semicolon, Syn::Semicolon).map(GreenElement::from),
			anon.map(GreenElement::from),
			empty_block.map(GreenElement::from),
			call.map(GreenElement::from),
		))
	}
}

#[cfg(test)]
mod test {
	use crate::{
		testing::*,
		zdoom::{zscript::ParseTree, Version},
	};

	use super::*;

	#[test]
	fn smoke_state_def() {
		const SOURCES: &[&str] = &["#### # 1;", "---- # 1;", "\"####\" \"#\" 1;"];

		let builder = ParserBuilder::new(Version::default());

		for source in SOURCES {
			let tbuf = crate::scan(source, Version::default());
			let result = crate::parse(builder.state_def(), source, &tbuf);
			let ptree: ParseTree = unwrap_parse_tree(result);
			assert_no_errors(&ptree);
		}
	}
}
