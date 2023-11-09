//! A general-purpose LL parser.
//!
//! This design is derived from those presented in the following articles:
//! - <https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html>
//! - <https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html>

use std::cell::Cell;

use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder, NodeCache, SyntaxKind, TextRange, TextSize};

use crate::LangExt;

/// A general-purpose LL parser.
pub struct Parser<'i, L: LangExt> {
	source: &'i str,
	tokens: Vec<Lexeme<L>>,
	pos: usize,
	fuel: Cell<u32>,
	events: Vec<Event>,
	errors: Vec<Error<L>>,
}

impl<'i, L: LangExt> Parser<'i, L> {
	#[must_use]
	pub fn new(
		source: &'i str,
		extras: <<L as LangExt>::Token as logos::Logos<'i>>::Extras,
	) -> Self {
		Self {
			source,
			tokens: L::Token::lexer_with_extras(source, extras)
				.spanned()
				.map(|(result, span)| match result {
					Ok(t) => Lexeme { kind: t, span },
					Err(t) => Lexeme { kind: t, span },
				})
				.collect(),
			pos: 0,
			fuel: Cell::new(256),
			events: vec![],
			errors: vec![],
		}
	}

	/// Starts a new sub-tree. Also see [`Self::close`].
	#[must_use]
	pub fn open(&mut self) -> OpenMark {
		let checkpoint = OpenMark(self.events.len());
		self.events.push(Event::Open(L::kind_to_raw(L::ERR_NODE)));
		checkpoint
	}

	/// Also see [`Self::open`]. Will panic if no sub-trees are open.
	pub fn close(&mut self, mark: OpenMark, syn: L::Kind) -> CloseMark {
		self.events[mark.0] = Event::Open(L::kind_to_raw(syn));
		self.events.push(Event::Close);
		let ret = mark.0;
		std::mem::forget(mark);
		CloseMark(ret)
	}

	pub fn open_before(&mut self, mark: CloseMark) -> OpenMark {
		let ret = OpenMark(mark.0);
		self.events
			.insert(mark.0, Event::Open(L::kind_to_raw(L::ERR_NODE)));
		ret
	}

	/// Mind that this has to perform O(n) vector element shifting.
	pub fn cancel(&mut self, mark: OpenMark) {
		let i = mark.0;
		std::mem::forget(mark);
		self.events.remove(i);
	}

	pub fn advance(&mut self, syn: L::Kind) {
		assert!(!self.eof());
		self.fuel.set(256);
		self.events.push(Event::Advance(L::kind_to_raw(syn)));
		self.pos += 1;
	}

	pub fn advance_n(&mut self, syn: L::Kind, tokens: u8) {
		assert!(
			tokens >= 1,
			"`advance_n` was passed 0 at {:?} (`{}`)",
			self.nth_span(0),
			self.nth_slice(0)
		);

		self.fuel.set(256);
		self.events
			.push(Event::AdvanceN(L::kind_to_raw(syn), tokens));
		self.pos += tokens as usize;
	}

	#[must_use]
	pub fn eof(&self) -> bool {
		self.pos == self.tokens.len()
	}

	#[must_use]
	pub fn nth(&self, lookahead: usize) -> L::Token {
		if self.fuel.get() == 0 {
			panic!(
				"parser is not advancing (stuck at {:?})",
				self.tokens[self.pos].span
			)
		}

		self.fuel.set(self.fuel.get() - 1);

		self.tokens
			.get(self.pos + lookahead)
			.map_or(L::EOF, |lexeme| lexeme.kind)
	}

	#[must_use]
	pub fn nth_slice(&self, lookahead: usize) -> &str {
		&self.source[self.tokens[self.pos + lookahead].span.clone()]
	}

	#[must_use]
	pub fn nth_span(&self, lookahead: usize) -> logos::Span {
		self.tokens[self.pos + lookahead].span.clone()
	}

	/// Shorthand for `self.nth(0) == token`.
	#[must_use]
	pub fn at(&self, token: L::Token) -> bool {
		self.nth(0) == token
	}

	/// See [`Self::at`].
	#[must_use]
	pub fn at_any(&self, choices: &'static [L::Token]) -> bool {
		let token = self.nth(0);
		choices.iter().any(|t| *t == token)
	}

