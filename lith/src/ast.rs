//! Structures for representing Lithica abstract syntax trees.

mod expr;
mod lit;

use crate::{Syn, SyntaxToken};

pub use self::{expr::*, lit::*};

/// A convenience for positions which accept either [`Syn::Ident`] or [`Syn::NameLit`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Name(pub(super) SyntaxToken);

impl Name {
	#[must_use]
	pub fn text(&self) -> &str {
		match self.0.kind() {
			Syn::Ident => self.0.text(),
			Syn::LitName => &self.0.text()[1..(self.0.text().len() - 1)],
			_ => unreachable!(),
		}
	}

	#[must_use]
	pub fn inner(&self) -> &SyntaxToken {
		&self.0
	}
}
