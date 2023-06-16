//! A parser for the [Universal Doom Map Format](https://doomwiki.org/wiki/UDMF).

// NOTE: Block field readers use const slices of strings and function pointers to
// run comparisons and mutate level parts; Compiler Explorer says these get
// optimized to inline tests at `opt-level=3` as of 1.69.0. If you're reading this
// a year or two from now, test again, and see if the GCC backend does the same.

mod linedef;
mod sectordef;
mod sidedef;
mod thingdef;

use std::{
	collections::HashMap,
	num::{ParseFloatError, ParseIntError},
};

use logos::{Lexer, Logos};

use crate::repr::{
	UdmfValue, Vertex,
	{
		LevelDef, LevelFormat, LineDef, LineFlags, SectorDef, SideDef, ThingDef, ThingFlags,
		UdmfNamespace,
	},
};

#[derive(Debug)]
pub enum Error {
	InvalidNamespace(String),
	Lex(logos::Span),
	NoNamespace,
	Parse {
		found: Option<Token>,
		span: logos::Span,
		expected: &'static [Token],
	},
	ParseFloat {
		inner: ParseFloatError,
		input: String,
	},
	ParseInt {
		inner: ParseIntError,
		input: String,
	},
	TextmapEmpty,
	TextmapTooShort,
	UnknownVertDefField(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::InvalidNamespace(namespace) => {
				write!(f, "`{namespace}` is not a valid UDMF namespace")
			}
			Self::Lex(span) => {
				write!(f, "unrecognized token at {span:?}")
			}
			Self::NoNamespace => {
				write!(f, "TEXTMAP is missing a UDMF namespace statement")
			}
			Self::Parse {
				found,
				span,
				expected,
			} => {
				let fnd = if let Some(token) = found {
					format!("`{token:?}`")
				} else {
					"end of input".to_string()
				};

				write!(
					f,
					"Found {fnd} at position: {span:?}; expected one of the following: {expected:#?}"
				)
			}
			Self::ParseFloat { inner, input } => {
				write!(f, "Failed to parse float from: `{input}` - reason: {inner}")
			}
			Self::ParseInt { inner, input } => {
				write!(
					f,
					"Failed to parse integer from: `{input}` - reason: {inner}"
				)
			}
			Self::TextmapEmpty => {
				write!(f, "TEXTMAP is empty")
			}
			Self::TextmapTooShort => {
				write!(f, "TEXTMAP is too short for any meaningful content")
			}
			Self::UnknownVertDefField(name) => {
				write!(f, "TEXTMAP contains vertex with unknown field: `{name}`")
			}
		}
	}
}

pub fn parse_textmap(source: &str) -> Result<LevelDef, Vec<Error>> {
	let mut lexer = Token::lexer(source);
	let namespace = parse_namespace(&mut lexer).map_err(|err| vec![err])?;

	let mut parser = Parser {
		level: LevelDef::new(LevelFormat::Udmf(namespace)),
		lexer,
		buf: None,
		errors: vec![],
	};

	while let Some(token) = parser.advance() {
		match token {
			Token::KwSideDef => {
				parser.sidedef();
			}
			Token::KwLineDef => {
				parser.linedef();
			}
			Token::KwVertex => {
				parser.vertdef();
			}
			Token::KwSector => {
				parser.sectordef();
			}
			Token::KwThing => {
				parser.thingdef();
			}
			other => {
				let span = parser.lexer.span();
				parser.skip_until(|token| token.is_top_level_keyword());

				parser.errors.push(Error::Parse {
					found: Some(other),
					span,
					expected: &[
						Token::KwLineDef,
						Token::KwSector,
						Token::KwSideDef,
						Token::KwSector,
					],
				});

				continue;
			}
		}
	}

	let Parser {
		mut level,
		lexer: _,
		buf: _,
		errors,
	} = parser;

	if errors.is_empty() {
		level.bounds = LevelDef::bounds(&level.geom.vertdefs);
		Ok(level)
	} else {
		Err(errors)
	}
}

