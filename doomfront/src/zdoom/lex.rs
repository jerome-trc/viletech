//! (G)ZDoom's [common scanner], re-implemented via [Logos](logos).
//!
//! [common scanner]: https://github.com/ZDoom/gzdoom/blob/master/src/common/engine/sc_man_scanner.re

use chumsky::prelude::Input;
use logos::Logos;

use crate::zdoom;

#[derive(logos::Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize, serde::Deserialize))]
#[logos(extras = crate::zdoom::Version)]
pub enum Token {
	// Literals ////////////////////////////////////////////////////////////////
	#[regex(r"[0-9]+([Ee][+-]?[0-9]+)[fF]?", priority = 4)]
	#[regex(r"[0-9]*\.[0-9]+([Ee][+-]?[0-9]+)?[fF]?", priority = 3)]
	#[regex(r"[0-9]+\.[0-9]*([Ee][+-]?[0-9]+)?[fF]?", priority = 2)]
	FloatLit,
	#[regex("0[xX][a-fA-F0-9]+[uUlL]?[uUlL]?", priority = 4)]
	#[regex(r"0[0-9]+[uUlL]?[uUlL]?", priority = 3)]
	#[regex(r"[0-9]+[uUlL]?[uUlL]?", priority = 2)]
	IntLit,
	#[regex("'[^''\n]*'")]
	NameLit,
	// #[regex(r#""((")|[^"])*""#, priority = 2)]
	#[regex(r#""(([\\]["])|[^"])*""#, priority = 2)]
	StringLit,
	// Keywords ////////////////////////////////////////////////////////////////
	#[regex("(?i)break", priority = 5)]
	KwBreak,
	#[regex("(?i)case", priority = 5)]
	KwCase,
	#[regex("(?i)const", priority = 5)]
	KwConst,
	#[regex("(?i)continue", priority = 5)]
	KwContinue,
	#[regex("(?i)default", priority = 5)]
	KwDefault,
	#[regex("(?i)do", priority = 5)]
	KwDo,
	#[regex("(?i)else", priority = 5)]
	KwElse,
	#[regex("(?i)for", priority = 5)]
	KwFor,
	#[regex("(?i)goto", priority = 5)]
	KwGoto,
	#[regex("(?i)if", priority = 5)]
	KwIf,
	#[regex("(?i)return", priority = 5)]
	KwReturn,
	#[regex("(?i)switch", priority = 5)]
	KwSwitch,
	#[regex("(?i)until", priority = 5)]
	KwUntil,
	#[regex("(?i)volatile", priority = 5)]
	KwVolatile,
	#[regex("(?i)while", priority = 5)]
	KwWhile,
	#[regex("(?i)bool", priority = 5)]
	KwBool,
	#[regex("(?i)float", priority = 5)]
	KwFloat,
	#[regex("(?i)double", priority = 5)]
	KwDouble,
	#[regex("(?i)char", priority = 5)]
	KwChar,
	#[regex("(?i)byte", priority = 5)]
	KwByte,
	#[regex("(?i)sbyte", priority = 5)]
	KwSByte,
	#[regex("(?i)short", priority = 5)]
	KwShort,
	#[regex("(?i)ushort", priority = 5)]
	KwUShort,
	#[regex("(?i)int8", priority = 5)]
	KwInt8,
	#[regex("(?i)uint8", priority = 5)]
	KwUInt8,
	#[regex("(?i)int16", priority = 5)]
	KwInt16,
	#[regex("(?i)uint16", priority = 5)]
	KwUInt16,
	#[regex("(?i)int", priority = 5)]
	KwInt,
	#[regex("(?i)uint", priority = 5)]
	KwUInt,
	#[regex("(?i)long", priority = 5)]
	KwLong,
	#[regex("(?i)ulong", priority = 5)]
	KwULong,
	#[regex("(?i)void", priority = 5)]
	KwVoid,
	#[regex("(?i)struct", priority = 5)]
	KwStruct,
	#[regex("(?i)class", priority = 5)]
	KwClass,
	#[regex("(?i)mixin", priority = 5)]
	KwMixin,
	#[regex("(?i)enum", priority = 5)]
	KwEnum,
	#[regex("(?i)name", ident_pre1_0_0, priority = 5)]
	KwName,
	#[regex("(?i)string", priority = 5)]
	KwString,
	#[regex("(?i)sound", priority = 5)]
	KwSound,
	#[regex("(?i)state", priority = 5)]
	KwState,
	#[regex("(?i)color", priority = 5)]
	KwColor,
	#[regex("(?i)vector2", priority = 5)]
	KwVector2,
	#[regex("(?i)vector3", priority = 5)]
	KwVector3,
	#[regex("(?i)map", priority = 5)]
	KwMap,
	#[regex("(?i)mapiterator", priority = 5)]
	KwMapIterator,
	#[regex("(?i)array", priority = 5)]
	KwArray,
	#[regex("(?i)in", priority = 5)]
	KwIn,
	#[regex("(?i)sizeof", priority = 5)]
	KwSizeof,
	#[regex("(?i)alignof", priority = 5)]
	KwAlignof,
	#[regex("(?i)abstract", priority = 5)]
	KwAbstract,
	#[regex("(?i)foreach", ident_pre4_10_0, priority = 5)]
	KwForeach,
	#[regex("(?i)true", priority = 5)]
	KwTrue,
	#[regex("(?i)false", priority = 5)]
	KwFalse,
	#[regex("(?i)none", priority = 5)]
	KwNone,
	#[regex("(?i)auto", priority = 5)]
	KwAuto,
	#[regex("(?i)property", priority = 5)]
	KwProperty,
	#[regex("(?i)flagdef", ident_pre3_7_0, priority = 5)]
	KwFlagdef,
	#[regex("(?i)native", priority = 5)]
	KwNative,
	#[regex("(?i)var", priority = 5)]
	KwVar,
	#[regex("(?i)out", ident_pre1_0_0, priority = 5)]
	KwOut,
	#[regex("(?i)static", priority = 5)]
	KwStatic,
	#[regex("(?i)transient", ident_pre1_0_0, priority = 5)]
	KwTransient,
	#[regex("(?i)final", ident_pre1_0_0, priority = 5)]
	KwFinal,
	#[regex("(?i)extend", ident_pre1_0_0, priority = 5)]
	KwExtend,
	#[regex("(?i)protected", ident_pre1_0_0, priority = 5)]
	KwProtected,
	#[regex("(?i)private", ident_pre1_0_0, priority = 5)]
	KwPrivate,
	#[regex("(?i)dot", priority = 5)]
	KwDot,
	#[regex("(?i)cross", priority = 5)]
	KwCross,
	#[regex("(?i)virtual", ident_pre1_0_0, priority = 5)]
	KwVirtual,
	#[regex("(?i)override", ident_pre1_0_0, priority = 5)]
	KwOverride,
	#[regex("(?i)vararg", ident_pre1_0_0, priority = 5)]
	KwVararg,
	#[regex("(?i)ui", ident_pre2_4_0, priority = 5)]
	KwUi,
	#[regex("(?i)play", ident_pre2_4_0, priority = 5)]
	KwPlay,
	#[regex("(?i)clearscope", ident_pre2_4_0, priority = 5)]
	KwClearscope,
	#[regex("(?i)virtualscope", ident_pre2_4_0, priority = 5)]
	KwVirtualscope,
	#[regex("(?i)super", ident_pre1_0_0, priority = 5)]
	KwSuper,
	#[regex("(?i)stop", priority = 5)]
	KwStop,
	#[regex("(?i)null", priority = 5)]
	#[regex("(?i)nullptr", ident_pre4_9_0, priority = 5)]
	KwNull,
	#[regex("(?i)is", ident_pre1_0_0, priority = 5)]
	KwIs,
	#[regex("(?i)replaces", ident_pre1_0_0, priority = 5)]
	KwReplaces,
	#[regex("(?i)states", priority = 5)]
	KwStates,
	#[regex("(?i)meta", ident_pre1_0_0, priority = 5)]
	KwMeta,
	#[regex("(?i)deprecated", ident_pre1_0_0, priority = 5)]
	KwDeprecated,
	#[regex("(?i)version", ident_pre2_4_0, priority = 5)]
	KwVersion,
	#[regex("(?i)action", ident_pre1_0_0, priority = 5)]
	KwAction,
	#[regex("(?i)readonly", ident_pre1_0_0, priority = 5)]
	KwReadonly,
	#[regex("(?i)internal", ident_pre3_4_0, priority = 5)]
	KwInternal,
	#[regex("(?i)let", ident_pre1_0_0, priority = 5)]
	KwLet,
	#[regex("(?i)bright", priority = 5)]
	KwBright,
	#[regex("(?i)fast", priority = 5)]
	KwFast,
	#[regex("(?i)slow", priority = 5)]
	KwSlow,
	#[regex("(?i)nodelay", priority = 5)]
	KwNoDelay,
	#[regex("(?i)canraise", priority = 5)]
	KwCanRaise,
	#[regex("(?i)offset", priority = 5)]
	KwOffset,
	#[regex("(?i)light", priority = 5)]
	KwLight,
	// Glyphs //////////////////////////////////////////////////////////////////
	#[token("&")]
	Ampersand,
	#[token("&&")]
	Ampersand2,
	#[token("&=")]
	AmpersandEq,
	#[token("<")]
	AngleL,
	#[token("<<")]
	AngleL2,
	#[token("<=")]
	AngleLEq,
	#[token("<<=")]
	AngleL2Eq,
	#[token(">")]
	AngleR,
	#[token(">=")]
	AngleREq,
	#[token(">>")]
	AngleR2,
	#[token(">>>")]
	AngleR3,
	#[token(">>=")]
	AngleR2Eq,
	#[token(">>>=")]
	AngleR3Eq,
	#[token("<>=")]
	AngleLAngleREq,
	#[token("*")]
	Asterisk,
	#[token("**")]
	Asterisk2,
	#[token("*=")]
	AsteriskEq,
	#[token("@")]
	AtSign,
	#[token("!")]
	Bang,
	#[token("!=")]
	BangEq,
	#[token("{")]
	BraceL,
	#[token("}")]
	BraceR,
	#[token("[")]
	BracketL,
	#[token("]")]
	BracketR,
	#[token("^")]
	Caret,
	#[token("^=")]
	CaretEq,
	#[token(":")]
	Colon,
	#[token("::")]
	Colon2,
	#[token(",")]
	Comma,
	#[token(".")]
	Dot,
	#[token("..")]
	Dot2,
	#[token("...")]
	Dot3,
	#[token("=")]
	Eq,
	#[token("==")]
	Eq2,
	#[token("~")]
	Grave,
	#[token("~==")]
	GraveEq2,
	#[token("-")]
	Minus,
	#[token("--")]
	Minus2,
	#[token("-=")]
	MinusEq,
	#[token("(")]
	ParenL,
	#[token(")")]
	ParenR,
	#[token("%")]
	Percent,
	#[token("%=")]
	PercentEq,
	#[token("|")]
	Pipe,
	#[token("||")]
	Pipe2,
	#[token("|=")]
	PipeEq,
	#[token("+")]
	Plus,
	#[token("++")]
	Plus2,
	#[token("+=")]
	PlusEq,
	#[token("#")]
	Pound,
	#[token("?")]
	Question,
	#[token(";")]
	Semicolon,
	#[token("/")]
	Slash,
	#[token("/=")]
	SlashEq,
	#[token("->")]
	ThinArrow,
	// Miscellaneous ///////////////////////////////////////////////////////////
	#[regex("(?i)#include")]
	PoundInclude,
	#[regex("[a-zA-Z_][a-zA-Z0-9_]*", priority = 4)]
	Ident,
	/// A heterogenous span of any character between NUL and ASCII 32.
	#[regex("[\0- ]+")]
	Whitespace,
	/// Doc comments are applicable only to [ZScript](crate::zdoom::zscript),
	/// and non-standard, being defined by [zscdoc].
	///
	/// [zscdoc]: https://gitlab.com/Gutawer/zscdoc
	#[regex(r#"///([^/][^\n]*)?"#, priority = 2)]
	DocComment,
	#[regex("//[^\n]*\n", priority = 1)]
	#[regex(r"/[*]([^*]|([*][^/]))*[*]+/")]
	Comment,
	Error,
}

