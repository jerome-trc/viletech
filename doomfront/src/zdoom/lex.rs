//! (G)ZDoom's [common scanner], re-implemented via [Logos](logos).
//!
//! [common scanner]: https://github.com/ZDoom/gzdoom/blob/master/src/common/engine/sc_man_scanner.re

use super::Version;

#[derive(logos::Logos, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[logos(extras = Version, error = Token)]
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
	#[regex(r#""(([\\]["])|[^"])*""#, priority = 2)]
	StringLit,
	// Keywords ////////////////////////////////////////////////////////////////
	#[doc(hidden)]
	__FirstKw,
	#[regex("(?i)abstract", priority = 5)]
	KwAbstract,
	#[regex("(?i)action", priority = 5, callback = Token::ident_pre1_0_0)]
	KwAction,
	#[regex("(?i)alignof", priority = 5)]
	KwAlignOf,
	#[regex("(?i)array", priority = 5)]
	KwArray,
	#[regex("(?i)auto", priority = 5)]
	KwAuto,
	#[regex("(?i)bool", priority = 5)]
	KwBool,
	#[regex("(?i)break", priority = 5)]
	KwBreak,
	#[regex("(?i)bright", priority = 5)]
	KwBright,
	#[regex("(?i)byte", priority = 5)]
	KwByte,
	#[regex("(?i)canraise", priority = 5)]
	KwCanRaise,
	#[regex("(?i)case", priority = 5)]
	KwCase,
	#[regex("(?i)char", priority = 5)]
	KwChar,
	#[regex("(?i)clearscope", priority = 5, callback = Token::ident_pre2_4_0)]
	KwClearScope,
	#[regex("(?i)class", priority = 5)]
	KwClass,
	#[regex("(?i)color", priority = 5)]
	KwColor,
	#[regex("(?i)const", priority = 5)]
	KwConst,
	#[regex("(?i)continue", priority = 5)]
	KwContinue,
	#[regex("(?i)cross", priority = 5)]
	KwCross,
	#[regex("(?i)default", priority = 5)]
	KwDefault,
	#[regex("(?i)deprecated", priority = 5, callback = Token::ident_pre1_0_0)]
	KwDeprecated,
	#[regex("(?i)do", priority = 5)]
	KwDo,
	#[regex("(?i)dot", priority = 5)]
	KwDot,
	#[regex("(?i)double", priority = 5)]
	KwDouble,
	#[regex("(?i)else", priority = 5)]
	KwElse,
	#[regex("(?i)enum", priority = 5)]
	KwEnum,
	#[regex("(?i)extend", priority = 5, callback = Token::ident_pre1_0_0)]
	KwExtend,
	#[regex("(?i)fail", priority = 5)]
	KwFail,
	#[regex("(?i)false", priority = 5)]
	KwFalse,
	#[regex("(?i)fast", priority = 5)]
	KwFast,
	#[regex("(?i)final", priority = 5, callback = Token::ident_pre1_0_0)]
	KwFinal,
	#[regex("(?i)flagdef", priority = 5, callback = Token::ident_pre3_7_0)]
	KwFlagdef,
	#[regex("(?i)float", priority = 5)]
	KwFloat,
	#[regex("(?i)for", priority = 5)]
	KwFor,
	#[regex("(?i)foreach", priority = 5, callback = Token::ident_pre4_10_0)]
	KwForEach,
	#[regex("(?i)goto", priority = 5)]
	KwGoto,
	#[regex("(?i)in", priority = 5)]
	KwIn,
	#[regex("(?i)if", priority = 5)]
	KwIf,
	#[regex("(?i)int", priority = 5)]
	KwInt,
	#[regex("(?i)int16", priority = 5)]
	KwInt16,
	#[regex("(?i)int8", priority = 5)]
	KwInt8,
	#[regex("(?i)internal", priority = 5, callback = Token::ident_pre3_4_0)]
	KwInternal,
	#[regex("(?i)is", priority = 5, callback = Token::ident_pre1_0_0)]
	KwIs,
	#[regex("(?i)let", priority = 5, callback = Token::ident_pre1_0_0)]
	KwLet,
	#[regex("(?i)light", priority = 5)]
	KwLight,
	#[regex("(?i)long", priority = 5)]
	KwLong,
	#[regex("(?i)loop", priority = 5)]
	KwLoop,
	#[regex("(?i)map", priority = 5)]
	KwMap,
	#[regex("(?i)mapiterator", priority = 5)]
	KwMapIterator,
	#[regex("(?i)meta", priority = 5, callback = Token::ident_pre1_0_0)]
	KwMeta,
	#[regex("(?i)mixin", priority = 5)]
	KwMixin,
	#[regex("(?i)name", priority = 5, callback = Token::ident_pre1_0_0)]
	KwName,
	#[regex("(?i)native", priority = 5)]
	KwNative,
	#[regex("(?i)nodelay", priority = 5)]
	KwNoDelay,
	#[regex("(?i)none", priority = 5)]
	KwNone,
	#[regex("(?i)null", priority = 5)]
	#[regex("(?i)nullptr", priority = 5, callback = Token::ident_pre4_9_0)]
	KwNull,
	#[regex("(?i)offset", priority = 5)]
	KwOffset,
	#[regex("(?i)out", priority = 5, callback = Token::ident_pre1_0_0)]
	KwOut,
	#[regex("(?i)override", priority = 5, callback = Token::ident_pre1_0_0)]
	KwOverride,
	#[regex("(?i)play", priority = 5, callback = Token::ident_pre2_4_0)]
	KwPlay,
	#[regex("(?i)private", priority = 5, callback = Token::ident_pre1_0_0)]
	KwPrivate,
	#[regex("(?i)property", priority = 5)]
	KwProperty,
	#[regex("(?i)protected", priority = 5, callback = Token::ident_pre1_0_0)]
	KwProtected,
	#[regex("(?i)readonly", priority = 5, callback = Token::ident_pre1_0_0)]
	KwReadonly,
	#[regex("(?i)replaces", priority = 5, callback = Token::ident_pre1_0_0)]
	KwReplaces,
	#[regex("(?i)return", priority = 5)]
	KwReturn,
	#[regex("(?i)sbyte", priority = 5)]
	KwSByte,
	#[regex("(?i)short", priority = 5)]
	KwShort,
	#[regex("(?i)sizeof", priority = 5)]
	KwSizeof,
	#[regex("(?i)slow", priority = 5)]
	KwSlow,
	#[regex("(?i)sound", priority = 5)]
	KwSound,
	#[regex("(?i)state", priority = 5)]
	KwState,
	#[regex("(?i)states", priority = 5)]
	KwStates,
	#[regex("(?i)static", priority = 5)]
	KwStatic,
	#[regex("(?i)stop", priority = 5)]
	KwStop,
	#[regex("(?i)string", priority = 5)]
	KwString,
	#[regex("(?i)struct", priority = 5)]
	KwStruct,
	#[regex("(?i)super", priority = 5, callback = Token::ident_pre1_0_0)]
	KwSuper,
	#[regex("(?i)switch", priority = 5)]
	KwSwitch,
	#[regex("(?i)transient", priority = 5, callback = Token::ident_pre1_0_0)]
	KwTransient,
	#[regex("(?i)true", priority = 5)]
	KwTrue,
	#[regex("(?i)ui", priority = 5, callback = Token::ident_pre2_4_0)]
	KwUi,
	#[regex("(?i)uint", priority = 5)]
	KwUInt,
	#[regex("(?i)uint16", priority = 5)]
	KwUInt16,
	#[regex("(?i)uint8", priority = 5)]
	KwUInt8,
	#[regex("(?i)ulong", priority = 5)]
	KwULong,
	#[regex("(?i)until", priority = 5)]
	KwUntil,
	#[regex("(?i)ushort", priority = 5)]
	KwUShort,
	#[regex("(?i)var", priority = 5)]
	KwVar,
	#[regex("(?i)vararg", priority = 5, callback = Token::ident_pre1_0_0)]
	KwVarArg,
	#[regex("(?i)vector2", priority = 5)]
	KwVector2,
	#[regex("(?i)vector3", priority = 5)]
	KwVector3,
	#[regex("(?i)version", priority = 5, callback = Token::ident_pre2_4_0)]
	KwVersion,
	#[regex("(?i)virtual", priority = 5, callback = Token::ident_pre1_0_0)]
	KwVirtual,
	#[regex("(?i)virtualscope", priority = 5, callback = Token::ident_pre2_4_0)]
	KwVirtualScope,
	#[regex("(?i)void", priority = 5)]
	KwVoid,
	#[regex("(?i)volatile", priority = 5)]
	KwVolatile,
	#[regex("(?i)wait", priority = 5)]
	KwWait,
	#[regex("(?i)while", priority = 5)]
	KwWhile,
	#[doc(hidden)]
	__LastKw,
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
	At,
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
	#[token("$")]
	Dollar,
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
	Tilde,
	#[token("~==")]
	TildeEq2,
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
	#[regex("#region[^\n]*\n")]
	RegionStart,
	#[regex("#endregion[^\n]*\n")]
	RegionEnd,
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
	#[regex("//[^\n]*\n*", priority = 1)]
	#[regex("//")]
	#[regex(r"/[*]([^*]|([*][^/]))*[*]+/")]
	Comment,
	#[default]
	Unknown,
	Eof,
}

