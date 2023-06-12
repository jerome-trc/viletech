//! "Green caching" solutions for reducing memory during parsing.

use std::{
	borrow::Borrow,
	cell::RefCell,
	hash::{BuildHasherDefault, Hash, Hasher},
	rc::Rc,
};

use hashbrown::hash_map::RawEntryMut;
use rowan::{GreenNode, GreenNodeData, GreenToken, GreenTokenData, NodeOrToken, SyntaxKind};
use rustc_hash::FxHasher;

use crate::GreenElement;

/*

(ROWAN)

XXX: the impl is a bit tricky. As usual when writing interners, we want to
store all values in one HashSet.

However, hashing trees is fun: hash of the tree is recursively defined. We
maintain an invariant -- if the tree is interned, then all of its children
are interned as well.

That means that computing the hash naively is wasteful -- we just *know*
hashes of children, and we can re-use those.

So here we use *raw* API of hashbrown and provide the hashes manually,
instead of going via a `Hash` impl. Our manual `Hash` and the
`#[derive(Hash)]` are actually different! At some point we had a fun bug,
where we accidentally mixed the two hashes, which made the cache much less
efficient.

(RAT)

Rowan itself used a `NoHash` newtype wrapper to prevent the aforementioned bug,
but DashMap type constraints forbid this. Not sure how much I can do about this.

There's also an unfortunate amount of code duplication here.
Maybe Rowan just has to get vendored entirely, or forked.

*/

pub trait GreenCache: 'static + Default + Clone {
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

#[derive(Debug, Default, Clone)]
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

/// Single-threaded green element cache, backed by [`hashbrown::HashSet`]s.
/// Wraps an [`Rc`] and is thus trivial to clone.
#[derive(Debug, Default, Clone)]
pub struct GreenCacheSt(Rc<RefCell<(HashSet<GreenNode>, HashSet<GreenToken>)>>);

type HashSet<T> = hashbrown::HashMap<T, (), BuildHasherDefault<FxHasher>>;

impl GreenCache for GreenCacheSt {
	fn token(&mut self, kind: SyntaxKind, text: &str) -> (u64, GreenToken) {
		let hash = {
			let mut h = FxHasher::default();
			kind.hash(&mut h);
			text.hash(&mut h);
			h.finish()
		};

		let mut inner = RefCell::borrow_mut(&self.0);
		let tokens = &mut inner.1;

		let entry = tokens
			.raw_entry_mut()
			.from_hash(hash, |token| token.kind() == kind && token.text() == text);

		let token = match entry {
			RawEntryMut::Occupied(entry) => entry.key().clone(),
			RawEntryMut::Vacant(entry) => {
				let token = GreenToken::new(kind, text);
				entry.insert_with_hasher(hash, token.clone(), (), |t| token_hash(t));
				token
			}
		};

		(hash, token)
	}

	fn node(
		&mut self,
		kind: SyntaxKind,
		children: &mut Vec<(u64, GreenElement)>,
		first_child: usize,
	) -> (u64, GreenNode) {
		let mut inner = RefCell::borrow_mut(&self.0);
		let nodes = &mut inner.0;

		let build_node = move |children: &mut Vec<(u64, GreenElement)>| {
			GreenNode::new(kind, children.drain(first_child..).map(|(_, it)| it))
		};

		let children_ref = &children[first_child..];

		if children_ref.len() > 3 {
			let node = build_node(children);
			return (0, node);
		}

		let hash = {
			let mut h = FxHasher::default();

			kind.hash(&mut h);

			for &(hash, _) in children_ref {
				if hash == 0 {
					let node = build_node(children);
					return (0, node);
				}

				hash.hash(&mut h);
			}

			h.finish()
		};

		// Green nodes are fully immutable, so it's ok to deduplicate them.
		// This is the same optimization that Roslyn does
		// https://github.com/KirillOsenkov/Bliki/wiki/Roslyn-Immutable-Trees
		//
		// For example, all `#[inline]` in this file share the same green node!
		// For `libsyntax/parse/parser.rs`, measurements show that deduping saves
		// 17% of the memory for green nodes!
		let entry = nodes.raw_entry_mut().from_hash(hash, |node| {
			node.kind() == kind && node.children().len() == children_ref.len() && {
				let lhs = node.children();
				let rhs = children_ref.iter().map(|(_, it)| match it {
					NodeOrToken::Node(node) => NodeOrToken::Node(node.borrow()),
					NodeOrToken::Token(token) => NodeOrToken::Token(token.borrow()),
				});

				let lhs = lhs.map(element_id);
				let rhs = rhs.map(element_id);

				lhs.eq(rhs)
			}
		});

		let node = match entry {
			RawEntryMut::Occupied(entry) => {
				drop(children.drain(first_child..));
				entry.key().clone()
			}
			RawEntryMut::Vacant(entry) => {
				let node = build_node(children);
				entry.insert_with_hasher(hash, node.clone(), (), |n| node_hash(n.borrow()));
				node
			}
		};

		(hash, node)
	}
}

/// Multi-threaded green element cache, backed by [`dashmap::DashSet`]s.
/// Wraps an [`Arc`] and is thus trivial to clone.
///
/// [`Arc`]: std::sync::Arc
#[derive(Debug, Default, Clone)]
#[cfg(feature = "parallel")]
pub struct GreenCacheMt(std::sync::Arc<(DashSet<GreenNode>, DashSet<GreenToken>)>);

