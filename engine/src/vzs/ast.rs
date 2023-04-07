//! Abstract syntax tree nodes.

mod expr;

use std::num::ParseIntError;

use bitflags::bitflags;
use doomfront::{
	rowan::{self, ast::AstNode},
	simple_astnode,
};

use crate::utils::string::unescape_char;

use super::{Syn, SyntaxNode, SyntaxToken};

pub use self::expr::*;

/// One of the top-level elements of a file or REPL input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Root {
	/// Only "inner" annotations are legal at the top level.
	Annotation(Annotation),
	Item(Item),
}

impl AstNode for Root {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::Annotation | Syn::FunctionDecl)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::Annotation => Some(Self::Annotation(Annotation(node))),
			Syn::FunctionDecl => Some(Self::Item(Item::FunctionDecl(FunctionDecl(node)))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Annotation(inner) => inner.syntax(),
			Self::Item(inner) => inner.syntax(),
		}
	}
}

/// Wraps a node tagged [`Syn::Annotation`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Annotation(SyntaxNode);

simple_astnode!(Syn, Annotation, Syn::Annotation);

impl Annotation {
	/// Returns `true` if this annotation uses the syntax `#![]` instead of `#[]`.
	#[must_use]
	pub fn is_inner(&self) -> bool {
		self.0.children_with_tokens().nth(1).unwrap().kind() == Syn::Bang
	}

	#[must_use]
	pub fn resolver(&self) -> Resolver {
		self.0.children().find_map(Resolver::cast).unwrap()
	}

	#[must_use]
	pub fn args(&self) -> Option<ArgList> {
		self.0.children().find_map(ArgList::cast)
	}
}

/// Wraps a node tagged [`Syn::Argument`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Argument(SyntaxNode);

simple_astnode!(Syn, Argument, Syn::Argument);

impl Argument {
	#[must_use]
	pub fn label(&self) -> Option<Label> {
		self.0
			.first_child()
			.filter(|node| node.kind() == Syn::Label)
			.map(Label)
	}

	#[must_use]
	pub fn expr(&self) -> Expression {
		Expression::cast(self.0.last_child().unwrap()).unwrap()
	}
}

/// Wraps a node tagged [`Syn::ArgList`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ArgList(SyntaxNode);

simple_astnode!(Syn, ArgList, Syn::ArgList);

impl ArgList {
	pub fn iter(&self) -> impl Iterator<Item = Argument> {
		self.0.children().filter_map(Argument::cast)
	}
}

/// Wraps a node tagged [`Syn::Block`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Block(SyntaxNode);

impl AstNode for Block {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		kind == Syn::Block
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		if node.kind() == Syn::Block {
			Some(Self(node))
		} else {
			None
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}
}

/// Wraps a node tagged [`Syn::DeclQualifiers`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct DeclQualifiers(SyntaxNode);

impl DeclQualifiers {
	#[must_use]
	pub fn as_flags(&self) -> DeclQualifierFlags {
		let mut ret = DeclQualifierFlags::empty();

		for n_or_t in self.0.children_with_tokens() {
			match n_or_t.kind() {
				Syn::KwAbstract => ret.insert(DeclQualifierFlags::ABSTRACT),
				Syn::KwCEval => ret.insert(DeclQualifierFlags::CEVAL),
				Syn::KwFinal => ret.insert(DeclQualifierFlags::FINAL),
				Syn::KwOverride => ret.insert(DeclQualifierFlags::OVERRIDE),
				Syn::KwPrivate => ret.insert(DeclQualifierFlags::PRIVATE),
				Syn::KwProtected => ret.insert(DeclQualifierFlags::PROTECTED),
				Syn::KwStatic => ret.insert(DeclQualifierFlags::STATIC),
				Syn::KwVirtual => ret.insert(DeclQualifierFlags::VIRTUAL),
				_ => {} // Whitespace or comment.
			}
		}

		ret
	}
}

simple_astnode!(Syn, DeclQualifiers, Syn::DeclQualifiers);

bitflags! {
	/// A more practical representation of [`DeclQualifiers`].
	pub struct DeclQualifierFlags: u8 {
		const ABSTRACT = 1 << 0;
		const CEVAL = 1 << 1;
		const FINAL = 1 << 2;
		const OVERRIDE = 1 << 3;
		const PRIVATE = 1 << 4;
		const PROTECTED = 1 << 5;
		const STATIC = 1 << 6;
		const VIRTUAL = 1 << 7;
	}
}

/// Wraps a node tagged [`Syn::FunctionDecl`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct FunctionDecl(SyntaxNode);

impl FunctionDecl {
	/// The identifier given to this function, after the return type specifier.
	#[must_use]
	pub fn name(&self) -> Name {
		self.0.children().find_map(Name::cast).unwrap()
	}

	/// Returns `None` if no qualifiers were written.
	#[must_use]
	pub fn qualifiers(&self) -> Option<DeclQualifiers> {
		self.0.children().find_map(DeclQualifiers::cast)
	}

	pub fn return_types(&self) -> impl Iterator<Item = TypeRef> {
		let rets = self
			.0
			.children()
			.find(|node| node.kind() == Syn::ReturnTypes)
			.unwrap();

		rets.children().filter_map(TypeRef::cast)
	}

	/// Built-in and native functions can only be declared, not defined,
	/// and thus they will have no blocks.
	#[must_use]
	pub fn body(&self) -> Option<Block> {
		self.0.children().find_map(Block::cast)
	}
}

simple_astnode!(Syn, FunctionDecl, Syn::FunctionDecl);

/// A function declaration, symbolic constant, type alias, class definition, et cetera.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
	FunctionDecl(FunctionDecl),
}

