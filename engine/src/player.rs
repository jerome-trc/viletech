use bitvec::vec::BitVec;

use crate::sim::ActorId;

#[derive(Debug)]
pub struct Player {
	/// What actor is this player controlling?
	pub actor: ActorId,
	pub bot: Option<Bot>,
	/// Mask indicating time freeze powerup status. Applied between teammates.
	pub time_freeze: BitVec,
}

pub const MAX_PLAYERS: usize = (u8::MAX as usize) + 1;

#[derive(Debug)]
pub struct Bot {
	// ???
}
