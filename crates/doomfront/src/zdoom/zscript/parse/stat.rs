//! Statement parsers.

use crate::{
	parser::Parser,
	zdoom::{zscript::Syntax, Token},
};

use super::{common::*, expr, types::*};

/// Builds a node tagged with one of the following:
/// - [`Syntax::AssignStat`]
/// - [`Syntax::BreakStat`]
/// - [`Syntax::CaseStat`]
/// - [`Syntax::CompoundStat`]
/// - [`Syntax::ContinueStat`]
/// - [`Syntax::DeclAssignStat`]
/// - [`Syntax::DefaultStat`]
/// - [`Syntax::DoUntilStat`]
/// - [`Syntax::DoWhileStat`]
/// - [`Syntax::EmptyStat`]
/// - [`Syntax::ExprStat`]
/// - [`Syntax::ForStat`]
/// - [`Syntax::ForEachStat`]
/// - [`Syntax::LocalStat`]
/// - [`Syntax::ReturnStat`]
/// - [`Syntax::StaticConstStat`]
/// - [`Syntax::SwitchStat`]
/// - [`Syntax::UntilStat`]
/// - [`Syntax::WhileStat`]
pub fn statement(p: &mut Parser<Syntax>) {
	let token = p.nth(0);

	if expr::in_first_set(token) {
		let peeked = p.find(1, |token| !token.is_trivia());
		let in_tref_1set = in_type_ref_first_set(token);

		// `class` is not in the identifier fallback set, so if looking at
		// `array<class`, we know this isn't a less-than expression.
		if in_tref_1set && (is_ident_lax(peeked) || peeked == Token::KwClass) {
			declassign_or_local_stat(p);
			return;
		}

		// When faced with the code: `int array = 0; int uint = 0; array<uint> c;`,
		// GZDoom's parser resolves the ambiguity by trying to parse a generic
		// array type anyway. We do the same.
		if token == Token::KwArray && peeked == Token::AngleL {
			declassign_or_local_stat(p);
			return;
		}

		let stat = p.open();
		expr(p);
		p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
		p.close(stat, Syntax::ExprStat);
		return;
	}

	if in_type_ref_first_set(token) {
		declassign_or_local_stat(p);
		return;
	}

	match token {
		Token::Semicolon => {
			let stat = p.open();
			p.advance(Syntax::Semicolon);
			p.close(stat, Syntax::EmptyStat);
		}
		Token::KwCase => {
			let stat = p.open();
			p.advance(Syntax::KwCase);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::Colon, Syntax::Colon, &[&["`:`"]]);
			p.close(stat, Syntax::CaseStat);
		}
		Token::KwDefault => {
			let stat = p.open();
			p.advance(Syntax::KwDefault);
			trivia_0plus(p);
			p.expect(Token::Colon, Syntax::Colon, &[&["`:`"]]);
			p.close(stat, Syntax::DefaultStat);
		}
		Token::BraceL => {
			compound_stat(p);
		}
		Token::KwSwitch => {
			let stat = p.open();
			p.advance(Syntax::KwSwitch);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			statement(p);
			p.close(stat, Syntax::SwitchStat);
		}
		Token::KwIf => {
			let stat = p.open();
			p.advance(Syntax::KwIf);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			statement(p);

			if p.find(0, |token| !token.is_trivia()) == Token::KwElse {
				trivia_0plus(p);
				p.advance(Syntax::KwElse);
				trivia_0plus(p);
				statement(p);
			}

			p.close(stat, Syntax::IfStat);
		}
		t @ (Token::KwWhile | Token::KwUntil) => {
			let stat = p.open();
			p.advance(if t == Token::KwWhile {
				Syntax::KwWhile
			} else {
				Syntax::KwUntil
			});
			trivia_0plus(p);
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			statement(p);
			p.close(
				stat,
				if t == Token::KwWhile {
					Syntax::WhileStat
				} else {
					Syntax::UntilStat
				},
			);
		}
		Token::KwDo => {
			let stat = p.open();
			p.advance(Syntax::KwDo);
			trivia_0plus(p);
			statement(p);
			trivia_0plus(p);

			let node_syn = match p.nth(0) {
				Token::KwWhile => {
					p.advance(Syntax::KwWhile);
					Syntax::DoWhileStat
				}
				Token::KwUntil => {
					p.advance(Syntax::KwUntil);
					Syntax::DoUntilStat
				}
				t => {
					p.advance_err_and_close(
						stat,
						Syntax::from(t),
						Syntax::Error,
						&[&["`until`", "`while`"]],
					);
					return;
				}
			};

			trivia_0plus(p);
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);

			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);

			p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
			p.close(stat, node_syn);
		}
		Token::KwFor => {
			let stat = p.open();
			p.advance(Syntax::KwFor);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
			trivia_0plus(p);

			// Initializers ////////////////////////////////////////////////////

			let init = p.open();

			while !p.at(Token::Semicolon) && !p.eof() {
				let t = p.nth(0);

				if expr::in_first_set(t) {
					let peeked = p.find(1, |token| !token.is_trivia());

					if in_type_ref_first_set(t) && is_ident_lax(peeked) {
						local_var(p);
						continue;
					}

					expr(p);
				} else if in_type_ref_first_set(t) {
					local_var(p);
				} else {
					p.advance_with_error(
						Syntax::from(t),
						&[&["an expression", "a local variable"]],
					);
				}

				if p.find(0, |token| !token.is_trivia()) == Token::Comma {
					trivia_0plus(p);
					p.advance(Syntax::Comma);
					trivia_0plus(p);
				} else {
					break;
				}
			}

			trivia_0plus(p);
			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(init, Syntax::ForLoopInit);
			trivia_0plus(p);

			// Condition ///////////////////////////////////////////////////////

			let cond = p.open();

			if p.at_if(expr::in_first_set) {
				expr(p);
			}

			trivia_0plus(p);
			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(cond, Syntax::ForLoopCond);
			trivia_0plus(p);

			// "Bump" //////////////////////////////////////////////////////////

			let iter = p.open();

			while !p.at(Token::ParenR) && !p.eof() {
				let t = p.nth(0);

				if expr::in_first_set(t) {
					expr(p);
				} else {
					p.advance_with_error(Syntax::from(t), &[&["an expression"]]);
				}

				if p.find(0, |token| !token.is_trivia()) == Token::Comma {
					trivia_0plus(p);
					p.advance(Syntax::Comma);
					trivia_0plus(p);
				} else {
					break;
				}
			}

			p.close(iter, Syntax::ForLoopIter);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			statement(p);
			p.close(stat, Syntax::ForStat);
		}
		Token::KwForEach => {
			let stat = p.open();
			p.advance(Syntax::KwForEach);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);

			trivia_0plus(p);
			var_name(p);
			trivia_0plus(p);

			p.expect(Token::Colon, Syntax::Colon, &[&["`:`"]]);

			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);

			p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			statement(p);
			p.close(stat, Syntax::ForEachStat);
		}
		Token::KwContinue => {
			let stat = p.open();
			p.advance(Syntax::KwContinue);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(stat, Syntax::ContinueStat);
		}
		Token::KwBreak => {
			let stat = p.open();
			p.advance(Syntax::KwBreak);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(stat, Syntax::BreakStat);
		}
		Token::KwReturn => {
			let stat = p.open();
			p.advance(Syntax::KwReturn);
			trivia_0plus(p);

			if expr::in_first_set(p.nth(0)) {
				expr::expr_list(p);
			}

			trivia_0plus(p);
			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(stat, Syntax::ReturnStat);
		}
		Token::BracketL => {
			let stat = p.open();
			p.advance(Syntax::BracketL);
			trivia_0plus(p);
			expr::expr_list(p);
			trivia_0plus(p);
			p.expect(Token::BracketR, Syntax::BracketR, &[&["`]`"]]);
			trivia_0plus(p);
			p.expect(Token::Eq, Syntax::Eq, &[&["`=`"]]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(stat, Syntax::AssignStat);
		}
		Token::KwStatic => {
			static_const_stat(p);
		}
		other => {
			p.advance_with_error(
				Syntax::from(other),
				&[&[
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
				]],
			);
		}
	}
}

