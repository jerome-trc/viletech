//! AST nodes for representing literals.

use crate::{simple_astnode, zdoom::ast::LitToken};

use super::{Syntax, SyntaxNode};

/// Wraps a node tagged [`Syntax::Literal`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Literal(pub(super) SyntaxNode);

simple_astnode!(Syntax, Literal, Syntax::Literal);

impl Literal {
	#[must_use]
	pub fn token(&self) -> LitToken<Syntax> {
		LitToken::new(self.0.first_token().unwrap())
	}
}
