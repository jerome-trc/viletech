//! Common utilities for auto-formatting code.

use rowan::{NodeOrToken, SyntaxElement, SyntaxNode, SyntaxText};

use crate::LangExt;

#[derive(Debug)]
pub struct AutoFormatter<CFG, CTX> {
	pub cfg: CFG,
	pub ctx: CTX,
	/// Identation depth in abstract terms, rather than in spaces, tabs, et cetera.
	pub depth: u32,
}

impl<CFG, CTX> AutoFormatter<CFG, CTX> {
	#[must_use]
	pub fn new(cfg: CFG, ctx: CTX) -> Self {
		Self { cfg, ctx, depth: 0 }
	}
}

#[derive(Debug)]
pub struct FormatConfig {
	pub tabs: TabStyle,
	pub line_ends: LineEnds,
	/// In terms of characters.
	pub max_line_len: usize,
}

impl FormatConfig {
	#[must_use]
	pub fn overlong<L: LangExt>(&self, node: &SyntaxNode<L>) -> bool {
		let mut char_len = 0;
		node.text()
			.for_each_chunk(|s| char_len += s.chars().count());
		char_len > self.max_line_len
	}

	#[must_use]
	pub fn overlong_iter<L: LangExt>(&self, elems: impl Iterator<Item = SyntaxElement<L>>) -> bool {
		let char_len = elems.fold(0, |mut i, e| {
			match e {
				NodeOrToken::Node(n) => i += char_count(&n.text()),
				NodeOrToken::Token(t) => i += t.text().chars().count(),
			}

			i
		});

		char_len > self.max_line_len
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnds {
	Cr,
	CrLf,
	Lf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BraceStyle {
	/// e.g.:
	///
	/// ```
	/// class MyClass {
	/// }
	/// ```
	SameLine,
	/// e.g.:
	///
	/// ```
	/// class MyClass
	/// {
	/// }
	/// ```
	NewLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabStyle {
	/// Use space characters for indentation and alignment.
	Spaces { width: u32 },
	/// Use tab characters for indentation and spaces for alignment.
	Tabs,
}

#[must_use]
pub fn char_count(text: &SyntaxText) -> usize {
	let mut ret = 0;
	text.for_each_chunk(|s| ret += s.chars().count());
	ret
}
