/// Entity (a.k.a. "actor") components for the playsim and renderer.
///
/// Notes to self/the reader/anyone who wants to make changes:
///
/// - These are arranged specifically based on GZDoom's per-sim-tick behavior.
/// If the choices made seem odd, now you know why.
/// - When changing a component's layout or creating a new component, keep an
/// eye on how many 64-byte cache lines it occupies and the effects of the `repr`
/// chosen. `C` is needed for Lith but `Rust` might produce more compact layouts.
use bitflags::bitflags;
use glam::DVec3;

use crate::{math::Rotator64, sim::PlaySim};

/// Currently only a type constraint.
/// Will probably provide functionality to component-managing code in the future.
pub trait Component: Sized {}

// Core ////////////////////////////////////////////////////////////////////////

/// State machine information, an optional player index, flags.
#[derive(Debug)]
pub struct Core {
	pub flags: CoreFlags,
	/// TODO: Actor state machines. For now, pretend this is a pointer.
	pub state_machine: [u8; 8],
	pub state: Option<usize>,
	/// Tics remaining in the current state.
	/// May be -1, which is treated as infinity.
	pub state_tics: i32,
	pub freeze_tics: u32,
	pub player: Option<u8>,
}

bitflags! {
	pub struct CoreFlags: u64 {
		/// The entity is excluded from gameplay-related checks.
		const NO_INTERACTION = 1 << 0;
		/// Level-wide time freezes do not affect this entity.
		const NO_TIMEFREEZE = 1 << 1;
		/// Effectively inert, but displayable.
		const NO_BLOCKMAP = 1 << 2;
		/// The entity is partially inside a scroll sector.
		const SCROLL_SECTOR = 1 << 3;
		const UNMORPHED = 1 << 4;
		const SOLID = 1 << 5;
		const NO_CLIP = 1 << 6;
		const NO_GRAVITY = 1 << 7;
	}
}

impl Core {
	#[must_use]
	pub fn no_interaction(&self) -> bool {
		self.flags.contains(CoreFlags::NO_INTERACTION)
	}

	#[must_use]
	pub fn is_frozen(&self, ctx: &PlaySim) -> bool {
		if self.freeze_tics > 0 {
			return true;
		}

		if self.flags.contains(CoreFlags::NO_TIMEFREEZE) {
			return false;
		}

		if !ctx.level.is_frozen() {
			return false;
		}

		if self.player.is_none() {
			return true;
		}

		let player = &ctx.players[self.player.unwrap() as usize];

		if player.bot.is_some() {
			return true;
		}

		// GZ: This is the only place in the entire engine when the two freeze
		// flags need different treatment. The time freezer flag also freezes
		// other players; the global setting does not.
		ctx.level.is_frozen_local() && player.time_freeze.is_empty()
	}
}

impl Component for Core {}

// Spatial /////////////////////////////////////////////////////////////////////

/// Position, motion, direction, size.
#[derive(Debug)]
pub struct Spatial {
	pub position: DVec3,
	pub velocity: DVec3,
	pub angles: Rotator64,
	pub radius: f64,
	pub height: f64,
	pub floor_z: f64,
	pub ceiling_z: f64,
	pub water_level: WaterLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WaterLevel {
	None,
	Feet,
	Waist,
	Eyes,
}

impl Spatial {
	#[must_use]
	pub fn vel_zero(&self) -> bool {
		self.velocity == DVec3::ZERO
	}
}

impl Component for Spatial {}

// Monster /////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Monster {
	pub flags: MonsterFlags,
	/// Primarily relevant to Nightmare! difficulty.
	pub times_respawned: u16,
}

bitflags! {
	pub struct MonsterFlags: u64 {
		/// For Strife and MBF.
		const FRIENDLY = 1 << 0;
		const KILL_COUNTED = 1 << 1;
		/// Solely for the Pain Elemental's attack.
		/// (VileTech may cull this if it proves possible.)
		const VERT_FRICTION = 1 << 2;
	}
}

impl Monster {
	#[must_use]
	pub fn kill_counted(&self) -> bool {
		self.flags
			.contains(MonsterFlags::KILL_COUNTED & !MonsterFlags::FRIENDLY)
	}
}

impl Component for Monster {}