#[cfg(feature = "parallel")]
type DashSet<T> = dashmap::DashSet<T, BuildHasherDefault<FxHasher>>;

#[cfg(feature = "parallel")]
impl GreenCache for GreenCacheMt {
	fn token(&mut self, kind: SyntaxKind, text: &str) -> (u64, GreenToken) {
		use dashmap::SharedValue;

		let inp = TokenHashInput { kind, text };

		let hash = {
			let mut h = FxHasher::default();
			inp.hash(&mut h);
			h.finish()
		};

		let shash = self.0 .1.hash_usize(&inp);
		let shardndx = self.0 .1.determine_shard(shash);
		let shards = self.0 .1.shards();
		let shard = &shards[shardndx];
		let mut map = shard.write();
		let entry = map
			.raw_entry_mut()
			.from_hash(hash, |token| token.kind() == kind && token.text() == text);

		let token = match entry {
			RawEntryMut::Occupied(entry) => entry.key().clone(),
			RawEntryMut::Vacant(entry) => {
				let token = GreenToken::new(kind, text);
				entry.insert_with_hasher(hash, token.clone(), SharedValue::new(()), |t| {
					token_hash(t.borrow())
				});
				token
			}
		};

		(hash, token)
	}

	fn node(
		&mut self,
		kind: SyntaxKind,
		children: &mut Vec<(u64, GreenElement)>,
		first_child: usize,
	) -> (u64, GreenNode) {
		use dashmap::SharedValue;

		let build_node = move |children: &mut Vec<(u64, GreenElement)>| {
			GreenNode::new(kind, children.drain(first_child..).map(|(_, it)| it))
		};

		let children_ref = &children[first_child..];

		if children_ref.len() > 3 {
			let node = build_node(children);
			return (0, node);
		}

		let hash = {
			let mut h = FxHasher::default();

			kind.hash(&mut h);

			for &(hash, _) in children_ref {
				if hash == 0 {
					let node = build_node(children);
					return (0, node);
				}

				hash.hash(&mut h);
			}

			h.finish()
		};

		let inp = NodeHashInput {
			kind,
			children: children_ref,
		};

		let shash = self.0 .0.hash_usize(&inp);
		let shardndx = self.0 .0.determine_shard(shash);
		let shards = self.0 .0.shards();
		let shard = &shards[shardndx];
		let mut map = shard.write();

		// (ROWAN)
		// Green nodes are fully immutable, so it's ok to deduplicate them.
		// This is the same optimization that Roslyn does
		// https://github.com/KirillOsenkov/Bliki/wiki/Roslyn-Immutable-Trees
		//
		// For example, all `#[inline]` in this file share the same green node!
		// For `libsyntax/parse/parser.rs`, measurements show that deduping saves
		// 17% of the memory for green nodes!
		let entry = map.raw_entry_mut().from_hash(hash, |node| {
			node.kind() == kind && node.children().len() == children_ref.len() && {
				let lhs = node.children();
				let rhs = children_ref.iter().map(|(_, it)| match it {
					NodeOrToken::Node(node) => NodeOrToken::Node(node.borrow()),
					NodeOrToken::Token(token) => NodeOrToken::Token(token.borrow()),
				});

				let lhs = lhs.map(element_id);
				let rhs = rhs.map(element_id);

				lhs.eq(rhs)
			}
		});

		let node = match entry {
			RawEntryMut::Occupied(entry) => {
				drop(children.drain(first_child..));
				entry.key().clone()
			}
			RawEntryMut::Vacant(entry) => {
				let node = build_node(children);
				entry.insert_with_hasher(hash, node.clone(), SharedValue::new(()), |n| {
					node_hash(n.borrow())
				});
				node
			}
		};

		(hash, node)
	}
}

fn token_hash(token: &GreenTokenData) -> u64 {
	let mut h = FxHasher::default();
	token.kind().hash(&mut h);
	token.text().hash(&mut h);
	h.finish()
}

fn node_hash(node: &GreenNodeData) -> u64 {
	let mut h = FxHasher::default();

	node.kind().hash(&mut h);

	for child in node.children() {
		match child {
			NodeOrToken::Node(it) => node_hash(it),
			NodeOrToken::Token(it) => token_hash(it),
		}
		.hash(&mut h)
	}

	h.finish()
}

fn element_id(elem: GreenElementRef<'_>) -> *const () {
	match elem {
		NodeOrToken::Node(it) => it as *const GreenNodeData as *const (),
		NodeOrToken::Token(it) => it as *const GreenTokenData as *const (),
	}
}

type GreenElementRef<'a> = NodeOrToken<&'a GreenNodeData, &'a GreenTokenData>;

/// Used to enforce specific hashing behavior when using `DashSet::hash_usize`.
struct TokenHashInput<'i> {
	kind: SyntaxKind,
	text: &'i str,
}

impl Hash for TokenHashInput<'_> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.kind.hash(state);
		self.text.hash(state);
	}
}

/// Used to enforce specific hashing behavior when using `DashSet::hash_usize`.
struct NodeHashInput<'c> {
	kind: SyntaxKind,
	children: &'c [(u64, NodeOrToken<GreenNode, GreenToken>)],
}

impl Hash for NodeHashInput<'_> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.kind.hash(state);

		for &(hash, _) in self.children {
			if hash == 0 {
				continue;
			}

			hash.hash(state);
		}
	}
}
