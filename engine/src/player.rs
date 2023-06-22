//! Intermediary structures between the human (or bot) player and actors.

use bitvec::vec::BitVec;

use crate::sim::actor::Actor;

/// An intermediate structure between a human player or bot and an [actor].
///
/// [actor]: crate::sim::actor
#[derive(Debug)]
pub struct Player {
	/// What actor is this player controlling?
	pub actor: Option<Actor>,
	pub bot: Option<Bot>,
	/// Mask indicating time freeze powerup status. Applied between teammates.
	pub time_freeze: BitVec,
}

pub const MAX_PLAYERS: usize = (u8::MAX as usize) + 1;

#[derive(Debug)]
pub struct Bot {
	// ???
}
