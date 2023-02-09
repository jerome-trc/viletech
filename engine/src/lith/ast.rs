//! Abstract syntax tree nodes.

mod expr;

use bitflags::bitflags;
use doomfront::{
	rowan::{self, ast::AstNode, SyntaxNode, SyntaxToken},
	simple_astnode,
};

use super::Syn;

use expr::*;

/// One of the top-level elements of a file or REPL input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Root {
	Item(Item),
}

impl AstNode for Root {
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

	fn cast(node: SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::FunctionDecl => Some(Self::Item(Item::FunctionDecl(FunctionDecl(node)))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode<Self::Language> {
		match self {
			Self::Item(inner) => inner.syntax(),
		}
	}
}

/// Wraps a [`Syn::Block`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Block(SyntaxNode<Syn>);

impl AstNode for Block {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		kind == Syn::Block
	}

	fn cast(node: SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		if node.kind() == Syn::Block {
			Some(Self(node))
		} else {
			None
		}
	}

	fn syntax(&self) -> &SyntaxNode<Self::Language> {
		&self.0
	}
}

/// Wraps a [`Syn::DeclQualifiers`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct DeclQualifiers(SyntaxNode<Syn>);

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
				_ => {} // Whitespace or comment
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

/// Wraps a [`Syn::FunctionDecl`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct FunctionDecl(SyntaxNode<Syn>);

impl FunctionDecl {
	/// The identifier given to this function, after the return type specifier.
	#[must_use]
	pub fn name(&self) -> SyntaxToken<Syn> {
		self.0
			.children_with_tokens()
			.find_map(|n_or_t| {
				if n_or_t.kind() == Syn::Identifier {
					n_or_t.into_token()
				} else {
					None
				}
			})
			.unwrap()
	}

	/// Returns `None` if no qualifiers were written.
	#[must_use]
	pub fn qualifiers(&self) -> Option<DeclQualifiers> {
		self.0.children().find_map(DeclQualifiers::cast)
	}

	pub fn return_types(&self) -> impl Iterator<Item = ExprType> {
		let rets = self
			.0
			.children()
			.find(|node| node.kind() == Syn::ReturnTypes)
			.unwrap();

		rets.children().filter_map(ExprType::cast)
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

	fn cast(node: SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::FunctionDecl => Some(Self::FunctionDecl(FunctionDecl(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode<Self::Language> {
		match self {
			Self::FunctionDecl(inner) => inner.syntax(),
		}
	}
}
