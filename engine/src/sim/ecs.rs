//! The entity-component-system parts of the playsim code.
//!
//! Also see [`crate::ecs`] for component definitions.
//!
//! The code here makes a lot of guesswork about how to achieve performance.
//! It is all subject to change as real-world test results come in.

use std::collections::VecDeque;

use crate::{
	ecs,
	sparse::{SparseSet, SparseSetIndex},
};

/// Holds all state of the entity-component-system part of VileTech's playsim.
#[derive(Debug)]
pub struct World {
	extant: SparseSet<ActorId, ActorId>,
	/// Non-extant IDs are kept here. Some extra gets allocated when a playsim
	/// reset occurs; despawned IDs will get sent to the back.
	///
	/// If attempting to spawn an actor with an empty ID queue, `head` gets
	/// advanced and the result is used to fulfill the spawn request.
	absent: VecDeque<ActorId>,
	/// Very deliberately kept public.
	pub comps: Components,
	head: ActorId,
}

impl World {
	/// `capacity_hint` is meant to inform what size of allocation should be
	/// pre-reserved. This should come from the number of "things" that were
	/// pre-placed in a level (monsters, inventory items, decorations).
	///
	/// Extra space beyond `capacity_hint` is always allocated to account for
	/// projectiles, hitscan puffs, teleport effects, Lost Souls, and whatever
	/// other entities a mod may spawn dynamically.
	///
	/// This extra headroom diminishes as `capacity_hint` increases; it is
	/// assumed that DOOM E1M1 requires more headroom relative to its initial
	/// thing count than the larger slaughtermaps which don't frontload all of
	/// their monsters at once.
	///
	/// This will panic if `capacity_hint` is 0.
	#[must_use]
	pub fn new(capacity_hint: usize) -> Self {
		assert_ne!(
			capacity_hint, 0,
			"Illegally tried to create an ECS world with 0 capacity."
		);

		let mut ret = Self {
			extant: SparseSet::default(),
			absent: VecDeque::default(),
			comps: Components::default(),
			head: ActorId { shard: 0, index: 0 },
		};

		ret.reset(capacity_hint);

		ret
	}

	pub fn spawn(&mut self) -> ActorId {
		if let Some(new_id) = self.absent.pop_front() {
			self.extant.insert(new_id, new_id);
			new_id
		} else {
			let new_id = self.advance_head();
			self.extant.insert(new_id, new_id);
			// Q: Is it worthwhile to advance the head further, pushing more
			// headroom onto the back of `absent`? Is it likely to be in the
			// cache at this point?
			new_id
		}
	}

	pub fn despawn(&mut self, id: ActorId) {
		let despawned = self.extant.remove(id).unwrap();
		self.absent.push_back(despawned);
		self.comps.despawn(id);
	}

	/// [`Self::new`] just constructs defaults and then forwards its
	/// `capacity_hint` to this function. See its docs for details.
	pub fn reset(&mut self, capacity_hint: usize) {
		let extra = match capacity_hint {
			0 => 0,
			1..=69 => {
				// Thing count for DOOM2 MAP01 is 69
				((capacity_hint as f32) * 0.5) as usize
			}
			70..=521 => {
				// Thing count for Plutonia MAP32 is 521
				((capacity_hint as f32) * 0.33) as usize
			}
			522..=1817 => {
				// Thing count for Scythe 2 MAP30 is 1817
				((capacity_hint as f32) * 0.2) as usize
			}
			1818..=3232 => {
				// Thing count for Alien Vendetta MAP25 is 3232
				((capacity_hint as f32) * 0.1) as usize
			}
			3233..=5537 => {
				// Thing count for Slaughterfest 2012 MAP25 is 5537
				((capacity_hint as f32) * 0.05) as usize
			}
			_ => {
				// Thing count for Cosmogenesis MAP05 is 81036 (!!!)
				((capacity_hint as f32) * 0.01) as usize
			}
		};

		let cap = capacity_hint + extra;
		self.extant = SparseSet::with_capacity(cap, cap);
		self.absent.clear();

		for _ in 0..extra {
			let id = self.advance_head();
			self.absent.push_back(id);
		}

		self.comps.clear();
		// Rebuild component arrays lazily
	}

	#[must_use]
	pub fn is_extant(&self, id: ActorId) -> bool {
		self.extant.contains(id)
	}

	#[must_use]
	fn advance_head(&mut self) -> ActorId {
		let ret = self.head;

		self.head.index += 1;

		if self.head.index == SHARD_LEN {
			self.head.index = 0;
			self.head.shard += 1;
		}

		ret
	}
}

impl Default for World {
	/// Shorthand for `Self::new(521)`.
	fn default() -> Self {
		Self::new(521)
	}
}

#[derive(Debug, Default)]
pub struct Components {
	pub core: ShardedSparseSet<ecs::Core>,
	pub spatial: ShardedSparseSet<ecs::Spatial>,
	pub monster: ShardedSparseSet<ecs::Monster>,
}

impl Components {
	fn clear(&mut self) {
		self.core.clear();
		self.spatial.clear();
		self.monster.clear();
	}

	fn despawn(&mut self, id: ActorId) {
		if self.core.contains(id) {
			self.core.remove(id);
		}

		if self.spatial.contains(id) {
			self.spatial.remove(id);
		}

		if self.monster.contains(id) {
			self.monster.remove(id);
		}
	}

