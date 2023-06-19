//! A general-purpose LL parser.
//!
//! This parser design is derived from
//! https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html.

use std::cell::Cell;

use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder, SyntaxKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Checkpoint(usize);

/// Ties a [`rowan::Language`] to a [`logos::Logos`] token.
pub trait LangExt: rowan::Language {
	type Token: 'static
		+ for<'i> logos::Logos<'i, Source = str, Error = Self::Token>
		+ Eq
		+ Copy
		+ Default;

	const EOF: Self::Token;
	const ERR_NODE: Self::Kind;
}

/// A general-purpose LL parser.
pub struct Parser<'i, L: LangExt> {
	source: &'i str,
	tokens: Vec<Lexeme<L>>,
	pos: usize,
	fuel: Cell<u32>,
	events: Vec<Event>,
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
		}
	}

	pub fn open(&mut self) -> Checkpoint {
		let checkpoint = Checkpoint(self.events.len());
		self.events.push(Event::Open(L::kind_to_raw(L::ERR_NODE)));
		checkpoint
	}

	pub fn close(&mut self, checkpoint: Checkpoint, syn: L::Kind) -> Checkpoint {
		self.events[checkpoint.0] = Event::Open(L::kind_to_raw(syn));
		self.events.push(Event::Close);
		Checkpoint(checkpoint.0)
	}

	pub fn open_before(&mut self, checkpoint: Checkpoint) -> Checkpoint {
		let ret = Checkpoint(checkpoint.0);
		self.events
			.insert(checkpoint.0, Event::Open(L::kind_to_raw(L::ERR_NODE)));
		ret
	}

	pub fn advance(&mut self, syn: L::Kind) {
		assert!(!self.eof());
		self.fuel.set(256);
		self.events.push(Event::Advance(L::kind_to_raw(syn)));
		self.pos += 1;
	}

	#[must_use]
	pub fn eof(&self) -> bool {
		self.pos == self.tokens.len()
	}

	#[must_use]
	pub fn nth(&self, lookahead: usize) -> L::Token {
		if self.fuel.get() == 0 {
			panic!("parser is not advancing")
		}

		self.fuel.set(self.fuel.get() - 1);

		self.tokens
			.get(self.pos + lookahead)
			.map_or(L::EOF, |lexeme| lexeme.kind)
	}

	#[must_use]
	pub fn at(&self, token: L::Token) -> bool {
		self.nth(0) == token
	}

	#[must_use]
	pub fn at_any(&self, choices: &'static [L::Token]) -> bool {
		let token = self.nth(0);
		choices.iter().any(|t| *t == token)
	}

	#[must_use]
	pub fn at_str_nc(&self, token: L::Token, string: &'static str) -> bool {
		let eof = Lexeme {
			kind: L::EOF,
			span: self.source.len()..self.source.len(),
		};

		let lexeme = self.tokens.get(self.pos).unwrap_or(&eof);

		lexeme.kind == token && self.source[lexeme.span.clone()].eq_ignore_ascii_case(string)
	}

	#[must_use]
	pub fn at_if(&self, predicate: fn(L::Token) -> bool) -> bool {
		predicate(self.nth(0))
	}

	pub fn eat(&mut self, token: L::Token, syn: L::Kind) -> bool {
		if self.at(token) {
			self.advance(syn);
			true
		} else {
			false
		}
	}

	#[must_use]
	pub fn eat_if(&mut self, predicate: fn(L::Token) -> bool, syn: L::Kind) -> bool {
		if self.at_if(predicate) {
			self.advance(syn);
			true
		} else {
			false
		}
	}

	pub fn expect(&mut self, token: L::Token, syn: L::Kind) {
		if self.eat(token, syn) {
			return;
		}

		unimplemented!("error handling unimplemented")
	}

	pub fn expect_str_nc(&mut self, token: L::Token, string: &'static str, syn: L::Kind) {
		let eof = Lexeme {
			kind: L::EOF,
			span: self.source.len()..self.source.len(),
		};

		let lexeme = self.tokens.get(self.pos).unwrap_or(&eof);

		if lexeme.kind == token && self.source[lexeme.span.clone()].eq_ignore_ascii_case(string) {
			self.advance(syn);
			return;
		}

		unimplemented!("error handling unimplemented")
	}

	pub fn expect_if(&mut self, predicate: fn(L::Token) -> bool, syn: L::Kind) {
		if self.eat_if(predicate, syn) {
			return;
		}

		unimplemented!("error handling unimplemented")
	}

	pub fn expect_any(&mut self, choices: &'static [(L::Token, L::Kind)]) {
		for choice in choices {
			if self.eat(choice.0, choice.1) {
				return;
			}
		}

		unimplemented!("error handling unimplemented")
	}

	pub fn advance_with_error(&mut self, syn: L::Kind) {
		let ckpt = self.open();
		// TODO: Error handling goes here.
		self.advance(syn);
		self.close(ckpt, L::ERR_NODE);
	}

	#[must_use]
	pub fn finish(self) -> GreenNode {
		let mut tokens = self.tokens.into_iter();
		let mut builder = GreenNodeBuilder::new();

		for event in self.events {
			match event {
				Event::Open(syn) => {
					builder.start_node(syn);
				}
				Event::Close => {
					builder.finish_node();
				}
				Event::Advance(syn) => {
					let token = tokens.next().unwrap();
					builder.token(syn, &self.source[token.span]);
				}
			}
		}

		assert!(tokens.next().is_none(), "not all tokens were consumed");
		builder.finish()
	}
}

#[derive(Debug)]
struct Lexeme<L: LangExt> {
	kind: L::Token,
	span: logos::Span,
}

#[derive(Debug)]
enum Event {
	Open(SyntaxKind),
	Close,
	Advance(SyntaxKind),
}
