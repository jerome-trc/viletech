//! A drop-in replacement for [`rowan::GreenNodeBuilder`] with more capabilities.

use rowan::{GreenNode, GreenToken, SyntaxKind};

use super::GreenElement;

/// A drop-in replacement for [`rowan::GreenNodeBuilder`] with more capabilities.
#[derive(Debug, Default)]
pub struct GreenBuilder<C: GreenCache> {
	cache: C,
	parents: Vec<(SyntaxKind, usize)>,
	children: Vec<(u64, GreenElement)>,
}

impl<C: GreenCache + Default> GreenBuilder<C> {
	#[must_use]
	pub fn new(cache: Option<C>) -> Self {
		Self {
			cache: cache.unwrap_or_default(),
			..Default::default()
		}
	}
}

impl<C: GreenCache> GreenBuilder<C> {
	/// Adds a new token to the current branch.
	#[inline]
	pub fn token(&mut self, kind: SyntaxKind, text: &str) {
		let (hash, token) = self.cache.token(kind, text);
		self.children.push((hash, token.into()));
	}

	/// Start a new node and make it current.
	///
	/// This is a cheap operation, involving only a vector push of two integers.
	#[inline]
	pub fn open(&mut self, kind: SyntaxKind) {
		self.parents.push((kind, self.children.len()));
	}

	/// Finish the current branch and restore the previous branch as current.
	#[inline]
	pub fn close(&mut self) {
		let (kind, first_child) = self.parents.pop().expect("Tried to close an absent node.");
		let (hash, node) = self.cache.node(kind, &mut self.children, first_child);
		self.children.push((hash, node.into()))
	}

	/// Drops the current branch if it matches `kind`,
	/// restoring the previous branch as current.
	#[inline]
	pub fn cancel(&mut self, kind: SyntaxKind) {
		if self.parents.last().unwrap().0 == kind {
			let (_, first_child) = self.parents.pop().unwrap();

			self.children.truncate(first_child);
		}
	}

	#[inline]
	pub fn cancel_if(&mut self, predicate: fn(SyntaxKind) -> bool) {
		if predicate(self.parents.last().unwrap().0) {
			let (_, first_child) = self.parents.pop().unwrap();

			self.children.truncate(first_child);
		}
	}

	/// Prepare for maybe wrapping the next node.
	///
	/// The way wrapping works is that you first of all get a checkpoint,
	/// then you place all tokens you want to wrap, and then *maybe* call
	/// [`Self::open_at`].
	#[inline]
	pub fn checkpoint(&self) -> Checkpoint {
		Checkpoint(self.children.len())
	}

	/// Wrap the previous branch marked by `checkpoint` in a new branch and
	/// make it current.
	#[inline]
	pub fn open_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
		let Checkpoint(checkpoint) = checkpoint;

		assert!(
			checkpoint <= self.children.len(),
			"Checkpoint no longer valid, was `close` called early?"
		);

		if let Some(&(_, first_child)) = self.parents.last() {
			assert!(
				checkpoint >= first_child,
				"Checkpoint no longer valid, was an unmatched `open_at` called?"
			);
		}

		self.parents.push((kind, checkpoint));
	}

	/// Drops all children added since `checkpoint`.
	#[inline]
	pub fn cancel_checkpoint(&mut self, checkpoint: Checkpoint) {
		self.children.truncate(checkpoint.0);
	}

	/// Complete tree building.
	/// Ensure that [`Self::open`] and [`Self::close`] calls are paired!
	#[inline]
	#[must_use]
	pub fn finish(mut self) -> GreenNode {
		assert_eq!(
			self.children.len(),
			1,
			"An opened branch was never closed ({:?}).",
			self.children[1].1.kind()
		);

		match self.children.pop().unwrap().1 {
			GreenElement::Node(node) => node,
			GreenElement::Token(_) => panic!("A green token can not be the root of a green tree."),
		}
	}

	#[must_use]
	pub fn parent_count(&self) -> usize {
		self.parents.len()
	}
}

/// A checkpoint for maybe wrapping a node. See [`GreenBuilder::checkpoint`].
#[derive(Clone, Copy, Debug)]
pub struct Checkpoint(usize);

// GreenCache //////////////////////////////////////////////////////////////////

pub trait GreenCache: 'static + Default {
	#[must_use]
	fn token(&mut self, kind: SyntaxKind, text: &str) -> (u64, GreenToken);

	#[must_use]
	fn node(
		&mut self,
		kind: SyntaxKind,
		children: &mut Vec<(u64, GreenElement)>,
		first_child: usize,
	) -> (u64, GreenNode);
}

#[derive(Debug, Default)]
pub struct GreenCacheNoop;

impl GreenCache for GreenCacheNoop {
	fn token(&mut self, kind: SyntaxKind, text: &str) -> (u64, GreenToken) {
		(0, GreenToken::new(kind, text))
	}

	fn node(
		&mut self,
		kind: SyntaxKind,
		children: &mut Vec<(u64, GreenElement)>,
		first_child: usize,
	) -> (u64, GreenNode) {
		(
			0,
			GreenNode::new(kind, children.drain(first_child..).map(|(_, elem)| elem)),
		)
	}
}

// TODO:
// - Single-threaded cache behind `Rc<RefCell>`.
// - Multi-threaded cache using DashMap and SegQueue behind `Arc`.