impl AstNode for Item {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		#[allow(clippy::match_like_matches_macro)]
		match kind {
			Syn::FunctionDecl => true,
			_ => false,
		}
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::FunctionDecl => Some(Self::FunctionDecl(FunctionDecl(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::FunctionDecl(inner) => inner.syntax(),
		}
	}
}

/// Wraps a node tagged [`Syn::Label`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Label(SyntaxNode);

simple_astnode!(Syn, Label, Syn::Label);

impl Label {
	/// Shorthand for
	/// `self.syntax().first_child_or_token().unwrap().into_token().unwrap()`.
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		self.0.first_child_or_token().unwrap().into_token().unwrap()
	}
}

/// Wraps a node tagged [`Syn::Literal`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Literal(SyntaxNode);

simple_astnode!(Syn, Literal, Syn::Literal);

impl Literal {
	#[must_use]
	pub fn token(&self) -> LitToken {
		LitToken(self.0.first_child_or_token().unwrap().into_token().unwrap())
	}
}

/// Wrapper around a [`SyntaxToken`] with convenience functions.
/// See [`Syn::Literal`]'s documentation to see possible token tags.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct LitToken(SyntaxToken);

impl LitToken {
	/// If this wraps a [`Syn::LitTrue`] or [`Syn::LitFalse`] token,
	/// this returns the corresponding value. Otherwise this returns `None`.
	#[must_use]
	pub fn bool(&self) -> Option<bool> {
		match self.0.kind() {
			Syn::LitTrue => Some(true),
			Syn::LitFalse => Some(false),
			_ => None,
		}
	}

	/// If this wraps a [`Syn::LitChar`], this returns the character within
	/// the delimiting quotation marks. Otherwise this returns `None`.
	#[must_use]
	pub fn char(&self) -> Option<char> {
		if self.0.kind() == Syn::LitChar {
			let text = self.0.text();
			let start = text.chars().position(|c| c == '\'').unwrap();
			let end = text.chars().rev().position(|c| c == '\'').unwrap();
			let inner = text.get((start + 1)..(text.len() - end - 1)).unwrap();
			unescape_char(inner).ok()
		} else {
			None
		}
	}

	#[must_use]
	pub fn float(&self) -> Option<f64> {
		if !matches!(self.0.kind(), Syn::LitFloat) {
			return None;
		}

		let text = self.0.text();

		// Identify the position of the suffix.
		let end = text.len() - text.chars().rev().position(|c| c != 'f').unwrap();
		let inner = &text[0..end];
		let mut temp = String::with_capacity(text.len());

		for c in inner.chars().filter(|c| *c != '_') {
			temp.push(c);
		}

		temp.parse::<f64>().ok()
	}

	/// Shorthand for `self.syntax().kind() == Syn::LitNull`.
	#[must_use]
	pub fn is_null(&self) -> bool {
		self.0.kind() == Syn::LitNull
	}

	/// Returns `None` if this is not tagged with [`Syn::LitInt`].
	/// Returns `Some(Err)` if integer parsing fails,
	/// such as if the written value is too large to fit into a `u64`.
	#[must_use]
	pub fn int(&self) -> Option<Result<u64, ParseIntError>> {
		if !matches!(self.0.kind(), Syn::LitInt) {
			return None;
		}

		let text = self.0.text();

		let radix = if text.len() > 2 {
			match &text[0..2] {
				"0x" => 16,
				"0b" => 2,
				"0o" => 8,
				_ => 10,
			}
		} else {
			10
		};

		// Identify the span between the prefix and suffix.
		let start = if radix != 10 { 2 } else { 0 };
		let end = text.len()
			- text
				.chars()
				.rev()
				.position(|c| !matches!(c, 'i' | 'u'))
				.unwrap();
		let inner = &text[start..end];
		let mut temp = String::with_capacity(inner.len());

		for c in inner.chars().filter(|c| *c != '_') {
			temp.push(c);
		}

		Some(u64::from_str_radix(&temp, radix))
	}

	/// If this wraps a [`Syn::LitString`] token, this returns the string's
	/// content with the delimiting quotation marks stripped away.
	/// Otherwise this returns `None`.
	#[must_use]
	pub fn string(&self) -> Option<&str> {
		if self.0.kind() == Syn::LitString {
			let text = self.0.text();
			let start = text.chars().position(|c| c == '"').unwrap();
			let end = text.chars().rev().position(|c| c == '"').unwrap();
			text.get((start + 1)..(text.len() - end - 1))
		} else {
			None
		}
	}

	#[must_use]
	pub fn syntax(&self) -> &SyntaxToken {
		&self.0
	}
}

/// Wraps a node tagged [`Syn::Name`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(SyntaxNode);

simple_astnode!(Syn, Name, Syn::Name);

impl Name {
	/// Shorthand for
	/// `self.syntax().first_child_or_token().unwrap().into_token().unwrap()`.
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		self.0.first_child_or_token().unwrap().into_token().unwrap()
	}
}

/// Wraps a [`Syn::Resolver`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Resolver(SyntaxNode);

simple_astnode!(Syn, Resolver, Syn::Resolver);

impl Resolver {
	/// Every token returns is tagged [`Syn::Ident`].
	pub fn parts(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0.children_with_tokens().filter_map(|n_or_t| {
			if n_or_t.kind() == Syn::Ident {
				n_or_t.into_token()
			} else {
				None
			}
		})
	}
}

/// Wraps a [`Syn::TypeRef`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct TypeRef(SyntaxNode);

simple_astnode!(Syn, TypeRef, Syn::TypeRef);
