//! Auto-formatting routines for all parts of the ZScript grammar.

mod expr;

#[cfg(test)]
mod test;

use rowan::GreenToken;

use crate::{
	formatting::{BraceStyle, FormatConfig, LineEnds, TabStyle},
	GreenElement,
};

use super::Syn;

pub use self::expr::*;

pub type AutoFormatter<'c> = crate::formatting::AutoFormatter<&'c Config, &'c Cache>;

#[derive(Debug)]
pub struct Config {
	pub common: FormatConfig,

	/// Whether the opening brace of the body of an actor state's anonymous
	/// action function should start on the same line or the next line.
	pub action_braces: BraceStyle,
	/// Whether the opening brace of a class definition block should
	/// start on the same line or the next line.
	pub class_braces: BraceStyle,
	/// Whether the opening brace of a default block (in an actor class definition)
	/// should start on the same line or the next line.
	pub default_braces: BraceStyle,
	/// How a block with no non-whitespace content should be formatted.
	/// [`BraceStyle::SameLine`] results in:
	///
	/// ```
	/// void EmptyFunction() {}
	/// ```
	///
	/// [`BraceStyle::NewLine`] results in:
	///
	/// ```
	/// void EmptyFunction()
	/// {
	/// }
	/// ```
	pub empty_braces: BraceStyle,
	/// Whether the opening brace of an enumeration definition block should
	/// start on the same line or the next line.
	pub enum_braces: BraceStyle,
	/// Whether the opening brace of a function body should
	/// start on the same line or the next line.
	pub function_braces: BraceStyle,
	/// Whether the opening brace of a compound statement belonging to a loop
	/// statement should start on the same line or the next line.
	pub loop_braces: BraceStyle,
	/// Whether the opening brace of a states block (in an actor class definition)
	/// should start on the same line or the next line.
	pub states_braces: BraceStyle,
	/// Whether the opening brace of a static constant statement's array initializer
	/// should start on the same line or the next line.
	pub static_const_braces: BraceStyle,
	/// Whether the opening brace of a struct definition block should
	/// start on the same line or the next line.
	pub struct_braces: BraceStyle,

	pub enum_trailing_comma: bool,
	pub static_const_brackets: StaticConstBrackets,
}

impl Config {
	/// Largely guided by <https://zdoom-docs.github.io/staging/Meta/Style.html>.
	#[must_use]
	pub fn new(line_ends: LineEnds) -> Self {
		Self {
			common: FormatConfig {
				tabs: TabStyle::Tabs,
				line_ends,
				max_line_len: 80,
			},

			action_braces: BraceStyle::NewLine,
			class_braces: BraceStyle::NewLine,
			default_braces: BraceStyle::NewLine,
			empty_braces: BraceStyle::SameLine,
			enum_braces: BraceStyle::NewLine,
			function_braces: BraceStyle::NewLine,
			loop_braces: BraceStyle::NewLine,
			states_braces: BraceStyle::NewLine,
			static_const_braces: BraceStyle::NewLine,
			struct_braces: BraceStyle::NewLine,

			enum_trailing_comma: true,
			static_const_brackets: StaticConstBrackets::AfterIdent,
		}
	}
}

/// See [`Config::static_const_brackets`].
/// Only involved in [`static_const_stat`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticConstBrackets {
	AfterIdent,
	BeforeIdent,
}

impl std::ops::Deref for Config {
	type Target = FormatConfig;

	fn deref(&self) -> &Self::Target {
		&self.common
	}
}

/// Pointers to commonly-used [`GreenToken`]s that can be cheaply cloned to avoid
/// allocating new memory for every instance of them.
///
/// This is a large structure and it is not very cheap to construct; it is recommended
/// that you allocate one on the heap ahead of time and re-use it for the duration
/// of your application's run-time.
#[derive(Debug)]
pub struct Cache {
	_cr: GreenElement,
	_crlf: GreenElement,
	_lf: GreenElement,
	spaces: [GreenElement; 3],
	_tabs: [GreenElement; 3],

	_angle_l: GreenElement,
	_angle_r: GreenElement,
	_at: GreenElement,
	_brace_l: GreenElement,
	_brace_r: GreenElement,
	_bracket_l: GreenElement,
	_bracket_r: GreenElement,
	_colon: GreenElement,
	_colon2: GreenElement,
	_comma: GreenElement,
	_dot: GreenElement,
	_eq: GreenElement,
	_minus: GreenElement,
	_paren_l: GreenElement,
	_paren_r: GreenElement,
	_plus: GreenElement,
	_question: GreenElement,
	_semicolon: GreenElement,
}

impl Cache {
	#[must_use]
	fn space(&self) -> GreenElement {
		self.spaces[0].clone()
	}
}

impl Default for Cache {
	fn default() -> Self {
		Self {
			_cr: GreenToken::new(Syn::Whitespace.into(), "\r").into(),
			_crlf: GreenToken::new(Syn::Whitespace.into(), "\r\n").into(),
			_lf: GreenToken::new(Syn::Whitespace.into(), "\n").into(),
			spaces: [
				GreenToken::new(Syn::Whitespace.into(), " ").into(),
				GreenToken::new(Syn::Whitespace.into(), "  ").into(),
				GreenToken::new(Syn::Whitespace.into(), "    ").into(),
			],
			_tabs: [
				GreenToken::new(Syn::Whitespace.into(), "\t").into(),
				GreenToken::new(Syn::Whitespace.into(), "\t\t").into(),
				GreenToken::new(Syn::Whitespace.into(), "\t\t\t\t").into(),
			],
			_angle_l: GreenToken::new(Syn::AngleL.into(), "<").into(),
			_angle_r: GreenToken::new(Syn::AngleR.into(), ">").into(),
			_at: GreenToken::new(Syn::At.into(), "@").into(),
			_brace_l: GreenToken::new(Syn::BraceL.into(), "{").into(),
			_brace_r: GreenToken::new(Syn::BraceR.into(), "}").into(),
			_bracket_l: GreenToken::new(Syn::BracketL.into(), "[").into(),
			_bracket_r: GreenToken::new(Syn::BracketR.into(), "]").into(),
			_colon: GreenToken::new(Syn::Colon.into(), ":").into(),
			_colon2: GreenToken::new(Syn::Colon2.into(), "::").into(),
			_comma: GreenToken::new(Syn::Comma.into(), ",").into(),
			_dot: GreenToken::new(Syn::Dot.into(), ".").into(),
			_eq: GreenToken::new(Syn::Eq.into(), "=").into(),
			_minus: GreenToken::new(Syn::Minus.into(), "-").into(),
			_paren_l: GreenToken::new(Syn::ParenL.into(), "(").into(),
			_paren_r: GreenToken::new(Syn::ParenR.into(), ")").into(),
			_plus: GreenToken::new(Syn::Plus.into(), "+").into(),
			_question: GreenToken::new(Syn::Question.into(), "?").into(),
			_semicolon: GreenToken::new(Syn::Semicolon.into(), ";").into(),
		}
	}
}