impl Token {
	#[must_use]
	pub fn is_keyword(self) -> bool {
		let u = self as u8;
		u > (Self::__FirstKw as u8) && u < (Self::__LastKw as u8)
	}

	// Callbacks ///////////////////////////////////////////////////////////////////

	#[allow(unused)]
	fn ident_pre1_0_0(lexer: &mut logos::Lexer<Self>) -> Result<(), Self> {
		if lexer.extras >= Version::V1_0_0 {
			Ok(())
		} else {
			Err(Self::Ident)
		}
	}

	#[allow(unused)]
	fn ident_pre2_4_0(lexer: &mut logos::Lexer<Self>) -> Result<(), Self> {
		if lexer.extras >= Version::V2_4_0 {
			Ok(())
		} else {
			Err(Self::Ident)
		}
	}

	#[allow(unused)]
	fn ident_pre3_4_0(lexer: &mut logos::Lexer<Self>) -> Result<(), Self> {
		if lexer.extras >= Version::V3_4_0 {
			Ok(())
		} else {
			Err(Self::Ident)
		}
	}

	#[allow(unused)]
	fn ident_pre3_7_0(lexer: &mut logos::Lexer<Self>) -> Result<(), Self> {
		if lexer.extras >= Version::V3_7_0 {
			Ok(())
		} else {
			Err(Self::Ident)
		}
	}

