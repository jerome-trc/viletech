//! Abstract syntax tree nodes.

use doomfront::rowan::{self, ast::AstNode, SyntaxNode};

use super::Syn;

/// One of the top-level elements of a file.
///
/// Mind that the overwhelming majority of REPL inputs will have none of these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Root {
	// ???
}

impl AstNode for Root {
	type Language = Syn;

	fn can_cast(_kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		todo!()
	}

	fn cast(_node: SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		todo!()
	}

	fn syntax(&self) -> &SyntaxNode<Self::Language> {
		todo!()
	}
}
