//! AST nodes for representing literals.

use crate::{simple_astnode, zdoom::ast::LitToken};

use super::{Syn, SyntaxNode};

/// Wraps a node tagged [`Syn::Literal`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Literal(pub(super) SyntaxNode);

simple_astnode!(Syn, Literal, Syn::Literal);

impl Literal {
	#[must_use]
	pub fn token(&self) -> LitToken<Syn> {
		LitToken::new(self.0.first_token().unwrap())
	}
}