fn parse_namespace(lexer: &mut Lexer<Token>) -> Result<UdmfNamespace, Error> {
	let _ = lexer
		.find(|result| result.is_ok_and(|token| token == Token::KwNamespace))
		.ok_or(Error::NoNamespace)?;
	let _ = lexer
		.find(|result| result.is_ok_and(|token| token == Token::Eq))
		.ok_or(Error::NoNamespace)?;
	let _ = lexer
		.find(|result| result.is_ok_and(|token| token == Token::StringLit))
		.ok_or(Error::NoNamespace)?;

	let span = lexer.span();

	let _ = lexer
		.find(|result| result.is_ok_and(|token| token == Token::Semicolon))
		.ok_or(Error::NoNamespace)?;

	let ns_str = &lexer.source()[(span.start + 1)..(span.end - 1)];

	let ns = if ns_str.eq_ignore_ascii_case("doom") {
		UdmfNamespace::Doom
	} else if ns_str.eq_ignore_ascii_case("heretic") {
		UdmfNamespace::Heretic
	} else if ns_str.eq_ignore_ascii_case("hexen") {
		UdmfNamespace::Hexen
	} else if ns_str.eq_ignore_ascii_case("strife") {
		UdmfNamespace::Strife
	} else if ns_str.eq_ignore_ascii_case("zdoom") {
		UdmfNamespace::ZDoom
	} else if ns_str.eq_ignore_ascii_case("eternity") {
		UdmfNamespace::Eternity
	} else if ns_str.eq_ignore_ascii_case("vavoom") {
		UdmfNamespace::Vavoom
	} else if ns_str.eq_ignore_ascii_case("zdoomtranslated") {
		UdmfNamespace::ZDoomTranslated
	} else {
		return Err(Error::InvalidNamespace(ns_str.to_string()));
	};

	Ok(ns)
}

#[derive(Debug)]
struct Parser<'i> {
	level: LevelDef,
	lexer: logos::Lexer<'i, Token>,
	buf: Option<Token>,
	errors: Vec<Error>,
}

impl<'i> Parser<'i> {
	#[must_use]
	fn advance(&mut self) -> Option<Token> {
		if self.buf.is_none() {
			let ret = match self.lexer.next() {
				Some(Ok(token)) => Some(token),
				Some(Err(())) => Some(Token::Unknown),
				None => None,
			};

			self.buf = match self.lexer.next() {
				Some(Ok(token)) => Some(token),
				Some(Err(())) => Some(Token::Unknown),
				None => None,
			};

			return ret;
		}

		let ret = self.buf;

		self.buf = match self.lexer.next() {
			Some(Ok(token)) => Some(token),
			Some(Err(())) => Some(Token::Unknown),
			None => None,
		};

		ret
	}

	#[must_use]
	fn one_of(&mut self, expected: &'static [Token]) -> Option<Token> {
		self.advance().filter(|token| expected.contains(token))
	}

	fn skip_until<F: Fn(Token) -> bool>(&mut self, predicate: F) {
		loop {
			match self.advance() {
				Some(token) => {
					if predicate(token) {
						return;
					}
				}
				None => return,
			}
		}
	}

	fn linedef(&mut self) {
		let mut linedef = LineDef {
			udmf_id: -1,
			vert_start: usize::MAX,
			vert_end: usize::MAX,
			flags: LineFlags::empty(),
			special: 0,
			trigger: 0,
			args: [0; 5],
			side_right: usize::MAX,
			side_left: None,
			udmf: HashMap::default(),
		};

		if self.one_of(&[Token::BraceL]).is_none() {
			self.skip_until(|token| token == Token::BraceR || token.is_top_level_keyword());
		}

		self.fields(&mut linedef, linedef::read_linedef_field);

		if self.one_of(&[Token::BraceR]).is_none() {
			// Error reported; just proceed.
		}

		self.level.geom.linedefs.push(linedef);
	}

	fn thingdef(&mut self) {
		let mut thingdef = ThingDef {
			tid: 0,
			ed_num: 0,
			pos: glam::vec3(0.0, 0.0, 0.0),
			angle: 0,
			flags: ThingFlags::empty(),
			special: 0,
			args: [0; 5],
			udmf: HashMap::default(),
		};

		if self.one_of(&[Token::BraceL]).is_none() {
			self.skip_until(|token| token == Token::BraceR || token.is_top_level_keyword());
		}

		self.fields(&mut thingdef, thingdef::read_thingdef_field);

		if self.one_of(&[Token::BraceR]).is_none() {
			// Error reported; just proceed.
		}

		self.level.thingdefs.push(thingdef);
	}

	fn sectordef(&mut self) {
		let mut sectordef = SectorDef {
			udmf_id: i32::MAX,
			height_floor: 0.0,
			height_ceil: 0.0,
			tex_floor: None,
			tex_ceil: None,
			light_level: 0,
			special: 0,
			trigger: 0,
			udmf: HashMap::default(),
		};

		if self.one_of(&[Token::BraceL]).is_none() {
			self.skip_until(|token| token == Token::BraceR || token.is_top_level_keyword());
		}

		self.fields(&mut sectordef, sectordef::read_sectordef_field);

		if self.one_of(&[Token::BraceR]).is_none() {
			// Error reported; just proceed.
		}

		self.level.geom.sectordefs.push(sectordef);
	}

