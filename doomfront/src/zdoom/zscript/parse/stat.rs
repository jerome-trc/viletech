//! Statement parsers.

use crate::{
	parser::Parser,
	zdoom::{zscript::Syn, Token},
};

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
/// - [`Syn::ReturnStat`]
/// - [`Syn::StaticConstStat`]
/// - [`Syn::SwitchStat`]
/// - [`Syn::UntilStat`]
/// - [`Syn::WhileStat`]
pub fn statement(p: &mut Parser<Syn>) {
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
		p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
		p.close(stat, Syn::ExprStat);
		return;
	}

	if in_type_ref_first_set(token) {
		declassign_or_local_stat(p);
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
			p.expect(Token::Colon, Syn::Colon, &[&["`:`"]]);
			p.close(stat, Syn::CaseStat);
		}
		Token::KwDefault => {
			let stat = p.open();
			p.advance(Syn::KwDefault);
			trivia_0plus(p);
			p.expect(Token::Colon, Syn::Colon, &[&["`:`"]]);
			p.close(stat, Syn::DefaultStat);
		}
		Token::BraceL => {
			compound_stat(p);
		}
		Token::KwSwitch => {
			let stat = p.open();
			p.advance(Syn::KwSwitch);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			statement(p);
			p.close(stat, Syn::SwitchStat);
		}
		Token::KwIf => {
			let stat = p.open();
			p.advance(Syn::KwIf);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			statement(p);

			if p.find(0, |token| !token.is_trivia()) == Token::KwElse {
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
			p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
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
						&[&["`until`", "`while`"]],
					);
					return;
				}
			};

			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);

			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);

			p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
			p.close(stat, node_syn);
		}
		Token::KwFor => {
			let stat = p.open();
			p.advance(Syn::KwFor);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);
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
					p.advance_with_error(Syn::from(t), &[&["an expression", "a local variable"]]);
				}

				if p.find(0, |token| !token.is_trivia()) == Token::Comma {
					trivia_0plus(p);
					p.advance(Syn::Comma);
					trivia_0plus(p);
				} else {
					break;
				}
			}

			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
			p.close(init, Syn::ForLoopInit);
			trivia_0plus(p);

			// Condition ///////////////////////////////////////////////////////

			let cond = p.open();

			if p.at_if(expr::in_first_set) {
				expr(p);
			}

			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
			p.close(cond, Syn::ForLoopCond);
			trivia_0plus(p);

			// "Bump" //////////////////////////////////////////////////////////

			let iter = p.open();

			while !p.at(Token::ParenR) && !p.eof() {
				let t = p.nth(0);

				if expr::in_first_set(t) {
					expr(p);
				} else {
					p.advance_with_error(Syn::from(t), &[&["an expression"]]);
				}

				if p.find(0, |token| !token.is_trivia()) == Token::Comma {
					trivia_0plus(p);
					p.advance(Syn::Comma);
					trivia_0plus(p);
				} else {
					break;
				}
			}

			p.close(iter, Syn::ForLoopIter);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			statement(p);
			p.close(stat, Syn::ForStat);
		}
		Token::KwForEach => {
			let stat = p.open();
			p.advance(Syn::KwForEach);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);

			trivia_0plus(p);
			var_name(p);
			trivia_0plus(p);

			p.expect(Token::Colon, Syn::Colon, &[&["`:`"]]);

			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);

			p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			statement(p);
			p.close(stat, Syn::ForEachStat);
		}
		Token::KwContinue => {
			let stat = p.open();
			p.advance(Syn::KwContinue);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
			p.close(stat, Syn::ContinueStat);
		}
		Token::KwBreak => {
			let stat = p.open();
			p.advance(Syn::KwBreak);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
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
			p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
			p.close(stat, Syn::ReturnStat);
		}
		Token::BracketL => {
			let stat = p.open();
			p.advance(Syn::BracketL);
			trivia_0plus(p);
			expr::expr_list(p);
			trivia_0plus(p);
			p.expect(Token::BracketR, Syn::BracketR, &[&["`]`"]]);
			trivia_0plus(p);
			p.expect(Token::Eq, Syn::Eq, &[&["`=`"]]);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
			p.close(stat, Syn::AssignStat);
		}
		Token::KwStatic => {
			static_const_stat(p);
		}
		other => {
			p.advance_with_error(
				Syn::from(other),
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

/// Builds a [`Syn::CompoundStat`] node.
pub(super) fn compound_stat(p: &mut Parser<Syn>) {
	let stat = p.open();
	p.expect(Token::BraceL, Syn::BraceL, &[&["`{`"]]);

	loop {
		trivia_0plus(p);

		if p.at(Token::BraceR) || p.eof() {
			break;
		}

		statement(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &[&["`}`", "a statement"]]);
	p.close(stat, Syn::CompoundStat);
}

/// Builds a [`Syn::StaticConstStat`] node.
pub(super) fn static_const_stat(p: &mut Parser<Syn>) {
	p.debug_assert_at_any(&[Token::KwStatic, Token::DocComment]);
	let stat = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwStatic);
	p.advance(Syn::KwStatic);
	trivia_1plus(p);
	p.expect(Token::KwConst, Syn::KwConst, &[&["`const`"]]);
	trivia_0plus(p);
	core_type(p);
	trivia_0plus(p);

	let t = p.find(0, |token| !token.is_trivia());

	if is_ident_lax(t) {
		trivia_0plus(p);
		ident_lax(p);
		trivia_0plus(p);
		p.expect(Token::BracketL, Syn::BracketL, &[&["`[`"]]);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syn::BracketR, &[&["`]`"]]);
	} else if t == Token::BracketL {
		trivia_0plus(p);
		p.advance(Syn::BracketL);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syn::BracketR, &[&["`]`"]]);
		trivia_0plus(p);
		ident_lax(p);
	} else {
		p.advance_err_and_close(stat, Syn::from(t), Syn::Error, &[&["`[`", "an identifier"]]);
		return;
	}

	trivia_0plus(p);
	p.expect(Token::Eq, Syn::Eq, &[&["`=`"]]);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &[&["`{`"]]);
	trivia_0plus(p);
	expr::expr_list(p);
	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &[&["`}`"]]);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
	p.close(stat, Syn::StaticConstStat);
}

