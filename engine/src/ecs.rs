//! All symbols involved in implementing VileTech's custom playsim ECS.

mod components;

use std::collections::VecDeque;

use rayon::prelude::*;

use crate::sparse::{SparseSet, SparseSetIndex};

pub use components::{Constant, SpecialVars};

// ID newtype //////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct EntityId(pub(self) usize);

impl From<EntityId> for usize {
	fn from(entity: EntityId) -> Self {
		entity.0
	}
}

impl SparseSetIndex for EntityId {}

// Entity storage //////////////////////////////////////////////////////////////

/// Structure for tracking entity state, kept separate from [`Components`] storage
/// to allow them to be borrowed independently. Keeps a dynamic array of entity
/// IDs in no specific order. An entity is extant if it is in this array, and
/// absent otherwise.
///
/// It's named the way it is in case a `SparseRegistry` gets implemented and
/// proves to be situationally better.
///
/// # Tradeoffs
///
/// - Iterations are as fast as they possibly can be, since the array slice can
/// be looped over as-is.
/// - Insertions into the array require a free entity be popped off the front
/// of the absent queue and pushed to the front of the extant array.
/// - Trying to find an existing entity's index in the array is always `O(n)`.
/// In the worst case, the whole array needs to be traversed to perform an
/// existence check or removal.
#[derive(Debug)]
pub struct DenseRegistry {
	/// Initialized to the length of `absent`.
	next_open: usize,
	absent: VecDeque<EntityId>,
	extant: Vec<EntityId>,
}

impl DenseRegistry {
	/// `hint` is meant to inform what size of allocation should be pre-reserved.
	/// This should come from the number of "things" that were pre-placed in a
	/// level (monsters, inventory items, decorations). Extra space beyond `hint`
	/// is always allocated to account for projectiles, hitscan puffs, teleport
	/// effects, Lost Souls, and whatever other entities a mod may spawn dynamically.
	/// This extra headroom diminishes as `hint` increases; it is assumed that
	/// DOOM E1M1 requires less headroom as a proportion of its initial thing
	/// count than the larger slaughtermaps which don't frontload all of their
	/// monsters at once.
	#[must_use]
	pub fn new(hint: usize) -> Self {
		assert_ne!(
			hint, 0,
			"Illegally tried to create an entity registry with 0 capacity."
		);

		let extra = Self::extra_alloc(hint);

		let mut ret = Self {
			next_open: extra,
			absent: VecDeque::with_capacity(extra),
			extant: Vec::with_capacity(hint + extra),
		};

		for i in 0..extra {
			ret.absent.push_back(EntityId(i));
		}

		ret
	}

	#[must_use]
	pub(self) fn extra_alloc(hint: usize) -> usize {
		match hint {
			0 => unreachable!(),
			1..=69 => {
				// Thing count for DOOM2 MAP01 is 69
				((hint as f32) * 0.5) as usize
			}
			70..=521 => {
				// Thing count for Plutonia MAP32 is 521
				((hint as f32) * 0.33) as usize
			}
			522..=1817 => {
				// Thing count for Scythe 2 MAP30 is 1817
				((hint as f32) * 0.2) as usize
			}
			1818..=3232 => {
				// Thing count for Alien Vendetta MAP25 is 3232
				((hint as f32) * 0.1) as usize
			}
			3233..=5537 => {
				// Thing count for Slaughterfest 2012 MAP25 is 5537
				((hint as f32) * 0.05) as usize
			}
			_ => {
				// Thing count for Cosmogenesis MAP05 is 81036 (!!!)
				((hint as f32) * 0.01) as usize
			}
		}
	}

	pub fn iter_extant(&self) -> impl Iterator<Item = EntityId> + '_ {
		self.extant.iter().copied()
	}

	pub fn par_iter_extant(&self) -> impl ParallelIterator<Item = EntityId> + '_ {
		self.extant.par_iter().copied()
	}

	pub fn spawn(&mut self) -> EntityId {
		if let Some(new_ent) = self.absent.pop_front() {
			self.extant.push(new_ent);
			new_ent
		} else {
			// - "Last" entity is N
			// - Next open entity is N + 1
			// - Advance `next_open` to N + 3
			// - N + 1 becomes extant
			// - N + 2 becomes absent as extra headroom

			let next_open = EntityId(self.next_open);
			self.extant.push(next_open);
			self.absent.push_back(EntityId(next_open.0 + 1));
			self.next_open += 2;
			next_open
		}
	}

	pub fn spawn_bulk(&mut self, count: usize) {
		for _ in 0..count {
			let _ = self.spawn();
		}
	}

	#[must_use]
	pub fn exists(&self, entity: EntityId) -> bool {
		self.extant.iter().any(|e| *e == entity)
	}

	pub fn remove_unchecked(&mut self, entity: EntityId) {
		self.extant.remove(
			self.extant
				.iter()
				.position(|e| *e == entity)
				.expect("`DenseRegistry::remove_unchecked` failed to find the given entity."),
		);

		self.absent.push_back(entity);
	}
}

// Component storage ///////////////////////////////////////////////////////////

/// Component collections are kept separate from the [entity registry](DenseRegistry)
/// to allow them to be borrowed independently. All members are kept public deliberately.
#[derive(Debug)]
pub struct Components {
	pub constant: SparseSet<EntityId, Constant>,
	pub special: SparseSet<EntityId, SpecialVars>,
}

impl Components {
	/// See [`DenseRegistry::new`], which consumes `hint` the same way, for details
	/// on how component collection capacity gets reserved.
	#[must_use]
	pub fn new(hint: usize) -> Self {
		assert_ne!(
			hint, 0,
			"Illegally tried to create a component storage with 0 capacity."
		);

		let extra = DenseRegistry::extra_alloc(hint);

		Self {
			constant: SparseSet::with_capacity(hint + extra, hint + extra),
			special: SparseSet::with_capacity(hint + extra, hint + extra),
		}
	}

	pub fn clear(&mut self) {
		self.constant.clear();
		self.special.clear();
	}
}
