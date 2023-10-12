//! A parser for the [Universal Doom Map Format](https://doomwiki.org/wiki/UDMF).

// NOTE: Block field readers use const slices of strings and function pointers to
// run comparisons and mutate level parts; Compiler Explorer says these get
// optimized to inline tests at `opt-level=3` as of 1.69.0. If you're reading this
// a year or two from now, test again, and see if the GCC backend does the same.

use logos::{Lexer, Logos};

/// UDMF files are large by necessity, so this trait exists to allow users to
/// define the most flexible and performant way to consume parsed input.
///
/// For consumption by [`parse`].
pub trait Sink: Sized {
	type Context: Sized;

	type LineDef;
	type SectorDef;
	type SideDef;
	type ThingDef;
	type Vertex;

	/// Returning `None` is considered an error condition and will cause an early exit from parsing.
	#[must_use]
	fn with_namespace(string: &str, ctx: Self::Context) -> Option<Self>;

	#[must_use]
	fn start_linedef(&mut self) -> Self::LineDef;
	fn linedef_property(&mut self, linedef: &mut Self::LineDef, kvp: KeyVal);
	fn finish_linedef(&mut self, linedef: Self::LineDef);

	#[must_use]
	fn start_sectordef(&mut self) -> Self::SectorDef;
	fn sectordef_property(&mut self, sectordef: &mut Self::SectorDef, kvp: KeyVal);
	fn finish_sectordef(&mut self, sectordef: Self::SectorDef);

	#[must_use]
	fn start_sidedef(&mut self) -> Self::SideDef;
	fn sidedef_property(&mut self, sidedef: &mut Self::SideDef, kvp: KeyVal);
	fn finish_sidedef(&mut self, sidedef: Self::SideDef);

	#[must_use]
	fn start_thingdef(&mut self) -> Self::ThingDef;
	fn thingdef_property(&mut self, thingdef: &mut Self::ThingDef, kvp: KeyVal);
	fn finish_thingdef(&mut self, thingdef: Self::ThingDef);

	#[must_use]
	fn start_vertex(&mut self) -> Self::Vertex;
	fn vertex_property(&mut self, vertex: &mut Self::Vertex, kvp: KeyVal);
	fn finish_vertex(&mut self, vertex: Self::Vertex);

	fn parse_error(&mut self, error: Error);
}

/// See [`Sink`].
#[derive(Debug)]
pub struct KeyVal<'i> {
	pub key: &'i str,
	pub val: Value<'i>,
}

/// See [`Sink`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value<'i> {
	True,
	False,
	String(&'i str),
	Float(&'i str),
	Int(&'i str),
}

pub fn parse<S: Sink>(source: &str, sink_ctx: S::Context) -> Result<S, Error> {
	let mut lexer = Token::lexer(source);

	let mut sink = match parse_namespace::<S>(&mut lexer, sink_ctx) {
		Ok(s) => s,
		Err(err) => return Err(err),
	};

	let mut parser = Parser {
		sink: &mut sink,
		lexer,
		buf: None,
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

				parser.sink.parse_error(Error::Parse {
					found: other,
					span,
					expected: &[
						Token::KwLineDef,
						Token::KwSector,
						Token::KwSideDef,
						Token::KwSector,
						Token::KwVertex,
					],
				});

				parser.skip_until(|token| token.is_top_level_keyword());
				continue;
			}
		}
	}

	Ok(sink)
}

#[derive(Debug)]
pub enum Error {
	InvalidNamespace(String),
	Lex(logos::Span),
	NoNamespace,
	Parse {
		found: Token,
		span: logos::Span,
		expected: &'static [Token],
	},
	TextmapEmpty,
	TextmapTooShort,
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
				write!(
					f,
					"found {found} at position: {span:?}; expected one of the following: {expected:#?}"
				)
			}
			Self::TextmapEmpty => {
				write!(f, "TEXTMAP is empty")
			}
			Self::TextmapTooShort => {
				write!(f, "TEXTMAP is too short for any meaningful content")
			}
		}
	}
}

fn parse_namespace<S: Sink>(lexer: &mut Lexer<Token>, ctx: S::Context) -> Result<S, Error> {
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

	S::with_namespace(ns_str, ctx).ok_or_else(|| Error::InvalidNamespace(ns_str.to_owned()))
}

#[derive(Debug)]
struct Parser<'i, S: Sink> {
	sink: &'i mut S,
	lexer: logos::Lexer<'i, Token>,
	buf: Option<Token>,
}

impl<'i, S: Sink> Parser<'i, S> {
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

	fn expect(&mut self, expected: &'static [Token]) -> Result<Token, (Token, logos::Span)> {
		match self.advance() {
			Some(t) => {
				if expected.contains(&t) {
					Ok(t)
				} else {
					Err((t, self.lexer.span()))
				}
			}
			None => Err((
				Token::Eof,
				self.lexer.source().len()..self.lexer.source().len(),
			)),
		}
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
		if let Err(err) = self.expect(&[Token::BraceL]) {
			self.sink.parse_error(Error::Parse {
				found: err.0,
				expected: &[Token::BraceL],
				span: err.1,
			});

			self.skip_until(|token| token == Token::BraceR || token.is_top_level_keyword());
			return;
		}

		let mut linedef = self.sink.start_linedef();
		self.fields(&mut linedef, S::linedef_property);

		if let Err(err) = self.expect(&[Token::BraceR]) {
			self.sink.parse_error(Error::Parse {
				found: err.0,
				expected: &[Token::BraceR],
				span: err.1,
			});
		}

		self.sink.finish_linedef(linedef);
	}