/// Builds a [`Syntax::CompoundStat`] node.
pub(super) fn compound_stat(p: &mut Parser<Syntax>) {
	let stat = p.open();
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);

	loop {
		trivia_0plus(p);

		if p.at(Token::BraceR) || p.eof() {
			break;
		}

		statement(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`", "a statement"]]);
	p.close(stat, Syntax::CompoundStat);
}

/// Builds a [`Syntax::StaticConstStat`] node.
pub(super) fn static_const_stat(p: &mut Parser<Syntax>) {
	p.debug_assert_at_any(&[Token::KwStatic, Token::DocComment]);
	let stat = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwStatic);
	p.advance(Syntax::KwStatic);
	trivia_1plus(p);
	p.expect(Token::KwConst, Syntax::KwConst, &[&["`const`"]]);
	trivia_0plus(p);
	core_type(p);
	trivia_0plus(p);

	let t = p.find(0, |token| !token.is_trivia());

	if is_ident_lax(t) {
		trivia_0plus(p);
		ident_lax(p);
		trivia_0plus(p);
		p.expect(Token::BracketL, Syntax::BracketL, &[&["`[`"]]);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syntax::BracketR, &[&["`]`"]]);
	} else if t == Token::BracketL {
		trivia_0plus(p);
		p.advance(Syntax::BracketL);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syntax::BracketR, &[&["`]`"]]);
		trivia_0plus(p);
		ident_lax(p);
	} else {
		p.advance_err_and_close(
			stat,
			Syntax::from(t),
			Syntax::Error,
			&[&["`[`", "an identifier"]],
		);
		return;
	}

	trivia_0plus(p);
	p.expect(Token::Eq, Syntax::Eq, &[&["`=`"]]);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);
	expr::expr_list(p);
	trivia_0plus(p);
	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	p.close(stat, Syntax::StaticConstStat);
}

/// Builds a [`Syntax::DeclAssignStat`] or [`Syntax::LocalStat`] node.
fn declassign_or_local_stat(p: &mut Parser<Syntax>) {
	let stat = p.open();

	let syn = if p.at(Token::KwLet) && p.find(1, |token| !token.is_trivia()) == Token::BracketL {
		p.advance(Syntax::KwLet);
		trivia_0plus(p);
		p.advance(Syntax::BracketL);
		trivia_0plus(p);
		ident_list::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syntax::BracketR, &[&["`]`"]]);
		trivia_0plus(p);
		p.expect(Token::Eq, Syntax::Eq, &[&["`=`"]]);
		trivia_0plus(p);
		expr(p);
		trivia_0plus(p);
		Syntax::DeclAssignStat
	} else {
		local_var(p);
		Syntax::LocalStat
	};

	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	p.close(stat, syn);
}

/// Builds a [`Syntax::LocalVar`] node. Does not expect a trailing semicolon.
fn local_var(p: &mut Parser<Syntax>) {
	let local = p.open();
	type_ref(p);
	trivia_0plus(p);

	while p.at_if(is_ident_lax) {
		local_var_init(p);

		if p.find(0, |token| !token.is_trivia()) != Token::Comma {
			break;
		} else {
			trivia_0plus(p);
			p.advance(Syntax::Comma);
			trivia_0plus(p);
		}
	}

	p.close(local, Syntax::LocalVar);
}

/// Builds a [`Syntax::LocalVarInit`] node.
fn local_var_init(p: &mut Parser<Syntax>) {
	let init = p.open();
	ident_lax(p);
	trivia_0plus(p);

	match p.nth(0) {
		Token::BracketL => {
			trivia_0plus(p);
			array_len(p);

			if p.find(0, |token| !token.is_trivia()) == Token::Eq {
				trivia_0plus(p);
				p.advance(Syntax::Eq);
				trivia_0plus(p);
				p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
				trivia_0plus(p);
				expr::expr_list(p);
				trivia_0plus(p);
				p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
			}
		}
		Token::Eq => {
			p.advance(Syntax::Eq);
			trivia_0plus(p);

			if p.at(Token::BraceL) {
				p.advance(Syntax::BraceL);
				trivia_0plus(p);
				expr::expr_list(p);
				trivia_0plus(p);
				p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
			} else {
				expr(p);
			}
		}
		Token::BraceL => {
			p.advance(Syntax::BraceL);
			trivia_0plus(p);
			expr::expr_list(p);
			trivia_0plus(p);
			p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
		}
		Token::Semicolon | Token::Comma => {} // No initializer; valid.
		other => {
			p.advance_err_and_close(
				init,
				Syntax::from(other),
				Syntax::LocalVarInit,
				&[&["`[`", "`=`", "`{`"]],
			);
			return;
		}
	}

	p.close(init, Syntax::LocalVarInit);
}