	#[allow(unused)]
	fn ident_pre4_9_0(lexer: &mut logos::Lexer<Self>) -> Result<(), Self> {
		if lexer.extras >= Version::V4_9_0 {
			Ok(())
		} else {
			Err(Self::Ident)
		}
	}

	#[allow(unused)]
	fn ident_pre4_10_0(lexer: &mut logos::Lexer<Self>) -> Result<(), Self> {
		if lexer.extras >= Version::V4_10_0 {
			Ok(())
		} else {
			Err(Self::Ident)
		}
	}
}

#[cfg(test)]
mod test {
	use std::path::PathBuf;

	use logos::Logos;

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
			let token = match result {
				Ok(t) | Err(t) => t,
			};

			println!("{token:?} ({:?}) : `{}`", lexer.span(), lexer.slice());
		}
	}

	#[test]
	fn with_sample_data() {
		const ENV_VAR: &str = "DOOMFRONT_ZDOOM_LEX_SAMPLE";

		let path = match std::env::var(ENV_VAR) {
			Ok(v) => PathBuf::from(v),
			Err(_) => {
				eprintln!("Environment variable not set: `{ENV_VAR}`.");
				return;
			}
		};

		if !path.exists() {
			eprintln!(
				"Path passed via `{ENV_VAR}` does not exist: {}",
				path.display()
			);
			return;
		}

		let bytes = std::fs::read(path).unwrap();
		let source = String::from_utf8(bytes).unwrap();

		let mut lexer = Token::lexer(&source);

		while let Some(result) = lexer.next() {
			let token = match result {
				Ok(t) | Err(t) => t,
			};

			println!("{token:?} ({:?}) : `{}`", lexer.span(), lexer.slice());
		}
	}
}
