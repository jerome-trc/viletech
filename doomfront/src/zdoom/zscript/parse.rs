mod actor;
mod common;
mod expr;
mod stat;
mod structure;
mod top;

use chumsky::{primitive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	parser_t,
	zdoom::{self, Token},
	GreenElement,
};

use super::Syn;

/// Gives context to functions yielding parser combinators
/// (e.g. the user's selected ZScript version).
///
/// Thus, this information never has to be passed through deep call trees, and any
/// breaking changes to this context are minimal in scope.
#[derive(Debug, Clone)]
pub struct ParserBuilder {
	pub(self) _version: zdoom::Version,
}

impl ParserBuilder {
	#[must_use]
	pub fn new(version: zdoom::Version) -> Self {
		Self { _version: version }
	}

	/// The returned parser emits a [`Syn::Root`] node.
	pub fn file<'i>(&self) -> parser_t!(GreenNode) {
		// TODO: Single-class file syntax.

		primitive::choice((
			self.trivia(),
			self.class_def().map(GreenElement::from),
			self.struct_def().map(GreenElement::from),
			self.enum_def().map(GreenElement::from),
			self.const_def().map(GreenElement::from),
			self.class_extend().map(GreenElement::from),
			self.struct_extend().map(GreenElement::from),
			self.mixin_class_def().map(GreenElement::from),
			self.include_directive().map(GreenElement::from),
			self.version_directive().map(GreenElement::from),
		))
		.repeated()
		.collect::<Vec<_>>()
		.map(|elems| GreenNode::new(Syn::Root.into(), elems))
		.boxed()
	}
}

pub mod hand {
	use crate::parser::CloseMark;

	use super::*;

	impl crate::parser::LangExt for Syn {
		type Token = Token;
		const EOF: Self::Token = Token::Eof;
		const ERR_NODE: Self::Kind = Syn::Error;
	}

	fn _ident(p: &mut crate::parser::Parser<Syn>) {
		p.expect_any(
			&[
				(Token::Ident, Syn::Ident),
				(Token::KwBright, Syn::Ident),
				(Token::KwCanRaise, Syn::Ident),
				(Token::KwFast, Syn::Ident),
				(Token::KwLight, Syn::Ident),
				(Token::KwOffset, Syn::Ident),
				(Token::KwSlow, Syn::Ident),
			],
			&["an identifier"],
		)
	}

	#[must_use]
	fn _eat_ident(p: &mut crate::parser::Parser<Syn>) -> bool {
		p.eat_any(&[
			(Token::Ident, Syn::Ident),
			(Token::KwBright, Syn::Ident),
			(Token::KwCanRaise, Syn::Ident),
			(Token::KwFast, Syn::Ident),
			(Token::KwLight, Syn::Ident),
			(Token::KwOffset, Syn::Ident),
			(Token::KwSlow, Syn::Ident),
		])
	}

	pub fn _expr(p: &mut crate::parser::Parser<Syn>) {
		_recur(p, Token::Eof);
	}

	fn _recur(p: &mut crate::parser::Parser<Syn>, left: Token) {
		let mut lhs = _primary_expr(p);

		loop {
			_trivia_0plus(p);

			let right = p.nth(0);

			match right {
				Token::Minus2 => {
					let m = p.open_before(lhs);
					p.advance(Syn::Minus2);
					lhs = p.close(m, Syn::PostfixExpr);
					continue;
				}
				Token::Plus2 => {
					let m = p.open_before(lhs);
					p.advance(Syn::Plus2);
					lhs = p.close(m, Syn::PostfixExpr);
					continue;
				}
				Token::ParenL => {
					let m = p.open_before(lhs);
					_trivia_0plus(p);
					_arg_list(p);
					_trivia_0plus(p);
					lhs = p.close(m, Syn::CallExpr);
					continue;
				}
				Token::BracketL => {
					let m = p.open_before(lhs);
					p.expect(Token::BracketL, Syn::BracketL, &["`[`"]);
					_trivia_0plus(p);
					_expr(p);
					_trivia_0plus(p);
					p.expect(Token::BracketR, Syn::BracketR, &["`]`"]);
					lhs = p.close(m, Syn::IndexExpr);
					continue;
				}
				_ => {}
			}

			if _infix_right_stronger(left, right) {
				if right == Token::Question {
					let m = p.open_before(lhs);
					p.advance(Syn::Question);
					_trivia_0plus(p);
					_expr(p);
					_trivia_0plus(p);
					p.expect(Token::Colon, Syn::Colon, &["`:`"]);
					_trivia_0plus(p);
					_expr(p);
					lhs = p.close(m, Syn::TernaryExpr);
				} else {
					let m = p.open_before(lhs);
					p.advance(Syn::from(right));
					_trivia_0plus(p);
					_recur(p, right);
					lhs = p.close(m, Syn::BinExpr);
				}
			} else {
				break;
			}
		}
	}