	fn sidedef(&mut self) {
		let mut sidedef = SideDef {
			offset: glam::IVec2::default(),
			tex_top: None,
			tex_bottom: None,
			tex_mid: None,
			sector: usize::MAX,
			udmf: HashMap::default(),
		};

		if self.one_of(&[Token::BraceL]).is_none() {
			self.skip_until(|token| token == Token::BraceR || token.is_top_level_keyword());
		}

		self.fields(&mut sidedef, sidedef::read_sidedef_field);

		if self.one_of(&[Token::BraceR]).is_none() {
			// Error reported; just proceed.
		}

		self.level.geom.sidedefs.push(sidedef);
	}

	fn vertdef(&mut self) {
		let mut vertex = Vertex(glam::Vec4::default());

		if self.one_of(&[Token::BraceL]).is_none() {
			self.skip_until(|token| token == Token::BraceR || token.is_top_level_keyword());
		}

		let mut err = None;

		self.fields(&mut vertex, |kvp, vert, _| {
			let float = match kvp.val {
				Value::Float(lit) => parse_f64(lit)?,
				_ => unimplemented!(),
			};

			// Recall that Y is up in VileTech.
			if kvp.key.eq_ignore_ascii_case("x") {
				vert.x = float as f32;
			} else if kvp.key.eq_ignore_ascii_case("y") {
				vert.z = float as f32;
			} else if kvp.key.eq_ignore_ascii_case("zceiling") {
				*vert.top_mut() = float as f32;
			} else if kvp.key.eq_ignore_ascii_case("zfloor") {
				*vert.bottom_mut() = float as f32;
			} else {
				err = Some(Error::UnknownVertDefField(kvp.key.to_string()));
			}

			Ok(())
		});

		if let Some(e) = err {
			self.errors.push(e);
		}

		if self.one_of(&[Token::BraceR]).is_none() {
			// Error reported; just proceed.
		}

		self.level.geom.vertdefs.push(vertex);
	}

	fn fields<F, T>(&mut self, elem: &mut T, mut reader: F)
	where
		F: FnMut(KeyValPair, &mut T, &LevelDef) -> Result<(), Error>,
	{
		loop {
			if !self.buf.is_some_and(|token| {
				matches!(
					token,
					Token::Ident
						| Token::KwSector | Token::KwLineDef
						| Token::KwNamespace | Token::KwSideDef
						| Token::KwThing | Token::KwVertex
				)
			}) {
				break;
			}

			let key_span = self.lexer.span();

			let _ = self.advance();

			if self.one_of(&[Token::Eq]).is_none() {
				self.skip_until(|token| matches!(token, Token::Semicolon | Token::BraceR));
				continue;
			}

			let val_span = self.lexer.span();

			let Some(val_token) = self.one_of(&[
				Token::IntLit,
				Token::FloatLit,
				Token::FalseLit,
				Token::TrueLit,
				Token::StringLit
			]) else {
				self.skip_until(|token| matches!(token, Token::Semicolon | Token::BraceR));
				continue;
			};

			let value = match val_token {
				Token::IntLit => Value::Int(&self.lexer.source()[val_span]),
				Token::FloatLit => Value::Float(&self.lexer.source()[val_span]),
				Token::FalseLit => Value::False,
				Token::TrueLit => Value::True,
				Token::StringLit => Value::String(&self.lexer.source()[val_span]),
				_ => unreachable!(),
			};

			if let Err(err) = reader(
				KeyValPair {
					key: &self.lexer.source()[key_span],
					val: value,
				},
				elem,
				&self.level,
			) {
				self.errors.push(err);
			}

			if self.one_of(&[Token::Semicolon]).is_none() {
				// Error reported; just proceed.
			}
		}
	}
}

#[derive(Debug)]
pub(crate) struct KeyValPair<'i> {
	key: &'i str,
	val: Value<'i>,
}

