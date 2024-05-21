//! A sharded sparse set for use in storing sim state.

use crate::sparse::{SparseSet, SparseSetIndex};

use super::actor::{self, Actor, ActorId};

/// A sharded sparse set for use in storing sim state.
///
/// Each shard is just a [`SparseSet`], pre-allocated with `LEN` capacity,
/// which forbids any insertions exceeding that capacity. This allows Lith
/// to hold pointers to the set's contents without fear that the memory
/// behind the pointer will remain stable.
#[derive(Debug)]
pub struct ShardedSparseSet<T, I, const LEN: usize>(Vec<Shard<T, I, LEN>>)
where
	I: SparseSetIndex;

#[derive(Debug)]
struct Shard<T, I, const LEN: usize>(SparseSet<I, T>)
where
	I: SparseSetIndex;

impl<T, I, const LEN: usize> Shard<T, I, LEN>
where
	I: SparseSetIndex,
{
	#[must_use]
	fn new() -> Self {
		Self(SparseSet::with_capacity(LEN, LEN))
	}
}

pub type ActorSet = ShardedSparseSet<Actor, ActorId, {ActorId::SHARD_LEN}>;
