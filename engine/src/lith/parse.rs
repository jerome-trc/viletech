//! The [`chumsky`]-based parser. Takes [lexer output] and emits a [`ParseTree`].
//!
//! The parse tree unites a lossless syntax tree, a syntax zipper tree, and an AST
//! in the scheme described by the [`rowan`] library and used in rust-analyzer.
//! Generate one by calling [`parse`].
//! See [here] for details.
//!
//! [lexer output]: super::syn::lex
//! [here]: https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/syntax.md

use chumsky::{primitive as prim, Parser};
use rowan::{GreenNode, GreenToken, NodeOrToken, SyntaxNode};

use super::{
	ast,
	syn::{SyntaxKind, SyntaxKind as Syn, Token},
};

// Only use `Syn` shorthand for parser, not public interface

pub type Error = chumsky::prelude::Simple<Token>;

/// Represents a source file, or an REPL submission. It may not necessarily
/// represent valid Lith; it contains no semantic information and the parser
/// recovers when it encounters an error.
pub struct ParseTree {
	green: GreenNode,
	zipper: SyntaxNode<SyntaxKind>,
	/// Errors that occurred during parsing, but not during lexing.
	errors: Vec<Error>,
	ast: ast::Tree,
}

impl ParseTree {
	#[must_use]
	fn new(root: GreenNode, errors: Vec<Error>) -> Self {
		let zipper = SyntaxNode::new_root(root.clone());

		Self {
			green: root,
			zipper: zipper.clone(),
			errors,
			ast: ast::Tree::new(zipper),
		}
	}

	#[must_use]
	pub fn ast(&self) -> &ast::Tree {
		&self.ast
	}

	#[must_use]
	pub fn raw(&self) -> &GreenNode {
		&self.green
	}

	#[must_use]
	pub fn zipper(&self) -> &SyntaxNode<SyntaxKind> {
		&self.zipper
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		!self.errors.is_empty()
	}

	#[must_use]
	pub fn errors(&self) -> &[Error] {
		&self.errors
	}
}

/// This will return `None` if the given `source` has no tokens to parse.
#[must_use]
pub fn parse(source: &str, tokens: Vec<Token>) -> Option<ParseTree> {
	if tokens.is_empty() {
		return None;
	}

	let (root, errs) = parser(source).parse_recovery(tokens);

	root.map(|r| ParseTree::new(r, errs))
}

type Output = NodeOrToken<GreenNode, GreenToken>;

fn parser(src: &str) -> impl Parser<Token, GreenNode, Error = Error> + '_ {
	prim::choice((annotation(src),))
		.repeated()
		.map(|children| GreenNode::new(Syn::Root.into(), children))
}

fn annotation(src: &str) -> impl Parser<Token, Output, Error = Error> + '_ {
	just(Syn::At)
		.map(|tok| nvec(src, tok))
		.then(just(Syn::Bang).or_not())
		.map(|(vec, opt)| pushopt(src, vec, opt))
		.then(resolver(src))
		.map(|(vec, outp)| pushout(vec, outp))
		.map(|o| Output::Node(GreenNode::new(Syn::Annotation.into(), o)))
}

fn resolver(src: &str) -> impl Parser<Token, Output, Error = Error> + '_ {
	just(Syn::Colon2)
		.or_not()
		.map(|opt| nvecpot(src, opt))
		.then(resolver_part(src).repeated())
		.map(|(o, parts)| append(o, parts))
		.map(|o| node(Syn::Resolver, o))
}

fn resolver_part(src: &str) -> impl Parser<Token, Output, Error = Error> + '_ {
	just(Syn::Colon2)
		.then(just(Syn::Identifier))
		.map(|(delim, ident)| {
			Output::Node(GreenNode::new(
				Syn::ResolverPart.into(),
				[out_tok(src, delim), out_tok(src, ident)],
			))
		})
}

// Helpers /////////////////////////////////////////////////////////////////////

#[must_use]
fn just(syn: Syn) -> impl Parser<Token, Token, Error = Error> {
	prim::filter(move |tok: &Token| tok.kind == syn)
}

#[must_use]
fn nvec(src: &str, tok: Token) -> Vec<Output> {
	vec![Output::Token(GreenToken::new(
		tok.kind.into(),
		&src[tok.span],
	))]
}

#[must_use]
fn nvecpot(src: &str, opt: Option<Token>) -> Vec<Output> {
	match opt {
		Some(tok) => {
			vec![Output::Token(GreenToken::new(
				tok.kind.into(),
				&src[tok.span],
			))]
		}
		None => {
			vec![]
		}
	}
}

#[must_use]
fn pushout(mut vec: Vec<Output>, outp: Output) -> Vec<Output> {
	vec.push(outp);
	vec
}

#[must_use]
fn pushopt(src: &str, mut vec: Vec<Output>, opt: Option<Token>) -> Vec<Output> {
	if let Some(tok) = opt {
		let gtok = GreenToken::new(tok.kind.into(), &src[tok.span]);
		vec.push(Output::Token(gtok));
	}

	vec
}

#[must_use]
fn out_tok(src: &str, tok: Token) -> Output {
	Output::Token(GreenToken::new(tok.kind.into(), &src[tok.span]))
}

#[must_use]
fn append(mut to: Vec<Output>, mut from: Vec<Output>) -> Vec<Output> {
	to.append(&mut from);
	to
}

#[must_use]
fn node(kind: Syn, children: Vec<Output>) -> Output {
	Output::Node(GreenNode::new(kind.into(), children))
}
