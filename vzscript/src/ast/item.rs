//! AST nodes for representing function declarations, symbolic constants, et cetera.

use doomfront::{
	rowan::{self, ast::AstNode},
	simple_astnode,
};

use crate::{Syn, SyntaxNode};

use super::Ident;

/// Wraps a node tagged [`Syn::FuncDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ser_de", derive(serde::Serialize))]
pub struct FuncDecl(pub(super) SyntaxNode);

simple_astnode!(Syn, FuncDecl, Syn::FuncDecl);

impl FuncDecl {
	#[must_use]
	pub fn name(&self) -> Ident {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::Ident)
					.map(Ident)
			})
			.unwrap()
	}
}
