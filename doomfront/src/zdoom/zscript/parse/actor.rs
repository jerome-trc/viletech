//! Parsers for parts of definitions for classes inheriting from `Actor`.

use chumsky::{primitive, IterParser, Parser};
use rowan::{GreenNode, GreenToken};

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{
		zscript::{parse::*, Syn},
		Token,
	},
	GreenElement, _ParseState,
};

use super::ParserBuilder;

impl ParserBuilder {
	/// The returned parser emits a [`Syn::FlagDef`] node.
	pub fn flag_def<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwFlagDef, Syn::KwFlagDef),
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
		.map_with_state(|_, span: logos::Span, state: &mut _ParseState| {
			GreenToken::new(Syn::StateSprite.into(), &state.source[span])
		});

		let sel_hold = comb::just_ts(Token::Pound, Syn::Pound)
			.repeated()
			.exactly(4)
			.map_with_state(|_, span: logos::Span, state: &mut _ParseState| {
				GreenToken::new(Syn::StateSprite.into(), &state.source[span])
			});

		let hold = comb::just_ts(Token::Minus2, Syn::Minus2)
			.repeated()
			.exactly(2)
			.map_with_state(|_, span: logos::Span, state: &mut _ParseState| {
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
			.map_with_state(|_, span, state: &mut _ParseState| {
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

/// Builds a [`Syn::FlagDef`] node.
pub fn flag_def(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::KwFlagDef);
	let flagdef = p.open();
	p.advance(Syn::KwFlagDef);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::Colon, Syn::Colon, &["`:`"]);
	trivia_0plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::Comma, Syn::Comma, &["`,`"]);
	trivia_0plus(p);
	p.expect(Token::IntLit, Syn::IntLit, &["an integer"]);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	p.close(flagdef, Syn::FlagDef);
}

/// Builds a [`Syn::PropertyDef`] node.
pub fn property_def(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::KwProperty);
	let propdef = p.open();
	p.advance(Syn::KwProperty);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::Colon, Syn::Colon, &["`:`"]);
	trivia_0plus(p);
	ident_list(p);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	p.close(propdef, Syn::PropertyDef);
}

/// Builds a [`Syn::DefaultBlock`] node.
pub fn default_block(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::KwDefault);
	let defblock = p.open();
	p.advance(Syn::KwDefault);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		let token = p.nth(0);

		if is_ident(token) {
			property_setting(p);
		} else if matches!(token, Token::Plus | Token::Minus) {
			flag_setting(p);
		} else {
			p.advance_with_error(Syn::from(token), &["`+` or `-`", "an identifier"]);
		}

		trivia_0plus(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);
	p.close(defblock, Syn::DefaultBlock);
}

/// Builds a [`Syn::FlagSetting`] node.
fn flag_setting(p: &mut crate::parser::Parser<Syn>) {
	debug_assert!(p.at_any(&[Token::Plus, Token::Minus]));
	let flag = p.open();

	p.advance(match p.nth(0) {
		Token::Plus => Syn::Plus,
		Token::Minus => Syn::Minus,
		_ => unreachable!(),
	});

	trivia_0plus(p);
	ident_chain_lax(p);

	if p.next_filtered(|token| !token.is_trivia()) == Token::Semicolon {
		trivia_0plus(p);
		p.advance(Syn::Semicolon);
	}

	p.close(flag, Syn::FlagSetting);
}

/// Builds a [`Syn::PropertySetting`] node.
fn property_setting(p: &mut crate::parser::Parser<Syn>) {
	debug_assert!(p.at_if(is_ident));
	let prop = p.open();
	ident_chain_lax(p);
	trivia_0plus(p);

	if !p.at(Token::Semicolon) {
		expr_list(p);
	}

	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	p.close(prop, Syn::PropertySetting);
}