/// Builds a [`Syn::DeclAssignStat`] or [`Syn::LocalStat`] node.
fn declassign_or_local_stat(p: &mut Parser<Syn>) {
	let stat = p.open();

	let syn = if p.at(Token::KwLet) && p.find(1, |token| !token.is_trivia()) == Token::BracketL {
		p.advance(Syn::KwLet);
		trivia_0plus(p);
		p.advance(Syn::BracketL);
		trivia_0plus(p);
		ident_list::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syn::BracketR, &[&["`]`"]]);
		trivia_0plus(p);
		p.expect(Token::Eq, Syn::Eq, &[&["`=`"]]);
		trivia_0plus(p);
		expr(p);
		trivia_0plus(p);
		Syn::DeclAssignStat
	} else {
		local_var(p);
		Syn::LocalStat
	};

	p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
	p.close(stat, syn);
}

/// Builds a [`Syn::LocalVar`] node. Does not expect a trailing semicolon.
fn local_var(p: &mut Parser<Syn>) {
	let local = p.open();
	type_ref(p);
	trivia_0plus(p);

	while p.at_if(is_ident_lax) {
		local_var_init(p);

		if p.find(0, |token| !token.is_trivia()) != Token::Comma {
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
fn local_var_init(p: &mut Parser<Syn>) {
	let init = p.open();
	ident_lax(p);
	trivia_0plus(p);

	match p.nth(0) {
		Token::BracketL => {
			trivia_0plus(p);
			array_len(p);

			if p.find(0, |token| !token.is_trivia()) == Token::Eq {
				trivia_0plus(p);
				p.advance(Syn::Eq);
				trivia_0plus(p);
				p.expect(Token::BraceL, Syn::BraceL, &[&["`{`"]]);
				trivia_0plus(p);
				expr::expr_list(p);
				trivia_0plus(p);
				p.expect(Token::BraceR, Syn::BraceR, &[&["`}`"]]);
			}
		}
		Token::Eq => {
			p.advance(Syn::Eq);
			trivia_0plus(p);

			if p.at(Token::BraceL) {
				p.advance(Syn::BraceL);
				trivia_0plus(p);
				expr::expr_list(p);
				trivia_0plus(p);
				p.expect(Token::BraceR, Syn::BraceR, &[&["`}`"]]);
			} else {
				expr(p);
			}
		}
		Token::BraceL => {
			p.advance(Syn::BraceL);
			trivia_0plus(p);
			expr::expr_list(p);
			trivia_0plus(p);
			p.expect(Token::BraceR, Syn::BraceR, &[&["`}`"]]);
		}
		Token::Semicolon | Token::Comma => {} // No initializer; valid.
		other => {
			p.advance_err_and_close(
				init,
				Syn::from(other),
				Syn::LocalVarInit,
				&[&["`[`", "`=`", "`{`"]],
			);
			return;
		}
	}

	p.close(init, Syn::LocalVarInit);
}