	fn _primary_expr(p: &mut crate::parser::Parser<Syn>) -> CloseMark {
		let ex = p.open();

		if _eat_ident(p) {
			return p.close(ex, Syn::IdentExpr);
		}

		match p.nth(0) {
			Token::IntLit => {
				p.advance(Syn::IntLit);
				p.close(ex, Syn::Literal)
			}
			Token::FloatLit => {
				p.advance(Syn::FloatLit);
				p.close(ex, Syn::Literal)
			}
			Token::KwTrue => {
				p.advance(Syn::TrueLit);
				p.close(ex, Syn::Literal)
			}
			Token::KwFalse => {
				p.advance(Syn::FalseLit);
				p.close(ex, Syn::Literal)
			}
			Token::StringLit => {
				p.advance(Syn::StringLit);
				p.close(ex, Syn::Literal)
			}
			Token::NameLit => {
				p.advance(Syn::NameLit);
				p.close(ex, Syn::Literal)
			}
			Token::KwNull => {
				p.advance(Syn::NullLit);
				p.close(ex, Syn::Literal)
			}
			Token::ParenL => {
				p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
				_trivia_0plus(p);

				if p.eat(Token::KwClass, Syn::KwClass) {
					// Class cast
					_trivia_0plus(p);
					p.expect(Token::AngleL, Syn::AngleL, &["`<`"]);
					_trivia_0plus(p);
					_ident(p);
					_trivia_0plus(p);
					p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
					_trivia_0plus(p);
					p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
					_trivia_0plus(p);
					_arg_list(p);
					return p.close(ex, Syn::ClassCastExpr);
				}

				_expr(p);
				_trivia_0plus(p);

				if p.eat(Token::ParenR, Syn::ParenR) {
					p.close(ex, Syn::GroupExpr)
				} else if p.eat(Token::Comma, Syn::Comma) {
					// Vector
					for _ in 0..3 {
						_trivia_0plus(p);
						_expr(p);
						_trivia_0plus(p);

						if !p.eat(Token::Comma, Syn::Comma) {
							p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
							break;
						}
					}

					p.close(ex, Syn::VectorExpr)
				} else {
					p.advance_err_and_close(ex, Syn::from(p.nth(0)), Syn::Error)
				}
			}
			Token::Bang => {
				p.advance(Syn::Bang);
				_recur(p, Token::Bang);
				p.close(ex, Syn::PrefixExpr)
			}
			Token::Minus2 => {
				p.advance(Syn::Minus2);
				_recur(p, Token::Minus2);
				p.close(ex, Syn::PrefixExpr)
			}
			Token::Plus2 => {
				p.advance(Syn::Plus2);
				_recur(p, Token::Plus2);
				p.close(ex, Syn::PrefixExpr)
			}
			Token::Minus => {
				p.advance(Syn::Minus);
				_recur(p, Token::Minus);
				p.close(ex, Syn::PrefixExpr)
			}
			Token::Plus => {
				p.advance(Syn::Plus);
				_recur(p, Token::Plus);
				p.close(ex, Syn::PrefixExpr)
			}
			Token::Tilde => {
				p.advance(Syn::Tilde);
				_recur(p, Token::Tilde);
				p.close(ex, Syn::PrefixExpr)
			}
			_ => p.advance_err_and_close(ex, Syn::Unknown, Syn::Error),
		}
	}