/// Builds a [`Syn::StatesBlock`] node.
pub fn states_block(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::KwStates);
	let sblock = p.open();
	p.advance(Syn::KwStates);
	trivia_0plus(p);

	if p.at(Token::ParenL) {
		states_usage(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);

	while !p.at(Token::BraceR) && !p.eof() {
		trivia_0plus(p);

		let token = p.nth(0);

		if is_ident_lax(token) {
			let peeked = p.next_filtered(|token| !token.is_trivia() && !is_ident_lax(token));

			if matches!(peeked, Token::Colon | Token::Dot) {
				let label = p.open();
				ident_chain(p);
				trivia_0plus(p);
				p.advance(Syn::Colon);
				p.close(label, Syn::StateLabel);
			} else if p.current_slice().len() != 4 {
				p.advance_with_error(
					Syn::Ident,
					&["exactly 4 ASCII characters", "`\"####\"`", "`\"----\"`"],
				);
			} else {
				state_def(p);
			}

			trivia_0plus(p);
			continue;
		}

		match token {
			Token::StringLit => {
				if p.current_slice().len() != 6 {
					p.advance_with_error(
						Syn::StringLit,
						&["exactly 4 ASCII characters", "`\"####\"`", "`\"----\"`"],
					);
				} else {
					state_def(p);
				}
			}
			Token::IntLit => {
				if p.current_slice().len() != 4 {
					p.advance_with_error(
						Syn::IntLit,
						&["exactly 4 ASCII characters", "`\"####\"`", "`\"----\"`"],
					);
				} else {
					state_def(p);
				}
			}
			Token::KwFail => {
				let flow = p.open();
				p.advance(Syn::KwFail);
				trivia_0plus(p);
				p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
				p.close(flow, Syn::StateFlow);
			}
			Token::KwStop => {
				let flow = p.open();
				p.advance(Syn::KwStop);
				trivia_0plus(p);
				p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
				p.close(flow, Syn::StateFlow);
			}
			Token::KwLoop => {
				let flow = p.open();
				p.advance(Syn::KwLoop);
				trivia_0plus(p);
				p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
				p.close(flow, Syn::StateFlow);
			}
			Token::KwWait => {
				let flow = p.open();
				p.advance(Syn::KwWait);
				trivia_0plus(p);
				p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
				p.close(flow, Syn::StateFlow);
			}
			Token::KwGoto => {
				let flow = p.open();
				p.advance(Syn::KwGoto);
				trivia_0plus(p);

				if p.eat(Token::KwSuper, Syn::KwSuper) {
					trivia_0plus(p);
					p.expect(Token::Colon2, Syn::Colon2, &["`::`"]);
					trivia_0plus(p);
					ident_chain_lax(p);
				} else if p.at_if(is_ident_lax) {
					let peeked =
						p.next_filtered(|token| !token.is_trivia() && !is_ident_lax(token));

					match peeked {
						Token::Dot => ident_chain(p),
						Token::Colon2 => {
							p.advance(Syn::Ident);
							trivia_0plus(p);
							p.advance(Syn::Colon2);
							trivia_0plus(p);
							ident_chain(p);
						}
						Token::Semicolon | Token::Plus | Token::Eof => {
							p.advance(Syn::Ident);
						}
						other => p.advance_with_error(
							Syn::from(other),
							&["an identifier", "`.`", "`::`", "`+`", "`;`"],
						),
					}
				} else {
					p.advance_with_error(Syn::from(p.nth(0)), &["an identifier", "`super`"]);
				}

				if p.next_filtered(|token| !token.is_trivia()) == Token::Plus {
					trivia_0plus(p);
					p.advance(Syn::Plus);
					trivia_0plus(p);
					p.expect(Token::IntLit, Syn::IntLit, &["an integer"]);
				}

				trivia_0plus(p);
				p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
				p.close(flow, Syn::StateFlow);
			}
			other => p.advance_with_error(
				Syn::from(other),
				&[
					"`goto`",
					"`fail`",
					"`loop`",
					"`stop`",
					"`wait`",
					"a state label",
					"a state sprite",
				],
			),
		}

		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);
	p.close(sblock, Syn::StatesBlock);
}

/// Builds a [`Syn::StatesUsage`] node.
pub fn states_usage(p: &mut crate::parser::Parser<Syn>) {
	fn kw(p: &mut crate::parser::Parser<Syn>) {
		p.expect_any_str_nc(
			&[
				(Token::Ident, "actor", Syn::Ident),
				(Token::Ident, "item", Syn::Ident),
				(Token::Ident, "overlay", Syn::Ident),
				(Token::Ident, "weapon", Syn::Ident),
			],
			&["`actor`", "`item`", "`overlay`", "`weapon`"],
		);
	}

	p.debug_assert_at(Token::ParenL);
	let usage = p.open();
	p.advance(Syn::ParenL);
	trivia_0plus(p);
	kw(p);
	trivia_0plus(p);

	while !p.at(Token::ParenR) && !p.eof() {
		if p.at(Token::Comma) {
			trivia_0plus(p);
			kw(p);
		}
	}

	trivia_0plus(p);
	p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
	p.close(usage, Syn::StatesUsage);
}

