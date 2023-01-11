//! The highest-level frontend representation.

use indexmap::IndexMap;
use rowan::{ast::AstNode, SyntaxNode};

use super::syn::SyntaxKind;

/// One component of a [parse tree];
/// an array of dynamically-typed thin wrappers over [syntax nodes].
///
/// [parse tree]: super::parse::ParseTree
/// [syntax nodes]: rowan::SyntaxNode
pub struct Tree {
	nodes: IndexMap<SyntaxNode<SyntaxKind>, Node>,
}

pub type Node = Box<dyn AstNode<Language = SyntaxKind>>;

impl Tree {
	#[must_use]
	pub(super) fn new(root: SyntaxNode<SyntaxKind>) -> Self {
		let guess = u32::from(root.text().len()) as usize;

		let ret = Self {
			// TODO: Establish a heuristic for this
			nodes: IndexMap::with_capacity(guess / 3),
		};

		for _ in root.preorder() {
			// ???
		}

		ret
	}

	pub fn all_nodes(&self) -> impl Iterator<Item = &Node> {
		self.nodes.values()
	}
}

/// A preprocessor directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preproc(SyntaxNode<SyntaxKind>);

impl AstNode for Preproc {
	type Language = SyntaxKind;

	fn can_cast(kind: SyntaxKind) -> bool
	where
		Self: Sized,
	{
		kind == SyntaxKind::Preproc
	}

	fn cast(node: SyntaxNode<SyntaxKind>) -> Option<Self>
	where
		Self: Sized,
	{
		if node.kind() == SyntaxKind::Preproc {
			return Some(Self(node));
		}

		None
	}

	fn syntax(&self) -> &SyntaxNode<SyntaxKind> {
		&self.0
	}
}
