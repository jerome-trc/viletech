//! [Token] and VZScript's procedurally-generated tokenizer.

use doomfront::chumsky::{self, prelude::Input};
use logos::Logos;

use crate::Version;

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[logos(extras = Version)]
pub enum Token {
	// Literals ////////////////////////////////////////////////////////////////
	#[regex(
		r##"(?x)"
(
	[^"\\]|
	(\\')|
	(\\")|
	(\\n)|
	(\\r)|
	(\\t)|
	(\\\\)|
	(\\0)|
	(\\x[0-8][0-9a-fA-F])|
	(\\u\{([0-9a-fA-F]_*){1,6}\})|
	(\\\n)
)*
""##
	)]
	StringLit,
	#[regex("[0-9][0-9_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	#[regex("0b[01_]*[01][01_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	#[regex("0o[0-7_]*[0-7][0-7_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	#[regex("0x[0-9a-fA-F_]*[0-9a-fA-F][0-9a-fA-F_]*(u8|u16|u32|u64|i8|i16|i32|i64)?")]
	IntLit,
	#[regex(r#"[0-9][0-9_]*\.[^._\p{XID_Start}]*(f32|f64)?"#)]
	#[regex(r#"[0-9][0-9_]*\.[0-9][0-9_]*(f32|f64)?"#)]
	#[regex(r#"[0-9][0-9_]*(\.[0-9][0-9_]*)?[eE][+-]?[0-9_]*[0-9][0-9_]*(f32|f64)?"#)]
	FloatLit,
	#[token("true")]
	TrueLit,
	#[token("false")]
	FalseLit,
	// TODO: Raw string literals
	#[token("()")]
	VoidLit,
	// Keywords ////////////////////////////////////////////////////////////////
	#[token("const")]
	KwConst,
	#[token("func")]
	KwFunc,
	#[token("struct")]
	KwStruct,
	// Glyphs //////////////////////////////////////////////////////////////////
	#[token("!")]
	Bang,
	#[token("[")]
	BracketL,
	#[token("]")]
	BracketR,
	#[token("-")]
	Minus,
	#[token("(")]
	ParenL,
	#[token(")")]
	ParenR,
	#[token("+")]
	Plus,
	#[token("#")]
	Pound,
	// Miscellaneous ///////////////////////////////////////////////////////////
	#[regex(r#"///([^/][^\n]*)?"#, priority = 2)]
	DocComment,
	#[regex("//[^\n]*\n", priority = 1)]
	#[regex(r"/[*]([^*]|([*][^/]))*[*]+/")]
	Comment,
	#[regex("[ \r\n\t]+")]
	Whitespace,
	#[regex("[a-zA-Z_][a-zA-Z0-9_]*", priority = 4)]
	Ident,
	Error,
}

pub type Lexer<'i> = doomfront::Lexer<'i, Token>;
pub type TokenMapper = doomfront::TokenMapper<Token>;
pub type TokenStream<'i> = doomfront::TokenStream<'i, Token>;

impl Token {
	#[must_use]
	pub fn stream(source: &str, version: Version) -> TokenStream<'_> {
		fn mapper(input: (Result<Token, ()>, logos::Span)) -> (Token, logos::Span) {
			(input.0.unwrap_or(Token::Error), input.1)
		}

		let f: TokenMapper = mapper; // Yes, this is necessary.

		chumsky::input::Stream::from_iter(
			Token::lexer_with_extras(source, version).spanned().map(f),
		)
		.spanned(source.len()..source.len())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn smoke_string() {
		const SOURCE: &str = r#""waiting for Romero to play""#;

		let mut lexer = Token::lexer_with_extras(
			SOURCE,
			Version {
				major: 0,
				minor: 0,
				rev: 0,
			},
		);

		let t = lexer.next().unwrap().unwrap();
		assert_eq!(t, Token::StringLit);
		assert_eq!(lexer.slice(), r#""waiting for Romero to play""#);
	}
}