impl KeyValPair<'_> {
	pub(self) fn to_map_value(&self) -> UdmfValue {
		match self.val {
			Value::True => UdmfValue::Bool(true),
			Value::False => UdmfValue::Bool(false),
			Value::String(lit) => UdmfValue::String(lit.into()),
			Value::Float(lit) => match lit.parse::<f64>() {
				Ok(float) => UdmfValue::Float(float),
				Err(_) => UdmfValue::String(lit.into()),
			},
			Value::Int(lit) => match lit.parse::<i32>() {
				Ok(int) => UdmfValue::Int(int),
				Err(_) => UdmfValue::String(lit.into()),
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Value<'i> {
	True,
	False,
	String(&'i str),
	Float(&'i str),
	Int(&'i str),
}

/// An error mapping helper for convenience and brevity.
pub(self) fn parse_u16(lit: &str) -> Result<u16, Error> {
	lit.parse().map_err(|err| Error::ParseInt {
		inner: err,
		input: lit.to_string(),
	})
}

/// An error mapping helper for convenience and brevity.
pub(self) fn parse_i32(lit: &str) -> Result<i32, Error> {
	lit.parse().map_err(|err| Error::ParseInt {
		inner: err,
		input: lit.to_string(),
	})
}

/// An error mapping helper for convenience and brevity.
pub(self) fn parse_u32(lit: &str) -> Result<u32, Error> {
	lit.parse().map_err(|err| Error::ParseInt {
		inner: err,
		input: lit.to_string(),
	})
}

/// An error mapping helper for convenience and brevity.
pub(self) fn parse_usize(lit: &str) -> Result<usize, Error> {
	lit.parse().map_err(|err| Error::ParseInt {
		inner: err,
		input: lit.to_string(),
	})
}

/// An error mapping helper for convenience and brevity.
fn parse_f64(lit: &str) -> Result<f64, Error> {
	lit.parse().map_err(|err| Error::ParseFloat {
		inner: err,
		input: lit.to_string(),
	})
}

/// See the [UDMF spec](https://github.com/ZDoom/gzdoom/blob/master/specs/udmf.txt),
/// section I for its grammar.
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n\f]+", skip r"//[^\n\r]*[\n\r]*", skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")]
pub enum Token {
	// Literals ////////////////////////////////////////////////////////////////
	#[regex(r"(?i)false")]
	FalseLit,
	#[regex(r"[+-]?[0-9]+\.[0-9]*([eE][+-]?[0-9]+)?")]
	FloatLit,
	#[token("0")]
	#[regex(r"0x[0-9A-Fa-f]+")]
	#[regex(r"[+-]?0*[1-9][0-9]*")]
	IntLit,
	#[regex(r#""([^"\\]*(\\.[^"\\]*)*)""#)]
	StringLit,
	#[regex(r"(?i)true")]
	TrueLit,
	// Keywords ////////////////////////////////////////////////////////////////
	#[regex(r"(?i)linedef")]
	KwLineDef,
	#[regex(r"(?i)sector")]
	KwSector,
	#[regex(r"(?i)sidedef")]
	KwSideDef,
	#[regex(r"(?i)thing")]
	KwThing,
	#[regex(r"(?i)vertex")]
	KwVertex,

	#[regex(r"(?i)namespace")]
	KwNamespace,
	// Glyphs //////////////////////////////////////////////////////////////////
	#[token("{")]
	BraceL,
	#[token("}")]
	BraceR,
	#[token("=")]
	Eq,
	#[token(";")]
	Semicolon,
	// Miscellaneous ///////////////////////////////////////////////////////////
	#[regex(r"[A-Za-z_]+[A-Za-z0-9_]*")]
	Ident,
	/// Input the lexer failed to recognize gets mapped to this.
	Unknown,
}

impl Token {
	#[must_use]
	fn is_top_level_keyword(self) -> bool {
		let u = self as u8;
		(u >= Self::KwLineDef as u8) && (u <= Self::KwVertex as u8)
	}
}

#[cfg(test)]
mod test {
	use std::path::PathBuf;

	use super::*;

	#[test]
	fn with_sample_data() {
		const ENV_VAR: &str = "SUBTERRA_UDMF_SAMPLE";

		let path = match std::env::var(ENV_VAR) {
			Ok(v) => PathBuf::from(v),
			Err(_) => {
				eprintln!(
					"Environment variable not set: `{ENV_VAR}`. \
					Cancelling `udmf::test::with_sample_data`."
				);
				return;
			}
		};

		let bytes = std::fs::read(path)
			.map_err(|err| panic!("File I/O failure: {err}"))
			.unwrap();
		let source = String::from_utf8_lossy(&bytes);

		#[must_use]
		fn format_errs(errors: Vec<Error>) -> String {
			let mut output = String::new();

			for error in errors {
				output.push_str(&format!("\r\n{error:#?}"));
			}

			output
		}

		let _ = match parse_textmap(source.as_ref()) {
			Ok(l) => l,
			Err(errs) => {
				panic!("Encountered errors: {}\r\n", format_errs(errs));
			}
		};
	}
}
