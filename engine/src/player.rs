//! Intermediary structures between the human (or bot) player and actors.

use bitvec::vec::BitVec;

#[derive(Debug)]
pub struct Player {
	pub bot: Option<Bot>,
	/// Mask indicating time freeze powerup status. Applied between teammates.
	pub time_freeze: BitVec,
}

pub const MAX_PLAYERS: usize = (u8::MAX as usize) + 1;

#[derive(Debug)]
pub struct Bot {
	// ???
}
