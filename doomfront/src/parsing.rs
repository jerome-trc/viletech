//! Utilities for making it easier to write succinct [`peg`] parsers.

use rowan::{GreenNode, SyntaxKind};
use smallvec::SmallVec;

use crate::GreenElement;

/// A helper for building a single [`GreenNode`], wrapping a [`SmallVec`].
#[derive(Debug)]
pub struct GreenBuilder<const CAP: usize> {
	buf: SmallVec<[GreenElement; CAP]>,
	syn: SyntaxKind,
}

impl<const CAP: usize> GreenBuilder<CAP> {
	#[must_use]
	pub fn new(syn: impl Into<SyntaxKind>) -> Self {
		Self {
			buf: SmallVec::new(),
			syn: syn.into(),
		}
	}

	pub fn push(&mut self, elem: impl Into<GreenElement>) {
		self.buf.push(elem.into());
	}

	pub fn append(&mut self, elems: Vec<impl Into<GreenElement>>) {
		for elem in elems {
			self.buf.push(elem.into());
		}
	}

	pub fn maybe(&mut self, elem: Option<impl Into<GreenElement>>) {
		if let Some(elem) = elem {
			self.buf.push(elem.into());
		}
	}

	pub fn append_many(&mut self, meta_vec: Vec<Vec<impl Into<GreenElement>>>) {
		for sub_vec in meta_vec {
			self.append(sub_vec);
		}
	}

	#[must_use]
	pub fn finish(self) -> GreenNode {
		GreenNode::new(self.syn, self.buf)
	}
}

/// A [green tree builder] able to hold 4 elements before spilling to the heap.
///
/// [green tree builder]: GreenBuilder
pub type Gtb4 = GreenBuilder<4>;

/// A [green tree builder] able to hold 8 elements before spilling to the heap.
///
/// [green tree builder]: GreenBuilder
pub type Gtb8 = GreenBuilder<8>;

/// A [green tree builder] able to hold 12 elements before spilling to the heap.
///
/// [green tree builder]: GreenBuilder
pub type Gtb12 = GreenBuilder<12>;

/// A [green tree builder] able to hold 16 elements before spilling to the heap.
///
/// [green tree builder]: GreenBuilder
pub type Gtb16 = GreenBuilder<16>;