pub type Lexer<'i> = crate::Lexer<'i, Token>;
pub type TokenMapper = crate::TokenMapper<Token>;
pub type TokenStream<'i> = crate::TokenStream<'i, Token>;

impl Token {
	#[must_use]
	pub fn stream<'i>(source: &'i str, version: Option<zdoom::Version>) -> TokenStream<'i> {
		fn mapper(input: (Result<Token, ()>, logos::Span)) -> (Token, logos::Span) {
			(input.0.unwrap_or(Token::Error), input.1)
		}

		let f: TokenMapper = mapper; // Yes, this is necessary.

		chumsky::input::Stream::from_iter(
			Token::lexer_with_extras(source, version.unwrap_or_default())
				.spanned()
				.map(f),
		)
		.spanned(source.len()..source.len())
	}
}

// Lexer callbacks /////////////////////////////////////////////////////////////

#[allow(unused)]
fn ident_pre1_0_0(lexer: &mut logos::Lexer<Token>) -> logos::Filter<()> {
	if lexer.extras >= zdoom::Version::V1_0_0 {
		logos::Filter::Emit(())
	} else {
		logos::Filter::Skip
	}
}

#[allow(unused)]
fn ident_pre2_4_0(lexer: &mut logos::Lexer<Token>) -> logos::Filter<()> {
	if lexer.extras >= zdoom::Version::V2_4_0 {
		logos::Filter::Emit(())
	} else {
		logos::Filter::Skip
	}
}

