//! Statement parsers.

use crate::zdoom::{zscript::Syn, Token};

use super::{common::*, expr};

/// Builds a node tagged with one of the following:
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
pub fn statement(p: &mut crate::parser::Parser<Syn>) {
	let token = p.nth(0);

	if expr::in_first_set(token) {
		let peeked = p.lookahead_filtered(|token| !token.is_trivia());

		if in_type_ref_first_set(token) && is_ident_lax(peeked) {
			assign_stat(p);
			return;
		}

		let stat = p.open();
		expr(p);
		p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
		p.close(stat, Syn::ExprStat);
		return;
	}

	if in_type_ref_first_set(token) {
		assign_stat(p);
		return;
	}

	match token {
		Token::Semicolon => {
			let stat = p.open();
			p.advance(Syn::Semicolon);
			p.close(stat, Syn::EmptyStat);
		}
		Token::KwCase => {
			let stat = p.open();
			p.advance(Syn::KwCase);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::Colon, Syn::Colon, &["`:`"]);
			p.close(stat, Syn::CaseStat);
		}
		Token::KwDefault => {
			let stat = p.open();
			p.advance(Syn::KwDefault);
			trivia_0plus(p);
			p.expect(Token::Colon, Syn::Colon, &["`:`"]);
			p.close(stat, Syn::DefaultStat);
		}
		Token::BraceL => {
			compound_stat(p);
		}
		Token::KwSwitch => {
			let stat = p.open();
			p.advance(Syn::KwSwitch);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
			trivia_0plus(p);
			statement(p);
			p.close(stat, Syn::SwitchStat);
		}
		Token::KwIf => {
			let stat = p.open();
			p.advance(Syn::KwIf);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
			trivia_0plus(p);
			statement(p);

			if p.next_filtered(|token| !token.is_trivia()) == Token::KwElse {
				trivia_0plus(p);
				p.advance(Syn::KwElse);
				trivia_0plus(p);
				statement(p);
			}

			p.close(stat, Syn::IfStat);
		}
		t @ (Token::KwWhile | Token::KwUntil) => {
			let stat = p.open();
			p.advance(if t == Token::KwWhile {
				Syn::KwWhile
			} else {
				Syn::KwUntil
			});
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
			trivia_0plus(p);
			statement(p);
			p.close(
				stat,
				if t == Token::KwWhile {
					Syn::WhileStat
				} else {
					Syn::UntilStat
				},
			);
		}
		Token::KwDo => {
			let stat = p.open();
			p.advance(Syn::KwDo);
			trivia_0plus(p);
			statement(p);
			trivia_0plus(p);

			let node_syn = match p.nth(0) {
				Token::KwWhile => {
					p.advance(Syn::KwWhile);
					Syn::DoWhileStat
				}
				Token::KwUntil => {
					p.advance(Syn::KwUntil);
					Syn::DoUntilStat
				}
				t => {
					p.advance_err_and_close(
						stat,
						Syn::from(t),
						Syn::Error,
						&["`until`", "`while`"],
					);
					return;
				}
			};

			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);

			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);

			p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
			p.close(stat, node_syn);
		}
		Token::KwFor => {
			let stat = p.open();
			p.advance(Syn::KwFor);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
			trivia_0plus(p);

			// Initializers ////////////////////////////////////////////////////

			while !p.at(Token::Semicolon) && !p.eof() {
				let t = p.nth(0);

				if expr::in_first_set(t) {
					let peeked = p.lookahead_filtered(|token| !token.is_trivia());

					if in_type_ref_first_set(t) && is_ident_lax(peeked) {
						local_var(p);
						continue;
					}

					expr(p);
				} else if in_type_ref_first_set(t) {
					local_var(p);
				} else {
					p.advance_with_error(Syn::from(t), &["an expression", "a local variable"]);
				}

				if p.next_filtered(|token| !token.is_trivia()) == Token::Comma {
					trivia_0plus(p);
					p.advance(Syn::Comma);
					trivia_0plus(p);
				} else {
					break;
				}
			}

			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
			trivia_0plus(p);

			// Condition ///////////////////////////////////////////////////////

			if p.at_if(expr::in_first_set) {
				expr(p);
			}

			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
			trivia_0plus(p);

			// "Bump" //////////////////////////////////////////////////////////

			while !p.at(Token::ParenR) && !p.eof() {
				let t = p.nth(0);

				if expr::in_first_set(t) {
					expr(p);
				} else {
					p.advance_with_error(Syn::from(t), &["an expression"]);
				}

				if p.next_filtered(|token| !token.is_trivia()) == Token::Comma {
					trivia_0plus(p);
					p.advance(Syn::Comma);
					trivia_0plus(p);
				} else {
					break;
				}
			}

			trivia_0plus(p);
			p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
			trivia_0plus(p);
			statement(p);
			p.close(stat, Syn::ForStat);
		}
		Token::KwForEach => {
			let stat = p.open();
			p.advance(Syn::KwForEach);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);

			trivia_0plus(p);
			var_name(p);
			trivia_0plus(p);

			p.expect(Token::Colon, Syn::Colon, &["`:`"]);

			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);

			p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
			trivia_0plus(p);
			statement(p);
			p.close(stat, Syn::ForEachStat);
		}
		Token::KwContinue => {
			let stat = p.open();
			p.advance(Syn::KwContinue);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
			p.close(stat, Syn::ContinueStat);
		}
		Token::KwBreak => {
			let stat = p.open();
			p.advance(Syn::KwBreak);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
			p.close(stat, Syn::BreakStat);
		}
		Token::KwReturn => {
			let stat = p.open();
			p.advance(Syn::KwReturn);
			trivia_0plus(p);

			if expr::in_first_set(p.nth(0)) {
				expr::expr_list(p);
			}

			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
			p.close(stat, Syn::ReturnStat);
		}
		Token::BracketL => {
			let stat = p.open();
			p.advance(Syn::BracketL);
			trivia_0plus(p);
			expr::expr_list(p);
			trivia_0plus(p);
			p.expect(Token::BracketR, Syn::BracketR, &["`]`"]);
			trivia_0plus(p);
			p.expect(Token::Eq, Syn::Eq, &["`=`"]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
			p.close(stat, Syn::AssignStat);
		}
		Token::KwStatic => {
			static_const_stat(p);
		}
		other => {
			p.advance_with_error(
				Syn::from(other),
				&[
					"`;`",
					"`case`",
					"`default`",
					"`{`",
					"`switch`",
					"`while`",
					"`until`",
					"`do`",
					"`for`",
					"`foreach`",
					"`continue`",
					"`break`",
					"`return`",
					"`[`",
					"`let` or a type name",
					"`static`",
				],
			);
		}
	}
}