	/// Like [`Self::at`], but only matches `token` as long as it holds `string`,
	/// ASCII case-insensitively.
	///
	/// This is needed for (G)ZDoom DSLs, many of which are unspecified and use only an
	/// ad-hoc parser as DoomFront's reference implementation. Representing every niche
	/// keyword used by every one of these languages would add complexity to every parser
	/// (since each would have to treat foreign keywords as identifiers), so instead
	/// make the smaller languages look for their keywords through identifiers.
	#[must_use]
	pub fn at_str_nc(&self, token: L::Token, string: &'static str) -> bool {
		let eof = Lexeme {
			kind: L::EOF,
			span: self.source.len()..self.source.len(),
		};

		let lexeme = self.tokens.get(self.pos).unwrap_or(&eof);

		lexeme.kind == token && self.source[lexeme.span.clone()].eq_ignore_ascii_case(string)
	}

	/// See [`Self::at`].
	#[must_use]
	pub fn at_if(&self, predicate: fn(L::Token) -> bool) -> bool {
		predicate(self.nth(0))
	}

	/// If [`Self::at`] matches `token`, [`Self::advance`] with `syn`.
	pub fn eat(&mut self, token: L::Token, syn: L::Kind) -> bool {
		if self.at(token) {
			self.advance(syn);
			true
		} else {
			false
		}
	}

	/// Like [`Self::eat`] but checks [`Self::at`] on each choice in the given order.
	pub fn eat_any(&mut self, choices: &'static [(L::Token, L::Kind)]) -> bool {
		for choice in choices {
			if self.at(choice.0) {
				self.advance(choice.1);
				return true;
			}
		}

		false
	}

	/// Composes [`Self::eat`] and [`Self::at_str_nc`].
	#[must_use]
	pub fn eat_str_nc(&mut self, token: L::Token, string: &'static str, syn: L::Kind) -> bool {
		if self.at_str_nc(token, string) {
			self.advance(syn);
			return true;
		}

		false
	}

	/// Composes [`Self::eat`] and [`Self::at_if`].
	#[must_use]
	pub fn eat_if(&mut self, predicate: fn(L::Token) -> bool, syn: L::Kind) -> bool {
		if self.at_if(predicate) {
			self.advance(syn);
			true
		} else {
			false
		}
	}

	/// If [`Self::eat`] fails to consume `token`, raise an error.
	pub fn expect(&mut self, token: L::Token, syn: L::Kind, expected: ExpectedSets) {
		if self.eat(token, syn) {
			return;
		}

		self.raise(expected);
	}

	/// If [`Self::eat_str_nc`] fails to consume `token` corresponding to `string`
	/// ASCII-case insensitively, raise an error.
	pub fn expect_str_nc(
		&mut self,
		token: L::Token,
		string: &'static str,
		syn: L::Kind,
		expected: ExpectedSets,
	) {
		if self.eat_str_nc(token, string, syn) {
			return;
		}

		self.raise(expected);
	}

	/// Composes [`Self::expect`] and [`Self::eat_if`].
	pub fn expect_if(
		&mut self,
		predicate: fn(L::Token) -> bool,
		syn: L::Kind,
		expected: ExpectedSets,
	) {
		if self.eat_if(predicate, syn) {
			return;
		}

		self.raise(expected);
	}

	/// Composes [`Self::expect`] and [`Self::eat_any`].
	pub fn expect_any(&mut self, choices: &'static [(L::Token, L::Kind)], expected: ExpectedSets) {
		for choice in choices {
			if self.eat(choice.0, choice.1) {
				return;
			}
		}

		self.raise(expected);
	}