	fn thingdef(&mut self) {
		if let Err(err) = self.expect(&[Token::BraceL]) {
			self.sink.parse_error(Error::Parse {
				found: err.0,
				expected: &[Token::BraceL],
				span: err.1,
			});

			self.skip_until(|token| token == Token::BraceR || token.is_top_level_keyword());
			return;
		}

		let mut thingdef = self.sink.start_thingdef();
		self.fields(&mut thingdef, S::thingdef_property);

		if let Err(err) = self.expect(&[Token::BraceR]) {
			self.sink.parse_error(Error::Parse {
				found: err.0,
				expected: &[Token::BraceR],
				span: err.1,
			});
		}

		self.sink.finish_thingdef(thingdef);
	}

	fn sectordef(&mut self) {
		if let Err(err) = self.expect(&[Token::BraceL]) {
			self.sink.parse_error(Error::Parse {
				found: err.0,
				expected: &[Token::BraceL],
				span: err.1,
			});

			self.skip_until(|token| token == Token::BraceR || token.is_top_level_keyword());
			return;
		}

		let mut sectordef = self.sink.start_sectordef();
		self.fields(&mut sectordef, S::sectordef_property);

		if let Err(err) = self.expect(&[Token::BraceR]) {
			self.sink.parse_error(Error::Parse {
				found: err.0,
				expected: &[Token::BraceR],
				span: err.1,
			});
		}

		self.sink.finish_sectordef(sectordef);
	}

	fn sidedef(&mut self) {
		if let Err(err) = self.expect(&[Token::BraceL]) {
			self.sink.parse_error(Error::Parse {
				found: err.0,
				expected: &[Token::BraceL],
				span: err.1,
			});

			self.skip_until(|token| token == Token::BraceR || token.is_top_level_keyword());
			return;
		}

		let mut sidedef = self.sink.start_sidedef();
		self.fields(&mut sidedef, S::sidedef_property);

		if let Err(err) = self.expect(&[Token::BraceR]) {
			self.sink.parse_error(Error::Parse {
				found: err.0,
				expected: &[Token::BraceR],
				span: err.1,
			});
		}

		self.sink.finish_sidedef(sidedef);
	}

	fn vertdef(&mut self) {
		if let Err(err) = self.expect(&[Token::BraceL]) {
			self.sink.parse_error(Error::Parse {
				found: err.0,
				expected: &[Token::BraceL],
				span: err.1,
			});

			self.skip_until(|token| token == Token::BraceR || token.is_top_level_keyword());
			return;
		}

		let mut vertex = self.sink.start_vertex();
		self.fields(&mut vertex, S::vertex_property);

		if let Err(err) = self.expect(&[Token::BraceR]) {
			self.sink.parse_error(Error::Parse {
				found: err.0,
				expected: &[Token::BraceR],
				span: err.1,
			});
		}

		self.sink.finish_vertex(vertex);
	}

	fn fields<F, T>(&mut self, obj: &mut T, mut callback: F)
	where
		F: FnMut(&mut S, &mut T, KeyVal),
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

			if let Err(err) = self.expect(&[Token::Eq]) {
				self.sink.parse_error(Error::Parse {
					found: err.0,
					span: err.1,
					expected: &[Token::Eq],
				});

				self.skip_until(|token| matches!(token, Token::Semicolon | Token::BraceR));
				continue;
			}

			let val_span = self.lexer.span();

			const EXPECTED: &[Token] = &[
				Token::IntLit,
				Token::FloatLit,
				Token::FalseLit,
				Token::TrueLit,
				Token::StringLit,
			];

			let val_token = match self.expect(EXPECTED) {
				Ok(t) => t,
				Err(err) => {
					self.sink.parse_error(Error::Parse {
						found: err.0,
						span: err.1,
						expected: EXPECTED,
					});

					self.skip_until(|token| matches!(token, Token::Semicolon | Token::BraceR));
					continue;
				}
			};

			let value = match val_token {
				Token::IntLit => Value::Int(&self.lexer.source()[val_span]),
				Token::FloatLit => Value::Float(&self.lexer.source()[val_span]),
				Token::FalseLit => Value::False,
				Token::TrueLit => Value::True,
				Token::StringLit => Value::String(&self.lexer.source()[val_span]),
				_ => unreachable!(),
			};

			callback(
				self.sink,
				obj,
				KeyVal {
					key: &self.lexer.source()[key_span],
					val: value,
				},
			);

			if let Err(err) = self.expect(&[Token::Semicolon]) {
				self.sink.parse_error(Error::Parse {
					found: err.0,
					span: err.1,
					expected: &[Token::Semicolon],
				});
			}
		}
	}
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
	/// Only used for [`Error::Parse`].
	Eof,
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

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Token::FalseLit => write!(f, "`false`"),
			Token::FloatLit => write!(f, "a floating-point number"),
			Token::IntLit => write!(f, "an integer"),
			Token::StringLit => write!(f, "a string"),
			Token::TrueLit => write!(f, "`true`"),
			Token::KwLineDef => write!(f, "`linedef`"),
			Token::KwSector => write!(f, "`sector`"),
			Token::KwSideDef => write!(f, "`sidedef`"),
			Token::KwThing => write!(f, "`thing`"),
			Token::KwVertex => write!(f, "`vertex`"),
			Token::KwNamespace => write!(f, "`namespace`"),
			Token::BraceL => write!(f, "`{{`"),
			Token::BraceR => write!(f, "`}}`"),
			Token::Eq => write!(f, "`=`"),
			Token::Semicolon => write!(f, "`;`"),
			Token::Eof => write!(f, "end of input"),
			Token::Ident => write!(f, "an identifier"),
			Token::Unknown => write!(f, "unknown"),
		}
	}
}
