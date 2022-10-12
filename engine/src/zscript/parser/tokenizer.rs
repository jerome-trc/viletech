/*

Copyright (C) 2021-2022 Jessica "Gutawer" Russell

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use core::str::CharIndices;
use std::collections::VecDeque;
use std::ops::Range;

use std::borrow::Cow;
use str_utils::*;

use super::error::{ParsingError, ParsingErrorLevel};
use super::fs::FileIndex;
use super::Span;

macro_rules! rule_grouping_type {
    ($name: ident, $reg_func: ident, $match_func: ident {
        $( $rule: literal => $res: ident ),*
    }) => {
        #[derive(Debug, PartialEq, Copy, Clone)]
        pub enum $name {
            $(
                $res,
            )*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match *self {
                    $(
                        Self::$res => write!(f, "`{}`", $rule),
                    )*
                }
            }
        }

        #[allow(unused)]
        fn $match_func<'src>(r: &'src str) -> Option<$name> {
            match &*r.to_lowercase() {
                $(
                    $rule => Some($name::$res),
                )*
                _ => None
            }
        }
    };
}

#[derive(Debug, PartialEq)]
pub struct Token<'src> {
	pub original: &'src str,
	pub data: TokenData<'src>,
}

fn span(file: FileIndex, text: &str, sub: &str) -> Span {
	let text_ptr = text.as_ptr() as usize;
	let self_ptr = sub.as_ptr() as usize;
	let start = self_ptr - text_ptr;
	let end = start + sub.len();
	Span { start, end, file }
}

impl<'src> Token<'src> {
	pub(super) fn span(&self, file: FileIndex, text: &'src str) -> Span {
		span(file, text, self.original)
	}
}

#[derive(Debug, PartialEq)]
pub enum TokenData<'src> {
	Identifier(&'src str),
	Name(&'src str),
	String(Cow<'src, str>),
	Int {
		val: u64,
		long: bool,
		unsigned: bool,
	},
	Float {
		val: f64,
		double: bool,
	},
	Punctuation(Punctuation),
	Keyword(Keyword),
	Include,

	NonWhitespace(Cow<'src, str>),

	DocComment(&'src str),
}

impl<'src> std::fmt::Display for TokenData<'src> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Identifier(_) => write!(f, "an identifier"),
			Self::Name(_) => write!(f, "a name literal"),
			Self::String(_) => write!(f, "a string literal"),
			Self::Int { .. } => write!(f, "an integer constant"),
			Self::Float { .. } => write!(f, "a float constant"),
			Self::Punctuation(p) => write!(f, "{}", p),
			Self::Keyword(k) => write!(f, "{}", k),
			Self::Include => write!(f, "#include"),

			Self::NonWhitespace(_) => write!(f, "non-whitespace"),

			Self::DocComment(_) => write!(f, "documentation string"),
		}
	}
}

rule_grouping_type!(Punctuation, punc, match_punc {
	r">>>=" => UnsignedRightShiftAssign,

	r">>>"  => UnsignedRightShift,
	r">>="  => RightShiftAssign,
	r"<<="  => LeftShiftAssign,
	r"~=="  => ApproxEquals,
	r"<>="  => ThreeWayComp,
	r"..."  => Ellipsis,

	r".."   => DotDot,
	r"+="   => PlusAssign,
	r"-="   => MinusAssign,
	r"*="   => TimesAssign,
	r"/="   => DivideAssign,
	r"%="   => ModuloAssign,
	r"&="   => BitwiseAndAssign,
	r"^="   => BitwiseXorAssign,
	r"|="   => BitwiseOrAssign,
	r">>"   => RightShift,
	r"<<"   => LeftShift,
	r"=="   => Equals,
	r"!="   => NotEquals,
	r">="   => GreaterThanEquals,
	r"<="   => LessThanEquals,
	r"&&"   => LogicalAnd,
	r"||"   => LogicalOr,
	r"++"   => Increment,
	r"--"   => Decrement,
	r"::"   => DoubleColon,
	r"->"   => Arrow,
	r"**"   => Raise,

	r"+"    => Plus,
	r"-"    => Minus,
	r"*"    => Times,
	r"/"    => Divide,
	r"%"    => Modulo,
	r"&"    => BitwiseAnd,
	r"^"    => BitwiseXor,
	r"|"    => BitwiseOr,
	r"~"    => BitwiseNot,
	r"!"    => LogicalNot,
	r"("    => LeftRound,
	r")"    => RightRound,
	r"["    => LeftSquare,
	r"]"    => RightSquare,
	r"{"    => LeftCurly,
	r"}"    => RightCurly,
	r"<"    => LeftAngle,
	r">"    => RightAngle,
	r"="    => Assign,
	r":"    => Colon,
	r";"    => Semicolon,
	r"."    => Dot,
	r","    => Comma,
	r"?"    => QuestionMark,
	r"#"    => Hash,
	r"@"    => AtSign
});

rule_grouping_type!(Keyword, keyword, match_keyword {
	"break"        => Break,
	"case"         => Case,
	"const"        => Const,
	"continue"     => Continue,
	"default"      => Default,
	"do"           => Do,
	"else"         => Else,
	"for"          => For,
	"goto"         => Goto,
	"if"           => If,
	"return"       => Return,
	"switch"       => Switch,
	"until"        => Until,
	"volatile"     => Volatile,
	"while"        => While,

	"char"         => Char,
	"long"         => Long,
	"ulong"        => ULong,
	"struct"       => Struct,
	"class"        => Class,
	"mixin"        => Mixin,
	"enum"         => Enum,
	"in"           => In,
	"sizeof"       => SizeOf,
	"alignof"      => AlignOf,

	"abstract"     => Abstract,
	"foreach"      => ForEach,
	"true"         => True,
	"false"        => False,
	"auto"         => Auto,
	"flagdef"      => FlagDef,
	"native"       => Native,
	"var"          => Var,
	"out"          => Out,
	"static"       => Static,
	"transient"    => Transient,
	"final"        => Final,
	"extend"       => Extend,
	"protected"    => Protected,
	"private"      => Private,
	"dot"          => Dot,
	"cross"        => Cross,
	"virtual"      => Virtual,
	"override"     => Override,
	"vararg"       => VarArg,
	"ui"           => UI,
	"play"         => Play,
	"clearscope"   => ClearScope,
	"virtualscope" => VirtualScope,
	"super"        => Super,
	"stop"         => Stop,
	"null"         => Null,

	"is"           => Is,
	"replaces"     => Replaces,
	"states"       => States,
	"meta"         => Meta,
	"deprecated"   => Deprecated,
	"version"      => Version,
	"action"       => Action,
	"readonly"     => ReadOnly,
	"internal"     => Internal,
	"let"          => Let
});

fn unescape(s: &'_ str) -> Cow<'_, str> {
	if s.contains('\\') {
		let s = s.replace(r#"\""#, "\"");
		let s = s.replace(r"\\", "\\");
		let s = s.replace("\\\n", "\n");
		let s = s.replace(r"\a", "\x07");
		let s = s.replace(r"\b", "\x08");
		let s = s.replace(r"\c", "\x1C");
		let s = s.replace(r"\f", "\x0C");
		let s = s.replace(r"\n", "\n");
		let s = s.replace(r"\t", "\t");
		let s = s.replace(r"\r", "\r");
		let s = s.replace(r"\v", "\x0B");
		let s = s.replace(r"\?", "?");
		Cow::Owned(s)
	} else {
		Cow::Borrowed(s)
	}
}

#[derive(Debug)]
pub struct Tokenizer<'src> {
	file: FileIndex,
	text: &'src str,
	current_range: Range<usize>,
	remaining: CharIndices<'src>,
	states_mode: bool,
	peeked: VecDeque<Option<Token<'src>>>,
}

impl<'src> Tokenizer<'src> {
	pub fn new(file: FileIndex, text: &'src str) -> Self {
		Self {
			file,
			text,
			current_range: 0..0,
			remaining: text.char_indices(),
			states_mode: false,
			peeked: VecDeque::new(),
		}
	}

	pub fn set_states_mode(&mut self, mode: bool) {
		if !self.peeked.is_empty() {
			panic!("set_states_mode called while in a peeked state");
		}
		self.states_mode = mode;
	}

	fn reset(&mut self) {
		match self.remaining.clone().next() {
			Some((i, _)) => {
				self.current_range = i..i;
			}
			None => {
				self.current_range = self.text.len()..self.text.len();
			}
		}
	}

	fn first(&self) -> Option<char> {
		self.remaining.clone().next().map(|(_, c)| c)
	}
	fn second(&self) -> Option<char> {
		self.remaining.clone().nth(1).map(|(_, c)| c)
	}

	fn bump(&mut self) -> Option<char> {
		let next = self.remaining.next();
		match next {
			Some((_, c)) => {
				self.current_range.end += c.len_utf8();
				Some(c)
			}
			None => {
				self.current_range = self.current_range.start..self.text.len();
				None
			}
		}
	}

	fn ident_or_keyword(&mut self) -> Token<'src> {
		loop {
			match self.first() {
				Some(c)
					if matches!(
						c,
						'a'..='z' | 'A'..='Z' | '0'..='9' | '_'
					) =>
				{
					self.bump();
				}
				_ => {
					let res = &self.text[self.current_range.clone()];
					let data = if let Some(k) = match_keyword(res) {
						TokenData::Keyword(k)
					} else {
						TokenData::Identifier(res)
					};
					break Token {
						original: res,
						data,
					};
				}
			}
		}
	}

	fn hex_digits(&mut self) {
		loop {
			match self.first() {
				Some(c)
					if matches!(
						c,
						'0'..='9' | 'a'..='f' | 'A'..='F'
					) =>
				{
					self.bump();
				}
				_ => {
					break;
				}
			}
		}
	}
	fn dec_digits(&mut self) -> bool {
		let mut seen = false;
		loop {
			match self.first() {
				Some(c) if matches!(c, '0'..='9') => {
					self.bump();
					seen = true;
				}
				_ => {
					break seen;
				}
			}
		}
	}
	fn exponent(&mut self) -> Result<(), ()> {
		if matches!(self.first(), Some('e') | Some('E')) {
			self.bump();
			if matches!(self.first(), Some('+') | Some('-')) {
				self.bump();
			}
			if !self.dec_digits() {
				Err(())
			} else {
				Ok(())
			}
		} else {
			Ok(())
		}
	}
	fn float_suffix(&mut self) -> bool {
		if matches!(self.first(), Some('f') | Some('F')) {
			self.bump();
			false
		} else {
			true
		}
	}
	fn int_suffix(&mut self) -> (bool, bool) {
		let mut long = false;
		let mut unsigned = false;
		for _ in 0..2 {
			if matches!(self.first(), Some('l') | Some('L') | Some('u') | Some('U')) {
				if matches!(self.first(), Some('l') | Some('L')) {
					long = true;
				}
				if matches!(self.first(), Some('u') | Some('U')) {
					unsigned = true;
				}
				self.bump();
			}
		}
		(long, unsigned)
	}

	fn number(&mut self, errs: &mut Vec<ParsingError>, first: char) -> Token<'src> {
		if first == '0' {
			if let Some('x') = self.first() {
				self.bump();
				self.hex_digits();
				let val = u64::from_str_radix(
					&self.text[self.current_range.start + 2..self.current_range.end],
					16,
				)
				.unwrap_or(u64::max_value());
				let (long, unsigned) = self.int_suffix();
				let res = &self.text[self.current_range.clone()];
				let data = TokenData::Int {
					val,
					long,
					unsigned,
				};
				return Token {
					original: res,
					data,
				};
			}
		}
		let mut float = first == '.';
		self.dec_digits();
		if !float && matches!(self.first(), Some('.')) {
			self.bump();
			float = true;
			self.dec_digits();
		}
		if matches!(self.first(), Some('e') | Some('E')) {
			float = true;
		}
		if float {
			let (val, double) = match self.exponent() {
				Ok(()) => {
					let val = self.text[self.current_range.clone()]
						.parse::<f64>()
						.unwrap_or(f64::INFINITY);
					let double = self.float_suffix();
					(val, double)
				}
				Err(()) => {
					let double = self.float_suffix();
					let s = &self.text[self.current_range.clone()];
					let err = ParsingError {
						level: ParsingErrorLevel::Error,
						msg: "expected at least one digit in exponent".to_string(),
						main_spans: vec1::vec1![span(self.file, self.text, s)],
						info_spans: vec![],
					};
					errs.push(err);
					(0.0, double)
				}
			};
			let res = &self.text[self.current_range.clone()];
			let data = TokenData::Float { val, double };
			Token {
				original: res,
				data,
			}
		} else if first == '0' {
			// this intentionally emulates ZScript's broken grammar rule for octal literals
			let s = &self.text[self.current_range.clone()];
			let i = s
				.find(|c: char| !('0'..='7').contains(&c))
				.unwrap_or(s.len());
			let s = &s[..i];
			let val = u64::from_str_radix(s, 8).unwrap_or(u64::max_value());

			let (long, unsigned) = self.int_suffix();
			let res = &self.text[self.current_range.clone()];
			let data = TokenData::Int {
				val,
				long,
				unsigned,
			};
			Token {
				original: res,
				data,
			}
		} else {
			let val = (&self.text[self.current_range.clone()])
				.parse::<u64>()
				.unwrap_or(u64::max_value());
			let (long, unsigned) = self.int_suffix();
			let res = &self.text[self.current_range.clone()];
			let data = TokenData::Int {
				val,
				long,
				unsigned,
			};
			Token {
				original: res,
				data,
			}
		}
	}

	fn punc_glue(&mut self, punc: Punctuation) -> Option<Punctuation> {
		use Punctuation::*;
		let first = self.first()?;
		let ret = match (punc, first) {
			(UnsignedRightShift, '=') => UnsignedRightShiftAssign,

			(RightShift, '>') => UnsignedRightShift,
			(RightShift, '=') => RightShiftAssign,
			(LeftShift, '=') => LeftShiftAssign,
			(DotDot, '.') => Ellipsis,

			(Dot, '.') => DotDot,
			(Plus, '=') => PlusAssign,
			(Minus, '=') => MinusAssign,
			(Times, '=') => TimesAssign,
			(Divide, '=') => DivideAssign,
			(Modulo, '=') => ModuloAssign,
			(BitwiseAnd, '=') => BitwiseAndAssign,
			(BitwiseXor, '=') => BitwiseXorAssign,
			(BitwiseOr, '=') => BitwiseOrAssign,
			(RightAngle, '>') => RightShift,
			(LeftAngle, '<') => LeftShift,
			(Assign, '=') => Equals,
			(LogicalNot, '=') => NotEquals,
			(RightAngle, '=') => GreaterThanEquals,
			(LeftAngle, '=') => LessThanEquals,
			(BitwiseAnd, '&') => LogicalAnd,
			(BitwiseOr, '|') => LogicalOr,
			(Plus, '+') => Increment,
			(Minus, '-') => Decrement,
			(Colon, ':') => DoubleColon,
			(Minus, '>') => Arrow,
			(Times, '*') => Raise,

			_ => {
				let second = self.second()?;
				let ret = match (punc, first, second) {
					(BitwiseNot, '=', '=') => ApproxEquals,
					(LeftAngle, '>', '=') => ThreeWayComp,
					_ => {
						return None;
					}
				};
				self.bump();
				self.bump();
				return Some(ret);
			}
		};
		self.bump();
		Some(ret)
	}

	fn punc(&mut self, first: Punctuation, glue: bool) -> Token<'src> {
		let mut ret = first;
		if glue {
			while let Some(p) = self.punc_glue(ret) {
				ret = p;
			}
		}
		let res = &self.text[self.current_range.clone()];
		let data = TokenData::Punctuation(ret);
		Token {
			original: res,
			data,
		}
	}

	fn name(&mut self, errs: &mut Vec<ParsingError>) -> Token<'src> {
		loop {
			match self.first() {
				Some('\'') => {
					self.bump();
					let res = &self.text[self.current_range.clone()];
					let data = TokenData::Name(
						&self.text[self.current_range.start + 1..self.current_range.end - 1],
					);
					break Token {
						original: res,
						data,
					};
				}
				Some('\n') | None => {
					let res = &self.text[self.current_range.clone()];
					let err = ParsingError {
						level: ParsingErrorLevel::Error,
						msg: "unterminated name constant".to_string(),
						main_spans: vec1::vec1![span(self.file, self.text, res)],
						info_spans: vec![],
					};
					errs.push(err);
					let data = TokenData::Name(
						&self.text[self.current_range.start + 1..self.current_range.end],
					);
					self.bump();
					break Token {
						original: res,
						data,
					};
				}
				_ => {
					self.bump();
				}
			}
		}
	}

	fn string(&mut self, errs: &mut Vec<ParsingError>) -> Token<'src> {
		let mut needs_unescape = false;
		loop {
			match self.first() {
				Some('\"') => {
					self.bump();
					let res = &self.text[self.current_range.clone()];
					let s = &self.text[self.current_range.start + 1..self.current_range.end - 1];
					let data = TokenData::String(if needs_unescape {
						unescape(s)
					} else {
						Cow::from(s)
					});
					break Token {
						original: res,
						data,
					};
				}
				Some('\\') => {
					self.bump();
					if matches!(self.first(), Some('\"')) {
						self.bump();
					}
					needs_unescape = true;
				}
				None => {
					self.bump();
					let res = &self.text[self.current_range.clone()];
					let err = ParsingError {
						level: ParsingErrorLevel::Error,
						msg: "unterminated string constant".to_string(),
						main_spans: vec1::vec1![span(self.file, self.text, res)],
						info_spans: vec![],
					};
					errs.push(err);
					let s = &self.text[self.current_range.start + 1..self.current_range.end - 1];
					let data = TokenData::String(if needs_unescape {
						unescape(s)
					} else {
						Cow::from(s)
					});
					break Token {
						original: res,
						data,
					};
				}
				_ => {
					self.bump();
				}
			}
		}
	}

	fn matches_nws(&mut self) -> bool {
		match self.first() {
			Some('/')
				if matches!(
					self.second(),
					Some('\u{0001}'..=' ')
						| Some('"') | Some(':') | Some(';')
						| Some('}') | Some('*') | Some('/')
						| None
				) =>
			{
				false
			}
			Some('\u{0001}'..=' ') | Some('"') | Some(':') | Some(';') | Some('}') | None => false,
			_ => true,
		}
	}

	fn nws(&mut self) -> Token<'src> {
		loop {
			if self.matches_nws() {
				self.bump();
			} else {
				let res = &self.text[self.current_range.clone()];
				let data =
					TokenData::NonWhitespace(Cow::from(&self.text[self.current_range.clone()]));
				break Token {
					original: res,
					data,
				};
			}
		}
	}

	fn line_comment(&mut self) -> Option<Token<'src>> {
		let doc = if matches!(self.first(), Some('/')) {
			self.bump();
			true
		} else {
			false
		};
		loop {
			match self.bump() {
				Some('\n') | None => {
					break;
				}
				_ => {}
			}
		}
		if doc {
			let res = &self.text[self.current_range.clone()];
			let data = TokenData::DocComment(
				&self.text[self.current_range.start + 3..self.current_range.end],
			);
			Some(Token {
				original: res,
				data,
			})
		} else {
			None
		}
	}

	fn block_comment(&mut self, errs: &mut Vec<ParsingError>) {
		loop {
			match self.bump() {
				Some('*') => {
					if matches!(self.first(), Some('/')) {
						self.bump();
						break;
					}
				}
				None => {
					let res = &self.text[self.current_range.clone()];
					let err = ParsingError {
						level: ParsingErrorLevel::Error,
						msg: "unterminated block comment".to_string(),
						main_spans: vec1::vec1![span(self.file, self.text, res)],
						info_spans: vec![],
					};
					errs.push(err);
					break;
				}
				_ => {}
			}
		}
	}

	fn invalid_start(&self, errs: &mut Vec<ParsingError>) {
		let s = &self.text[self.current_range.clone()];
		let c = s.chars().next().unwrap();
		let err = ParsingError {
			level: ParsingErrorLevel::Error,
			msg: format!("unknown token start U+{:04X}", c as u32),
			main_spans: vec1::vec1![span(self.file, self.text, s)],
			info_spans: vec![],
		};
		errs.push(err);
	}

	fn next_internal(&mut self, errs: &mut Vec<ParsingError>) -> Option<Token<'src>> {
		if self.states_mode {
			'outer_states: loop {
				self.reset();
				let start = self.bump()?;
				break Some(match start {
					'\u{0001}'..=' ' => {
						continue;
					}
					'#' => {
						if self
							.remaining
							.as_str()
							.starts_with_ignore_ascii_case("region")
						{
							self.current_range.end += "region".len();
							self.remaining.nth("region".len());
							loop {
								match self.bump() {
									Some('\n') | None => {
										continue 'outer_states;
									}
									_ => {}
								}
							}
						} else if self
							.remaining
							.as_str()
							.starts_with_ignore_ascii_case("endregion")
						{
							self.current_range.end += "endregion".len();
							self.remaining.nth("endregion".len());
							loop {
								match self.bump() {
									Some('\n') | None => {
										continue 'outer_states;
									}
									_ => {}
								}
							}
						} else {
							self.nws()
						}
					}
					'/' => {
						if matches!(self.first(), Some('/')) {
							self.bump();
							if let Some(t) = self.line_comment() {
								return Some(t);
							};
							continue 'outer_states;
						} else if matches!(self.first(), Some('*')) {
							self.bump();
							self.block_comment(errs);
							continue 'outer_states;
						} else if self.matches_nws() {
							self.nws()
						} else {
							self.invalid_start(errs);
							continue 'outer_states;
						}
					}
					'}' => self.punc(Punctuation::RightCurly, false),
					':' => self.punc(Punctuation::Colon, false),
					';' => self.punc(Punctuation::Semicolon, false),

					'\"' => {
						if let Token {
							original,
							data: TokenData::String(s),
						} = self.string(errs)
						{
							Token {
								original,
								data: TokenData::NonWhitespace(s),
							}
						} else {
							unreachable!()
						}
					}

					c if !matches!(c, '\u{0001}'..=' ' | '"' | ':' | ';' | '}') => self.nws(),

					_ => {
						self.invalid_start(errs);
						continue 'outer_states;
					}
				});
			}
		} else {
			'outer: loop {
				self.reset();
				let start = self.bump()?;
				break Some(match start {
					'\u{0001}'..=' ' => {
						continue 'outer;
					}
					'a'..='z' | 'A'..='Z' | '_' => self.ident_or_keyword(),
					'0'..='9' => self.number(errs, start),
					'#' => {
						if self
							.remaining
							.as_str()
							.starts_with_ignore_ascii_case("include")
						{
							self.current_range.end += "include".len();
							self.remaining.nth("include".len());
							let res = &self.text[self.current_range.clone()];
							let data = TokenData::Include;
							Token {
								original: res,
								data,
							}
						} else if self
							.remaining
							.as_str()
							.starts_with_ignore_ascii_case("region")
						{
							self.current_range.end += "region".len();
							self.remaining.nth("region".len());
							loop {
								match self.bump() {
									Some('\n') | None => {
										continue 'outer;
									}
									_ => {}
								}
							}
						} else if self
							.remaining
							.as_str()
							.starts_with_ignore_ascii_case("endregion")
						{
							self.current_range.end += "endregion".len();
							self.remaining.nth("endregion".len());
							loop {
								match self.bump() {
									Some('\n') | None => {
										continue 'outer;
									}
									_ => {}
								}
							}
						} else {
							self.punc(Punctuation::Hash, true)
						}
					}
					'.' => {
						if matches!(self.first(), Some('0'..='9')) {
							self.number(errs, start)
						} else {
							self.punc(Punctuation::Dot, true)
						}
					}
					'/' => {
						if matches!(self.first(), Some('/')) {
							self.bump();
							if let Some(t) = self.line_comment() {
								return Some(t);
							};
							continue 'outer;
						} else if matches!(self.first(), Some('*')) {
							self.bump();
							self.block_comment(errs);
							continue 'outer;
						} else {
							self.punc(Punctuation::Divide, true)
						}
					}
					'+' => self.punc(Punctuation::Plus, true),
					'-' => self.punc(Punctuation::Minus, true),
					'*' => self.punc(Punctuation::Times, true),
					'%' => self.punc(Punctuation::Modulo, true),
					'&' => self.punc(Punctuation::BitwiseAnd, true),
					'^' => self.punc(Punctuation::BitwiseXor, true),
					'|' => self.punc(Punctuation::BitwiseOr, true),
					'~' => self.punc(Punctuation::BitwiseNot, true),
					'!' => self.punc(Punctuation::LogicalNot, true),
					'(' => self.punc(Punctuation::LeftRound, true),
					')' => self.punc(Punctuation::RightRound, true),
					'[' => self.punc(Punctuation::LeftSquare, true),
					']' => self.punc(Punctuation::RightSquare, true),
					'{' => self.punc(Punctuation::LeftCurly, true),
					'}' => self.punc(Punctuation::RightCurly, true),
					'<' => self.punc(Punctuation::LeftAngle, true),
					'>' => self.punc(Punctuation::RightAngle, true),
					'=' => self.punc(Punctuation::Assign, true),
					':' => self.punc(Punctuation::Colon, true),
					';' => self.punc(Punctuation::Semicolon, true),
					',' => self.punc(Punctuation::Comma, true),
					'?' => self.punc(Punctuation::QuestionMark, true),
					'@' => self.punc(Punctuation::AtSign, true),

					'\'' => self.name(errs),
					'\"' => self.string(errs),

					_ => {
						self.invalid_start(errs);
						continue 'outer;
					}
				});
			}
		}
	}

	fn doc_filter(t: &Option<Token>) -> bool {
		matches!(
			t,
			Some(Token {
				data: TokenData::DocComment(_),
				..
			})
		)
	}

	pub(super) fn next_no_doc(&mut self, errs: &mut Vec<ParsingError>) -> Option<Token<'src>> {
		while let Some(p) = self.peeked.pop_front() {
			if Self::doc_filter(&p) {
				continue;
			}
			return p;
		}
		loop {
			let r = self.next_internal(errs);
			if Self::doc_filter(&r) {
				continue;
			}
			break r;
		}
	}

	pub(super) fn peek_no_doc(&mut self, errs: &mut Vec<ParsingError>) -> &Option<Token<'src>> {
		while !self.peeked.iter().any(|t| !Self::doc_filter(t)) {
			let r = self.next_internal(errs);
			self.peeked.push_back(r);
		}
		self.peeked.iter().find(|t| !Self::doc_filter(t)).unwrap()
	}

	pub(super) fn peek_twice_no_doc(
		&mut self,
		errs: &mut Vec<ParsingError>,
	) -> &Option<Token<'src>> {
		while self
			.peeked
			.iter()
			.filter(|t| !Self::doc_filter(t))
			.nth(1)
			.is_none()
		{
			let r = self.next_internal(errs);
			self.peeked.push_back(r);
		}
		self.peeked
			.iter()
			.filter(|t| !Self::doc_filter(t))
			.nth(1)
			.unwrap()
	}

	pub(super) fn next_doc(&mut self, errs: &mut Vec<ParsingError>) -> Option<Token<'src>> {
		if let Some(p) = self.peeked.pop_front() {
			return p;
		}
		self.next_internal(errs)
	}

	pub(super) fn peek_doc(&mut self, errs: &mut Vec<ParsingError>) -> &Option<Token<'src>> {
		if self.peeked.get(0).is_none() {
			let r = self.next_internal(errs);
			self.peeked.push_back(r);
		}
		&self.peeked[0]
	}
}

#[cfg(test)]
mod test {
	use super::*;

	fn dummy_file() -> FileIndex {
		FileIndex(0)
	}

	#[test]
	fn test_tokenizer() {
		let s = r#"
            something _s "Hi!" // hello!
            /* hi
             * end of comment */
            #region Some Stuff
            'Hello...'424l += 0x7453 0.42 @
            #endregion
        "#;
		let mut errs = vec![];
		let mut tok = Tokenizer::new(dummy_file(), s);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: "something",
				data: TokenData::Identifier("something")
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: "_s",
				data: TokenData::Identifier("_s")
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#""Hi!""#,
				data: TokenData::String(Cow::from("Hi!"))
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"'Hello...'"#,
				data: TokenData::Name("Hello...")
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"424l"#,
				data: TokenData::Int {
					val: 424,
					long: true,
					unsigned: false
				}
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"+="#,
				data: TokenData::Punctuation(Punctuation::PlusAssign)
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"0x7453"#,
				data: TokenData::Int {
					val: 0x7453,
					long: false,
					unsigned: false
				}
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"0.42"#,
				data: TokenData::Float {
					val: 0.42,
					double: true
				}
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"@"#,
				data: TokenData::Punctuation(Punctuation::AtSign)
			}
		);
		assert!(tok.next_no_doc(&mut errs).is_none());
	}

	#[test]
	fn test_tokenizer_states() {
		let s = r#"HI#24 AA##A /A; A// comment
            A/B/* comment again */
            "Hello\t...""#;
		let mut tok = Tokenizer::new(dummy_file(), s);
		tok.set_states_mode(true);
		let mut errs = vec![];
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"HI#24"#,
				data: TokenData::NonWhitespace(Cow::from("HI#24"))
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"AA##A"#,
				data: TokenData::NonWhitespace(Cow::from("AA##A"))
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"/A"#,
				data: TokenData::NonWhitespace(Cow::from("/A"))
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#";"#,
				data: TokenData::Punctuation(Punctuation::Semicolon)
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"A"#,
				data: TokenData::NonWhitespace(Cow::from("A"))
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#"A/B"#,
				data: TokenData::NonWhitespace(Cow::from("A/B"))
			}
		);
		assert_eq!(
			tok.next_no_doc(&mut errs).unwrap(),
			Token {
				original: r#""Hello\t...""#,
				data: TokenData::NonWhitespace(Cow::from("Hello\t..."))
			}
		);
		assert!(tok.next_no_doc(&mut errs).is_none());
	}
}
