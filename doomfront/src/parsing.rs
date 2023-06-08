//! Utilities for making it easier to write succinct [`peg`] parsers.

use rowan::{GreenNode, GreenToken, SyntaxKind};
use smallvec::SmallVec;

use crate::GreenElement;

/// A helper for building a single [`GreenNode`], wrapping a [`SmallVec`].
///
/// Use [`Self::start`] to add the element that signifies the start of what appears
/// to be a syntax component, and then submit optional following elements expected
/// to be present. If an expected element turns out to be a `None`, the builder
/// remembers this and marks itself as having failed to emit an error node containing
/// those tokens instead.
#[derive(Debug)]
pub struct GreenBuilder<const CAP: usize> {
	buf: SmallVec<[GreenElement; CAP]>,
	failed: bool,
	syn: SyntaxKind,
	err_syn: SyntaxKind,
}

impl<const CAP: usize> GreenBuilder<CAP> {
	#[must_use]
	pub fn new(syn: impl Into<SyntaxKind>, err_syn: impl Into<SyntaxKind>) -> Self {
		Self {
			buf: SmallVec::new(),
			failed: false,
			syn: syn.into(),
			err_syn: err_syn.into(),
		}
	}

	pub fn start(&mut self, elem: impl Into<GreenElement>) {
		self.buf.push(elem.into());
	}

	pub fn start_s(&mut self, syn: impl Into<SyntaxKind>, string: &str) {
		self.buf.push(GreenToken::new(syn.into(), string).into());
	}

	pub fn just(&mut self, elem: Option<impl Into<GreenElement>>) {
		if let Some(e) = elem {
			self.buf.push(e.into());
		} else {
			self.failed = true;
		}
	}

	pub fn just_s(&mut self, syn: impl Into<SyntaxKind>, string: Option<&str>) {
		if let Some(s) = string {
			self.buf.push(GreenToken::new(syn.into(), s).into());
		} else {
			self.failed = true;
		}
	}

	pub fn maybe(&mut self, elem: Option<impl Into<GreenElement>>) {
		if let Some(e) = elem {
			self.buf.push(e.into());
		}
	}

	pub fn maybe_s(&mut self, syn: impl Into<SyntaxKind>, string: Option<&str>) {
		if let Some(s) = string {
			self.buf.push(GreenToken::new(syn.into(), s).into());
		}
	}

	pub fn append(&mut self, elems: Vec<impl Into<GreenElement>>) {
		for elem in elems {
			self.buf.push(elem.into());
		}
	}

	/// Like [`Self::append`], but expects `elem` to have a length of at least 1.
	/// Otherwise, the node is considered to have failed.
	pub fn append_min1(&mut self, elems: Vec<impl Into<GreenElement>>) {
		if elems.is_empty() {
			self.failed = true;
		} else {
			for elem in elems {
				self.buf.push(elem.into());
			}
		}
	}

	/// Marks the node as having failed, and does nothing else.
	pub fn fail(&mut self) {
		self.failed = true;
	}

	#[must_use]
	pub fn finish(self) -> GreenNode {
		if !self.failed {
			GreenNode::new(self.syn, self.buf)
		} else {
			GreenNode::new(self.err_syn, self.buf)
		}
	}
}

pub type Gtb4 = GreenBuilder<4>;
pub type Gtb8 = GreenBuilder<8>;
pub type Gtb16 = GreenBuilder<16>;