#[allow(unused)]
fn ident_pre3_4_0(lexer: &mut logos::Lexer<Token>) -> logos::Filter<()> {
	if lexer.extras >= zdoom::Version::V3_4_0 {
		logos::Filter::Emit(())
	} else {
		logos::Filter::Skip
	}
}

#[allow(unused)]
fn ident_pre3_7_0(lexer: &mut logos::Lexer<Token>) -> logos::Filter<()> {
	if lexer.extras >= zdoom::Version::V3_7_0 {
		logos::Filter::Emit(())
	} else {
		logos::Filter::Skip
	}
}

#[allow(unused)]
fn ident_pre4_9_0(lexer: &mut logos::Lexer<Token>) -> logos::Filter<()> {
	if lexer.extras >= zdoom::Version::V4_9_0 {
		logos::Filter::Emit(())
	} else {
		logos::Filter::Skip
	}
}

#[allow(unused)]
fn ident_pre4_10_0(lexer: &mut logos::Lexer<Token>) -> logos::Filter<()> {
	if lexer.extras >= zdoom::Version::V4_10_0 {
		logos::Filter::Emit(())
	} else {
		logos::Filter::Skip
	}
}

#[cfg(test)]
mod test {
	use super::*;

	const SOURCE: &str = r#"

States (actor, overlay) {
	Spawn:
		FAIL A -2 offset(-1, 1) light("?", "??") light("!") {
				return GetSpawnHealth;
		} TNT1 A Random(1, 2) A_Pain
	Death: ____ A 0
	Labelled: TNT1 A 0
	9: TNT1 A 0
		7HA_ A 15 bright
		HAX7 B 15 bright A_Pain
		HAX7 "A[" 20 bright {
			health = Random[ rngtbl ](3, 4);
			health = Random[rgbtbl](1, 2);
			health = sqrt(1);
		}
		gOTO super :: SPAWN + 0
}

"#;

	#[test]
	fn smoke() {
		let mut lexer = Token::lexer(SOURCE);

		while let Some(result) = lexer.next() {
			let token = result.unwrap();
			println!("{token:?} : `{}`", lexer.slice());
		}
	}
}