	pub fn iter_all(&self) -> impl Iterator<Item = ArchAll> {
		itertools::izip![self.core.iter(), self.spatial.iter(), self.monster.iter(),]
	}

	pub fn iter_mut_all(&mut self) -> impl Iterator<Item = ArchAllMut> {
		itertools::izip![
			self.core.iter_mut(),
			self.spatial.iter_mut(),
			self.monster.iter_mut(),
		]
	}
}

pub type ArchAll<'c> = (&'c ecs::Core, &'c ecs::Spatial, &'c ecs::Monster);
pub type ArchAllMut<'c> = (
	&'c mut ecs::Core,
	&'c mut ecs::Spatial,
	&'c mut ecs::Monster,
);

/// Each shard is just a [`SparseSet`], pre-allocated with [`SHARD_LEN`]
/// capacity, which forbids insertions past that limit.
/// ***In theory***, this has the following advantages:
/// - Lith actor objects can hold raw pointers directly to their components,
/// safe in the knowledge that the memory behind them is stable.
/// - If the vec itself needs to re-allocate, only pointers are mem-copied.
/// - We can experiment with speculatively allocating new shards on another thread.
#[derive(Debug)]
#[repr(transparent)]
pub struct ShardedSparseSet<C: ecs::Component> {
	shards: Vec<Shard<C>>,
}

impl<C: ecs::Component> ShardedSparseSet<C> {
	pub fn add(&mut self, id: ActorId, component: C) {
		if self.shards.len() < id.shard {
			self.shards.insert(id.shard, Shard::new());
		}

		self.shards[id.shard].add(id.index, component);
	}

	pub fn remove(&mut self, id: ActorId) -> C {
		self.shards[id.shard].remove(id.index)
	}

	/// In other words, check if actor `id` has component `C`.
	#[must_use]
	pub fn contains(&self, id: ActorId) -> bool {
		id.shard < self.shards.len() && self.shards[id.shard].contains(id.index)
	}

	pub fn clear(&mut self) {
		self.shards.clear();
	}

	pub fn iter(&self) -> impl Iterator<Item = &C> {
		self.shards.iter().flatten()
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut C> {
		self.shards.iter_mut().flatten()
	}
}

impl<C: ecs::Component> Default for ShardedSparseSet<C> {
	fn default() -> Self {
		Self {
			shards: Vec::default(),
		}
	}
}

#[derive(Debug)]
#[repr(transparent)]
pub(super) struct Shard<C: ecs::Component>(SparseSet<usize, C>);

impl<C: ecs::Component> Shard<C> {
	#[must_use]
	fn new() -> Self {
		Self(SparseSet::with_capacity(SHARD_LEN, SHARD_LEN))
	}

	fn add(&mut self, index: usize, component: C) {
		assert!(
			!self.is_full(),
			"Tried to overfill a component sparse shard."
		);
		self.0.insert(index, component);
	}

	fn remove(&mut self, index: usize) -> C {
		self.0.remove(index).unwrap()
	}

	#[must_use]
	fn contains(&self, index: usize) -> bool {
		self.0.contains(index)
	}

	#[must_use]
	fn is_full(&self) -> bool {
		self.0.dense_len() == self.0.dense_capacity()
	}

	#[must_use]
	fn as_slice(&self) -> &[C] {
		self.0.as_dense_slice()
	}

	#[must_use]
	unsafe fn as_mut_slice(&mut self) -> &mut [C] {
		self.0.as_dense_mut_slice()
	}

	#[must_use]
	fn ptr_for(&mut self, index: usize) -> *mut C {
		unsafe {
			let base = self.0.as_dense_mut_ptr();
			base.add(self.0.dense_index_of(index).unwrap())
		}
	}
}

impl<'c, C: ecs::Component> IntoIterator for &'c Shard<C> {
	type Item = &'c C;
	type IntoIter = std::slice::Iter<'c, C>;

	fn into_iter(self) -> Self::IntoIter {
		self.as_slice().iter()
	}
}

impl<'c, C: ecs::Component> IntoIterator for &'c mut Shard<C> {
	type Item = &'c mut C;
	type IntoIter = std::slice::IterMut<'c, C>;

	fn into_iter(self) -> Self::IntoIter {
		unsafe { self.as_mut_slice().iter_mut() }
	}
}

const SHARD_LEN: usize = 512;

/// In canonical ECS terms this would be called an "entity" or "entity ID".
///
/// There is no concept of a "null" actor ID. Indicate possible absence via `Option`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActorId {
	/// Only relevant to the [`ShardedSparseSet`].
	pub(super) shard: usize,
	/// Only relevant to the [`Shard`].
	pub(super) index: usize,
	// Q:
	// - Is it faster to use one index and mathematically resolve shards?
	// - Is it faster to use smaller ints and cast each up to `usize`?
}

impl From<ActorId> for usize {
	fn from(value: ActorId) -> Self {
		// `index` is guaranteed to be less than `BUCKET_LEN`. For this to
		// overflow, one would have to allocate orders of magnitude more actors
		// than the largest Doom maps regardless of the width of `usize`
		(value.shard * SHARD_LEN) + value.index
	}
}

impl SparseSetIndex for ActorId {}