/// Builds a [`Syn::StateDef`] node.
pub fn state_def(p: &mut crate::parser::Parser<Syn>) {
	#[cfg(debug_assertions)]
	if !is_ident_lax(p.nth(0)) && !matches!(p.nth(0), Token::StringLit | Token::IntLit) {
		panic!();
	}

	let state = p.open();
	p.advance(Syn::StateSprite);
	trivia_0plus(p);
	state_frames(p);
	trivia_0plus(p);
	expr(p);
	trivia_0plus(p);

	loop {
		if p.eof() {
			break;
		}

		if p.at_any(&[Token::Semicolon, Token::BraceL, Token::Ident]) {
			break;
		}

		match p.nth(0) {
			Token::KwLight => {
				let light = p.open();
				p.advance(Syn::KwLight);
				trivia_0plus(p);
				p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
				trivia_0plus(p);

				p.expect_any(
					&[
						(Token::StringLit, Syn::StringLit),
						(Token::NameLit, Syn::NameLit),
					],
					&["a string", "a name"],
				);

				trivia_0plus(p);

				while !p.at(Token::ParenR) && !p.eof() {
					p.expect(Token::Comma, Syn::Comma, &["`,`"]);
					trivia_0plus(p);
					p.expect_any(
						&[
							(Token::StringLit, Syn::StringLit),
							(Token::NameLit, Syn::NameLit),
						],
						&["a string", "a name"],
					);
					trivia_0plus(p);
				}

				trivia_0plus(p);
				p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
				p.close(light, Syn::StateLight);
			}
			Token::KwOffset => {
				let offset = p.open();
				p.advance(Syn::KwOffset);
				trivia_0plus(p);
				p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
				trivia_0plus(p);
				expr(p);
				trivia_0plus(p);
				p.expect(Token::Comma, Syn::Comma, &["`,`"]);
				trivia_0plus(p);
				expr(p);
				trivia_0plus(p);
				p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
				p.close(offset, Syn::StateOffset);
			}
			t @ (Token::KwBright
			| Token::KwCanRaise
			| Token::KwFast
			| Token::KwSlow
			| Token::KwNoDelay) => p.advance(Syn::from(t)),
			other => p.advance_with_error(
				Syn::from(other),
				&[
					"`bright`",
					"`canraise`",
					"`fast`",
					"`light`",
					"`nodelay`",
					"`offset`",
					"`slow`",
				],
			),
		}

		trivia_0plus(p);
	}

	trivia_0plus(p);

	if p.at(Token::Semicolon) {
		p.advance(Syn::Semicolon);
	} else {
		action_function(p);
	}

	p.close(state, Syn::StateDef);
}

fn state_frames(p: &mut crate::parser::Parser<Syn>) {
	let token = p.nth(0);

	if is_ident_lax(token) {
		p.advance(Syn::StateFrames);
		return;
	}

	if token == Token::StringLit {
		p.advance(Syn::StateFrames);
		return;
	}

	let mut n = 0;

	while !p.at_if(Token::is_trivia) {
		if !matches!(
			p.nth(n),
			Token::Ident | Token::IntLit | Token::BracketL | Token::BracketR | Token::Backslash
		) {
			break;
		}

		n += 1;
	}

	p.advance_n(Syn::StateFrames, n);
}

/// Builds a [`Syn::ActionFunction`] node.
fn action_function(p: &mut crate::parser::Parser<Syn>) {
	if p.at(Token::BraceL) {
		compound_stat(p);
	} else {
		ident(p);
		trivia_0plus(p);

		if p.at(Token::ParenL) {
			expr::arg_list(p);
		}

		trivia_0plus(p);
		p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	}
}

#[cfg(test)]
mod test {
	use crate::{
		testing::*,
		zdoom::{self, zscript::ParseTree},
	};

	use super::*;

	#[test]
	fn smoke_state_def() {
		const SOURCES: &[&str] = &["#### # 1;", "---- # 1;", "\"####\" \"#\" 1;"];

		for source in SOURCES {
			let ptree: ParseTree = crate::parse(source, state_def, zdoom::Version::default());
			assert_no_errors(&ptree);
			prettyprint_maybe(ptree.cursor());
		}
	}

	#[test]
	fn smoke_goto() {
		const SOURCE: &str = r#####"States {
	goto Super::FrameSetup;
}"#####;

		let ptree: ParseTree = crate::parse(SOURCE, states_block, zdoom::Version::default());
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}