	fn _arg_list(p: &mut crate::parser::Parser<Syn>) {
		debug_assert!(p.at(Token::ParenL));
		let arglist = p.open();
		p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
		_trivia_0plus(p);

		while !p.at(Token::ParenR) && !p.eof() {
			_argument(p);
		}

		_trivia_0plus(p);
		p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
		p.close(arglist, Syn::ArgList);
	}

	fn _argument(p: &mut crate::parser::Parser<Syn>) {
		let arg = p.open();

		if _eat_ident(p) {
			_trivia_0plus(p);
			p.expect(Token::Colon, Syn::Colon, &["`:`"]);
			_trivia_0plus(p);
		}

		_expr(p);
		_trivia_0plus(p);

		if !p.at(Token::ParenR) {
			p.expect(Token::Comma, Syn::Comma, &["`,`"]);
		}

		p.close(arg, Syn::Argument);
	}

	#[must_use]
	fn _infix_right_stronger(left: Token, right: Token) -> bool {
		#[must_use]
		fn strength(token: Token) -> Option<usize> {
			const PREC_TABLE: &[&[Token]] = &[
				&[
					Token::Eq,
					Token::AsteriskEq,
					Token::SlashEq,
					Token::PercentEq,
					Token::PlusEq,
					Token::MinusEq,
					Token::AngleL2Eq,
					Token::AngleR2Eq,
					Token::AmpersandEq,
					Token::PipeEq,
					Token::CaretEq,
					Token::AngleR3Eq,
				],
				&[Token::Question],
				&[Token::Pipe2],
				&[Token::Ampersand2],
				&[Token::Eq2, Token::BangEq, Token::TildeEq2],
				&[
					Token::AngleL,
					Token::AngleR,
					Token::AngleLEq,
					Token::AngleREq,
					Token::AngleLAngleREq,
					Token::KwIs,
				],
				&[Token::Dot2],
				&[Token::Pipe],
				&[Token::Caret],
				&[Token::Ampersand],
				&[Token::AngleL2, Token::AngleR2, Token::AngleR3],
				&[Token::Plus, Token::Minus],
				&[
					Token::Asterisk,
					Token::Slash,
					Token::Percent,
					Token::KwCross,
					Token::KwDot,
				],
				&[Token::Asterisk2],
				&[Token::Minus2, Token::Plus2],
				&[Token::Dot],
			];

			PREC_TABLE.iter().position(|level| level.contains(&token))
		}

		let Some(right_s) = strength(right) else {
			return false;
		};

		let Some(left_s) = strength(left) else {
			debug_assert_eq!(left, Token::Eof);
			return true;
		};

		right_s > left_s
	}

	fn _trivia(p: &mut crate::parser::Parser<Syn>) -> bool {
		p.eat(Token::Whitespace, Syn::Whitespace) || p.eat(Token::Comment, Syn::Comment)
	}

	fn _trivia_0plus(p: &mut crate::parser::Parser<Syn>) {
		while _trivia(p) {}
	}

	fn _trivia_1plus(p: &mut crate::parser::Parser<Syn>) {
		p.expect_any(
			&[
				(Token::Whitespace, Syn::Whitespace),
				(Token::Comment, Syn::Comment),
			],
			&["whitespace or a comment (one or more)"],
		);

		_trivia_0plus(p);
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
	fn with_sample_data() {
		let (_, sample) = match read_sample_data("DOOMFRONT_ZSCRIPT_SAMPLE") {
			Ok(s) => s,
			Err(err) => {
				eprintln!("Skipping ZScript sample data-based unit test. Reason: {err}");
				return;
			}
		};

		let parser = ParserBuilder::new(Version::default()).file();
		let tbuf = crate::scan(&sample, zdoom::Version::default());
		let result = crate::parse(parser, &sample, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
		assert_no_errors(&ptree);
	}
}