	/// Composes [`Self::expect_any`] and [`Self::expect_str_nc`].
	pub fn expect_any_str_nc(
		&mut self,
		choices: &'static [(L::Token, &'static str, L::Kind)],
		expected: ExpectedSets,
	) {
		for choice in choices {
			if self.eat_str_nc(choice.0, choice.1, choice.2) {
				return;
			}
		}

		self.raise(expected);
	}

	/// Put together tokens into one `syn`.
	/// If `advance_if` returns `false`, the merge loop breaks.
	pub fn merge(
		&mut self,
		syn: L::Kind,
		advance_if: fn(L::Token) -> bool,
		fallback: fn(L::Token) -> L::Kind,
		expected: ExpectedSets,
	) {
		let mut n = 0;

		loop {
			let token = self.nth(n);

			if advance_if(token) {
				n += 1;
			} else {
				break;
			}
		}

		if n > 0 {
			self.advance_n(syn, n as u8);
		} else {
			self.advance_with_error(fallback(self.nth(0)), expected);
		}
	}

	/// Looks ahead for the next token (which may be the EOF) for which `predicate`
	/// returns `true`. If `0` is passed, this starts at the current token.
	#[must_use]
	pub fn find(&self, offset: usize, predicate: fn(L::Token) -> bool) -> L::Token {
		if self.pos >= self.tokens.len() {
			return L::EOF;
		}

		self.tokens[(self.pos + offset)..]
			.iter()
			.find_map(|t| {
				if predicate(t.kind) {
					Some(t.kind)
				} else {
					None
				}
			})
			.unwrap_or(L::EOF)
	}

	fn raise(&mut self, expected: ExpectedSets) {
		self.errors.push(Error {
			expected,
			found: self.tokens.get(self.pos).cloned().unwrap_or(Lexeme {
				kind: L::EOF,
				span: self.source.len()..self.source.len(),
			}),
		});
	}

	/// [Opens](Self::open) a new error node, [advances](Self::advance) it with
	/// a `syn` token, and then [closes](Self::close) it.
	pub fn advance_with_error(&mut self, syn: L::Kind, expected: ExpectedSets) {
		let ckpt = self.open();
		self.raise(expected);

		if !self.eof() {
			self.advance(syn);
		}

		self.close(ckpt, L::ERR_NODE);
	}

	/// Raise an error and advance the token cursor (if not at the end-of-input).
	/// The sub-tree opened by `checkpoint` gets [closed](Self::close) with `err`.
	pub fn advance_err_and_close(
		&mut self,
		checkpoint: OpenMark,
		token: L::Kind,
		err: L::Kind,
		expected: ExpectedSets,
	) -> CloseMark {
		self.raise(expected);

		if !self.eof() {
			self.advance(token);
		}

		self.close(checkpoint, err)
	}

	/// Use when getting ready to open a new node to validate that the parser
	/// is currently at the first expected token of that node.
	pub fn assert_at(&self, token: L::Token)
	where
		L::Token: std::fmt::Debug,
	{
		assert!(
			self.at(token),
			"parser expected to be at a `{token:#?}`, but is at `{t:#?}`\r\n\
			(position {pos}, span {span:?}, slice: `{slice}`)",
			t = self.nth(0),
			pos = self.pos,
			span = self.nth_span(0),
			slice = self.nth_slice(0)
		)
	}

	/// Debug mode-only counterpart to [`Self::assert_at`].
	pub fn debug_assert_at(&self, token: L::Token)
	where
		L::Token: std::fmt::Debug,
	{
		#[cfg(debug_assertions)]
		self.assert_at(token);
		#[cfg(not(debug_assertions))]
		let _ = token;
	}

	/// See [`Self::assert_at`].
	pub fn assert_at_any(&self, choices: &'static [L::Token])
	where
		L::Token: std::fmt::Debug,
	{
		assert!(
			self.at_any(choices),
			"parser's current token did not pass a predicate (at `{t:#?}`)\r\n\
			(position {pos}, span {span:?}, slice: `{slice}`)",
			t = self.nth(0),
			pos = self.pos,
			span = self.nth_span(0),
			slice = self.nth_slice(0)
		);
	}

	/// Debug mode-only counterpart to [`Self::assert_at_any`].
	pub fn debug_assert_at_any(&self, choices: &'static [L::Token])
	where
		L::Token: std::fmt::Debug,
	{
		#[cfg(debug_assertions)]
		self.assert_at_any(choices);
		#[cfg(not(debug_assertions))]
		let _ = choices;
	}

	/// See [`Self::debug_assert_at`].
	pub fn assert_at_if(&self, predicate: fn(L::Token) -> bool)
	where
		L::Token: std::fmt::Debug,
	{
		assert!(
			predicate(self.nth(0)),
			"parser's current token did not pass a predicate (at `{t:#?}`)\r\n\
			(position {pos}, span {span:?}, slice: `{slice}`)",
			t = self.nth(0),
			pos = self.pos,
			span = self.nth_span(0),
			slice = self.nth_slice(0)
		);
	}

	/// Debug mode-only counterpart to [`Self::assert_at_if`].
	pub fn debug_assert_at_if(&self, predicate: fn(L::Token) -> bool)
	where
		L::Token: std::fmt::Debug,
	{
		#[cfg(debug_assertions)]
		self.assert_at_if(predicate);
		#[cfg(not(debug_assertions))]
		let _ = predicate;
	}

	/// Panics if an [opened] subtree was never [closed], or if no sub-trees
	/// were ever opened at all.
	///
	/// [opened]: Self::open
	/// [closed]: Self::close
	#[must_use]
	pub fn finish(self, cache: Option<&mut NodeCache>) -> (GreenNode, Vec<Error<L>>) {
		let mut tokens = self.tokens.into_iter();

		let mut builder = match cache {
			Some(c) => GreenNodeBuilder::with_cache(c),
			None => GreenNodeBuilder::new(),
		};

		for event in self.events {
			match event {
				Event::Advance(syn) => {
					let lexeme = tokens.next().unwrap();
					builder.token(syn, &self.source[lexeme.span]);
				}
				Event::Open(syn) => {
					builder.start_node(syn);
				}
				Event::Close => {
					builder.finish_node();
				}
				Event::AdvanceN(syn, 1) => {
					let lexeme = tokens.next().unwrap();
					builder.token(syn, &self.source[lexeme.span]);
				}
				Event::AdvanceN(syn, n) => {
					let start = tokens.next().unwrap().span.start;

					// An unconditional assertion in `Self::advance_n` protects
					// against `n` ever being less than one.
					for _ in 0..(n - 2) {
						let _ = tokens.next().unwrap();
					}

					let end = tokens.next().unwrap().span.end;
					builder.token(syn, &self.source[start..end]);
				}
			}
		}

		assert!(tokens.next().is_none(), "not all tokens were consumed");
		(builder.finish(), self.errors)
	}
}

/// See [`Parser::open`] and [`Parser::close`].
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct OpenMark(usize);

impl Drop for OpenMark {
	fn drop(&mut self) {
		panic!("an `OpenMark` was not consumed")
	}
}

/// See [`Parser::close`] and [`Parser::open_before`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CloseMark(usize);

/// A member of each [`Error`].
pub type ExpectedSets = &'static [&'static [&'static str]];

pub struct Error<L: LangExt> {
	expected: ExpectedSets,
	found: Lexeme<L>,
}

impl<L: LangExt> Error<L> {
	pub fn expected(&self) -> impl Iterator<Item = &'static str> {
		self.expected.iter().flat_map(|s| *s).copied()
	}

	#[must_use]
	pub fn found(&self) -> Lexeme<L> {
		self.found.clone()
	}
}

impl<L: LangExt> std::fmt::Display for Error<L>
where
	L::Token: std::fmt::Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"found {} at {:?} - expected one of the following: {}",
			self.found.kind,
			self.found.span,
			{
				let mut out = String::new();

				for e in self.expected() {
					out.push_str(e);
					out.push('/');
				}

				out.pop();
				out
			}
		)
	}
}