/// Builds a [`Syn::CompoundStat`] node.
pub(super) fn compound_stat(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::BraceL);
	let stat = p.open();
	p.advance(Syn::BraceL);

	loop {
		trivia_0plus(p);

		if p.at(Token::BraceR) || p.eof() {
			break;
		}

		statement(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`", "a statement"]);
	p.close(stat, Syn::CompoundStat);
}

/// Builds a [`Syn::StaticConstStat`] node.
pub(super) fn static_const_stat(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at_any(&[Token::KwStatic, Token::DocComment]);
	let stat = p.open();
	doc_comments(p);
	trivia_0plus(p);
	p.debug_assert_at(Token::KwStatic);
	p.advance(Syn::KwStatic);
	trivia_1plus(p);
	p.expect(Token::KwConst, Syn::KwConst, &["`const`"]);
	trivia_0plus(p);
	type_ref(p);
	trivia_0plus(p);

	let t = p.next_filtered(|token| !token.is_trivia());

	if is_ident(t) {
		trivia_0plus(p);
		ident(p);
		trivia_0plus(p);
		p.expect(Token::BracketL, Syn::BracketL, &["`[`"]);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syn::BracketR, &["`]`"]);
	} else if t == Token::BracketL {
		trivia_0plus(p);
		p.advance(Syn::BracketL);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syn::BracketR, &["`]`"]);
		trivia_0plus(p);
		ident(p);
	} else {
		p.advance_err_and_close(stat, Syn::from(t), Syn::Error, &["`[`", "an identifier"]);
		return;
	}

	trivia_0plus(p);
	p.expect(Token::Eq, Syn::Eq, &["`=`"]);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);
	trivia_0plus(p);
	expr::expr_list(p);
	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	p.close(stat, Syn::StaticConstStat);
}

/// Builds a [`Syn::DeclAssignStat`] or [`Syn::LocalStat`] node.
fn assign_stat(p: &mut crate::parser::Parser<Syn>) {
	let stat = p.open();

	let syn = if p.at(Token::KwLet)
		&& p.lookahead_filtered(|token| !token.is_trivia()) == Token::BracketL
	{
		p.advance(Syn::KwLet);
		trivia_0plus(p);
		p.advance(Syn::BracketL);
		trivia_0plus(p);
		ident_list(p);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syn::BracketR, &["`]`"]);
		trivia_0plus(p);
		p.expect(Token::Eq, Syn::Eq, &["`=`"]);
		trivia_0plus(p);
		expr(p);
		trivia_0plus(p);
		Syn::DeclAssignStat
	} else {
		local_var(p);
		Syn::LocalStat
	};

	p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	p.close(stat, syn);
}

/// Builds a [`Syn::LocalVar`] node. Does not expect a trailing semicolon.
fn local_var(p: &mut crate::parser::Parser<Syn>) {
	let local = p.open();
	type_ref(p);
	trivia_0plus(p);

	while p.at_if(is_ident_lax) {
		local_var_init(p);

		if p.next_filtered(|token| !token.is_trivia()) != Token::Comma {
			break;
		} else {
			trivia_0plus(p);
			p.advance(Syn::Comma);
			trivia_0plus(p);
		}
	}

	p.close(local, Syn::LocalVar);
}

/// Builds a [`Syn::LocalVarInit`] node.
fn local_var_init(p: &mut crate::parser::Parser<Syn>) {
	let init = p.open();
	ident_lax(p);
	trivia_0plus(p);

	match p.nth(0) {
		Token::BracketL => {
			trivia_0plus(p);
			array_len(p);
		}
		Token::Eq => {
			p.advance(Syn::Eq);
			trivia_0plus(p);

			if p.at(Token::BraceL) {
				p.advance(Syn::BraceL);
				trivia_0plus(p);
				expr::expr_list(p);
				trivia_0plus(p);
				p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);
			} else {
				expr(p);
			}
		}
		Token::BraceL => {
			p.advance(Syn::BraceL);
			trivia_0plus(p);
			expr::expr_list(p);
			trivia_0plus(p);
			p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);
		}
		Token::Semicolon | Token::Comma => {} // No initializer; valid.
		other => {
			p.advance_err_and_close(
				init,
				Syn::from(other),
				Syn::LocalVarInit,
				&["`[`", "`=`", "`{`"],
			);
			return;
		}
	}

	p.close(init, Syn::LocalVarInit);
}

#[cfg(test)]
mod test {
	use super::*;

	use crate::{
		testing::*,
		zdoom::{self, zscript::ParseTree},
	};

	#[test]
	fn smoke_for_loop() {
		const SOURCES: &[&str] = &[
			r#"for (;;) {}"#,
			r#"for (int i = 0;;) {}"#,
			r#"for (;i < arr.len();) {}"#,
			r#"for (;;++i) {}"#,
			r#"for ( int i = 0 ; i < arr.len() ; ++i) {}"#,
		];

		for source in SOURCES {
			let ptree: ParseTree =
				crate::parse(source, statement, zdoom::lex::Context::ZSCRIPT_LATEST);
			assert_no_errors(&ptree);
			prettyprint_maybe(ptree.cursor());
		}
	}

	#[test]
	fn smoke_if() {
		const SOURCE: &str = r"if(player_data ) {
			uint press =	  GetPlayerInput(INPUT_BUTTONS) &
							(~GetPlayerInput(INPUT_OLDBUTTONS));

					if(press & BT_USER1)	player_data.Binds.Use(0);
			else	if(press & BT_USER2)	player_data.Binds.Use(1);
			else	if(press & BT_USER3)	player_data.Binds.Use(2);
			else	if(press & BT_USER4)	player_data.Binds.Use(3);
		}";

		let ptree: ParseTree = crate::parse(SOURCE, statement, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}
