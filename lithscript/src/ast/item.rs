//! AST nodes for representing function declarations, symbolic constants, et cetera.

use doomfront::{simple_astnode, AstError, AstResult};

use crate::{Syn, SyntaxNode, SyntaxToken};

/// Wraps a node tagged [`Syn::FuncDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FuncDecl(pub(super) SyntaxNode);

simple_astnode!(Syn, FuncDecl, Syn::FuncDecl);

impl FuncDecl {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}
}