impl<L: LangExt> std::fmt::Debug for Error<L>
where
	L::Token: std::fmt::Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Error")
			.field("expected", &self.expected)
			.field("found", &self.found)
			.finish()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lexeme<L: LangExt> {
	kind: L::Token,
	span: logos::Span,
}

impl<L: LangExt> Lexeme<L> {
	#[must_use]
	pub fn token(&self) -> L::Token {
		self.kind
	}

	#[must_use]
	pub fn span(&self) -> logos::Span {
		self.span.clone()
	}

	/// Like [`Self::span`] but narrows down the start and end scalars to [`u32`].
	#[must_use]
	pub fn text_range(&self) -> TextRange {
		TextRange::new(
			TextSize::from(self.span.start as u32),
			TextSize::from(self.span.end as u32),
		)
	}
}

/// A generic Pratt precedence checker.
/// Returns `true` if `right` binds more strongly in an infix expression.
#[must_use]
pub fn pratt<L: LangExt>(left: L::Token, right: L::Token, precedence: &[&[L::Token]]) -> bool {
	let strength = |token| precedence.iter().position(|level| level.contains(&token));

	let Some(right_s) = strength(right) else {
		return false;
	};

	let Some(left_s) = strength(left) else {
		#[cfg(debug_assertions)]
		if left != L::EOF {
			panic!()
		}

		return true;
	};

	right_s > left_s
}

#[derive(Debug)]
enum Event {
	Open(SyntaxKind),
	Close,
	Advance(SyntaxKind),
	AdvanceN(SyntaxKind, u8),
}
