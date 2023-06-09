//! Frontend for the [`DECORATE`](https://zdoom.org/wiki/DECORATE)
//! language defined by (G)ZDoom.
//!
//! DECORATE is a data definition language and pseudo-scripting language for
//! creating new game content.

pub mod ast;
pub mod parse;
mod syn;

pub use syn::Syn;

use rowan::{GreenNode, GreenToken};

use crate::{
	parsing::{Gtb12, Gtb4, Gtb8},
	GreenElement,
};

pub type IncludeTree = parse::IncludeTree;
pub type SyntaxNode = rowan::SyntaxNode<Syn>;
pub type SyntaxToken = rowan::SyntaxToken<Syn>;
pub type SyntaxElem = rowan::SyntaxElement<Syn>;

peg::parser! {
	pub grammar parser() for str {
		// Expressions /////////////////////////////////////////////////////////

		pub rule expr() -> GreenNode = precedence! {
			// Assignment //////////////////////////////////////////////////////
			lhs:@ t0:trivia()* eq:$("=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Eq, eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* anglel2_eq:$("<<=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AngleL2Eq, anglel2_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* angler3_eq:$(">>>=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AngleR3Eq, angler3_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* angler2_eq:$(">>=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AngleR2Eq, angler2_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* plus_eq:$("+=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::PlusEq, plus_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* minus_eq:$("-=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::MinusEq, minus_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* asterisk_eq:$("*=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AsteriskEq, asterisk_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* slash_eq:$("/=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::SlashEq, slash_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* pct_eq:$("%=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::PercentEq, pct_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* amp_eq:$("&=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AmpersandEq, amp_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* pipe_eq:$("|=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::PipeEq, pipe_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:@ t0:trivia()* caret_eq:$("^=") t1:trivia()* rhs:(@) {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::CaretEq, caret_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			// Ternary /////////////////////////////////////////////////////////
			cond:@
			t0:trivia()*
			quest:$("?")
			t1:trivia()*
			if_true:expr()
			t2:trivia()*
			colon:$(":")
			t3:trivia()*
			if_false:(@) {
				let mut gtb = Gtb12::new(Syn::TernaryExpr, Syn::Error);
				gtb.start(cond);
				gtb.append(t0);
				gtb.start_s(Syn::Question, quest);
				gtb.append(t1);
				gtb.start(if_true);
				gtb.append(t2);
				gtb.start_s(Syn::Colon, ":");
				gtb.append(t3);
				gtb.finish()
			}
			--
			// Binary, logical comparison //////////////////////////////////////
			lhs:(@) t0:trivia()* pipe2:$("||") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Pipe2, pipe2);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			lhs:(@) t0:trivia()* amp2:$("&&") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Ampersand2, amp2);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			lhs:(@) t0:trivia()* eq2:$("==") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Eq2, eq2);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:(@) t0:trivia()* bang_eq:$("!=") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::BangEq, bang_eq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			// Binary, ordered comparison //////////////////////////////////////
			lhs:(@) t0:trivia()* angle_req:$(">=") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AngleREq, angle_req);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:(@) t0:trivia()* angle_leq:$("<=") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AngleLEq, angle_leq);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:(@) t0:trivia()* angle_r:$(">") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AngleR, angle_r);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:(@) t0:trivia()* angle_l:$("<") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AngleL, angle_l);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			// Binary, bitwise /////////////////////////////////////////////////
			lhs:(@) t0:trivia()* pipe:$("|") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Pipe, pipe);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			lhs:(@) t0:trivia()* caret:$("^") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Caret, caret);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			lhs:(@) t0:trivia()* amp:$("&") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Ampersand, amp);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			lhs:(@) t0:trivia()* plus:$("<<") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AngleL2, plus);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:(@) t0:trivia()* plus:$(">>") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AngleR2, plus);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:(@) t0:trivia()* angler3:$(">>>") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::AngleR3, angler3);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			// Binary, arithmetic //////////////////////////////////////////////
			lhs:(@) t0:trivia()* plus:$("+") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Plus, plus);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:(@) t0:trivia()* minus:$("-") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Minus, minus);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			lhs:(@) t0:trivia()* asterisk:$("*") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Asterisk, asterisk);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:(@) t0:trivia()* slash:$("/") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Slash, slash);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			lhs:(@) t0:trivia()* pct:$("%") t1:trivia()* rhs:@ {
				let mut gtb = Gtb8::new(Syn::BinExpr, Syn::Error);
				gtb.start(lhs);
				gtb.append(t0);
				gtb.start_s(Syn::Percent, pct);
				gtb.append(t1);
				gtb.start(rhs);
				gtb.finish()
			}
			--
			// Unary, prefix ///////////////////////////////////////////////////
			plus2:$("++") t:trivia()* operand:@ {
				let mut gtb = Gtb4::new(Syn::UnaryExpr, Syn::Error);
				gtb.start_s(Syn::Plus2, plus2);
				gtb.append(t);
				gtb.start(operand);
				gtb.finish()
			}
			minus2:$("--") t:trivia()* operand:@  {
				let mut gtb = Gtb4::new(Syn::UnaryExpr, Syn::Error);
				gtb.start_s(Syn::Minus2, minus2);
				gtb.append(t);
				gtb.start(operand);
				gtb.finish()
			}
			plus:$("+") t:trivia()* operand:@ {
				let mut gtb = Gtb4::new(Syn::UnaryExpr, Syn::Error);
				gtb.start_s(Syn::Plus, plus);
				gtb.append(t);
				gtb.start(operand);
				gtb.finish()
			}
			minus:$("-") t:trivia()* operand:@ {
				let mut gtb = Gtb4::new(Syn::UnaryExpr, Syn::Error);
				gtb.start_s(Syn::Minus, minus);
				gtb.append(t);
				gtb.start(operand);
				gtb.finish()
			}
			bang:$("!") t:trivia()* operand:@ {
				let mut gtb = Gtb4::new(Syn::UnaryExpr, Syn::Error);
				gtb.start_s(Syn::Bang, bang);
				gtb.append(t);
				gtb.start(operand);
				gtb.finish()
			}
			grave:$("~") t:trivia()* operand:@  {
				let mut gtb = Gtb4::new(Syn::UnaryExpr, Syn::Error);
				gtb.start_s(Syn::Grave, grave);
				gtb.append(t);
				gtb.start(operand);
				gtb.finish()
			}
			// Unary, postfix //////////////////////////////////////////////////
			operand:@ t:trivia()* plus2:$("++") {
				let mut gtb = Gtb4::new(Syn::UnaryExpr, Syn::Error);
				gtb.start(operand);
				gtb.append(t);
				gtb.start_s(Syn::Plus2, plus2);
				gtb.finish()
			}
			operand:@ t:trivia()* minus2:$("--") {
				let mut gtb = Gtb4::new(Syn::UnaryExpr, Syn::Error);
				gtb.start(operand);
				gtb.append(t);
				gtb.start_s(Syn::Minus2, minus2);
				gtb.finish()
			}
			--
			// Non-atomic primary (call, indexing) /////////////////////////////
			lhs:(@)
			t0:trivia()*
			parenl:$("(")
			t1:trivia()*
			args:expr_list()?
			t2:trivia()*
			parenr:$(")")? {
				let mut gtb = Gtb8::new(Syn::CallExpr, Syn::Error);
				gtb.append(t0);
				gtb.start_s(Syn::ParenL, parenl);
				gtb.append(t1);
				gtb.maybe_append(args);
				gtb.append(t2);
				gtb.just_s(Syn::ParenR, parenr);
				gtb.finish()
			}
			lhs:(@)
			t0:trivia()*
			bracketl:$("[")
			t1:trivia()*
			index:expr()
			t2:trivia()*
			bracketr:$("]")? {
				let mut gtb = Gtb8::new(Syn::IndexExpr, Syn::Error);
				gtb.append(t0);
				gtb.start_s(Syn::BracketL, bracketl);
				gtb.append(t1);
				gtb.start(index);
				gtb.append(t2);
				gtb.just_s(Syn::BracketR, bracketr);
				gtb.finish()
			}
			--
			// Atoms, grouped //////////////////////////////////////////////////
			parenl:$("(") t0:trivia()* inner:expr() t1:trivia()* parenr:$(")") {
				let mut gtb = Gtb8::new(Syn::GroupedExpr, Syn::Error);
				gtb.start_s(Syn::ParenL, parenl);
				gtb.append(t0);
				gtb.start(inner);
				gtb.append(t1);
				gtb.start_s(Syn::ParenR, parenr);
				gtb.finish()
			}
			id:ident() {
				GreenNode::new(Syn::IdentExpr.into(), [id.into()])
			}
			lit:(
				string_lit() / name_lit() /
				float_lit() / int_lit() /
				true_lit() / false_lit()
			) {
				GreenNode::new(Syn::Literal.into(), [lit.into()])
			}
		}

		pub rule expr_list() -> Vec<GreenElement>
			= 	expr0:expr()
				t0:trivia()*
				items:expr_list_item()*
		{
			let mut t0 = t0;

			t0.insert(0, expr0.into());

			for mut item in items {
				t0.append(&mut item);
			}

			t0
		}

		rule expr_list_item() -> Vec<GreenElement>
			= t0:trivia()* comma:$(",") t1:trivia()* expr1:expr()
		{
			let mut t0 = t0;
			let mut t1 = t1;
			t0.push(GreenToken::new(Syn::Comma.into(), comma).into());
			t0.append(&mut t1);
			t0.push(expr1.into());
			t0
		}

		// Literals ////////////////////////////////////////////////////////////

		rule false_lit() -> GreenToken = lit:$("false") {
			GreenToken::new(Syn::KwFalse.into(), lit)
		}

		pub rule float_lit() -> GreenToken
			= lit:$(
				(
					['0'..='9']+
					['e' | 'E']
					['+' | '-']?
					['0'..='9']+
					['f' | 'F']?
				) /
				(
					['0'..='9']*
					"."
					['0'..='9']+
					(
						['e' | 'E']
						['+' | '-']?
						['0'..='9']+
					)?
					['f' | 'F']?
				) /
				(
					['0'..='9']+
					"."
					['0'..='9']*
					(
						['e' | 'E']
						['+' | '-']?
						['0'..='9']+
					)?
					['f' | 'F']?
				)
			)
		{
			GreenToken::new(Syn::FloatLit.into(), lit)
		}

		pub rule int_lit() -> GreenToken
			= lit:$(
				(
					['0'..='9']+
					['u' | 'U' | 'l' | 'L']?
					['u' | 'U' | 'l' | 'L']?
				) /
				(
					"0"
					['x' | 'X']
					['a'..='z' | 'A'..='Z' | '0'..='9']+
					['u' | 'U' | 'l' | 'L']?
					['u' | 'U' | 'l' | 'L']?
				) /
				(
					"0"
					['0'..='9']+
					['u' | 'U' | 'l' | 'L']?
					['u' | 'U' | 'l' | 'L']?
				)
			)
		{
			GreenToken::new(Syn::IntLit.into(), lit)
		}

		pub rule name_lit() -> GreenToken
			= lit:$("'" [^ '\n' | '\'']* "'")
		{
			GreenToken::new(Syn::NameLit.into(), lit)
		}

		pub rule string_lit() -> GreenToken
			= lit:$("\"" (("\\" "\"") / ([^ '"']))* "\"")
		{
			GreenToken::new(Syn::StringLit.into(), lit)
		}

		rule true_lit() -> GreenToken = lit:$("true") {
			GreenToken::new(Syn::KwTrue.into(), lit)
		}

		// Miscellaneous ///////////////////////////////////////////////////////

		rule ident() -> GreenToken
			= string:$(
				['a'..='z' | 'A'..='Z' | '_']
				['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*
			)
		{
			GreenToken::new(Syn::Ident.into(), string)
		}

		pub rule trivia() -> GreenElement = t:(wsp() / comment()) { t.into() }

		pub rule wsp() -> GreenToken = string:$(['\0'..=' ']+) {
			GreenToken::new(Syn::Whitespace.into(), string)
		}

		pub rule comment() -> GreenToken
			= string:(
				$(
					"//" [^ '\n']* "\n"*
				) /
				$(
					"/*" ([^ '*'] / ("*" [^ '/']))* "*"+ "/"
				)
			)
		{
			GreenToken::new(Syn::Comment.into(), string)
		}
	}
}
