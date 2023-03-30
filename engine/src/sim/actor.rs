//! The principal game simulation entity structure.

use std::num::{NonZeroI32, NonZeroUsize};

use bitflags::bitflags;
use glam::{DVec2, DVec3};

use crate::{
	data::{self, Blueprint, DamageType, Sound, Species},
	gfx::Rgb32,
	math::{Angle32, Angle64, Rotator64},
	sparse::SparseSetIndex,
};

/// A game simulation entity; may be a monster, player character, inventory item,
/// decoration, projectile, or similar. Sometimes known as a "map object" or
/// "mobj" for short. Maps to the `AActor` class in (G)ZDoom's internals and the
/// `Actor` class in both ZScript and Lith.
///
/// VileTech's gameplay code needs to be somewhat similar to that of ZDoom for a
/// certain degree of backwards compatibility. As such, sim logic will be built
/// up on this type; with time, work will be done to optimize and trim away
/// parts that have aged poorly in favour of more flexible solutions.
///
/// Notes:
/// - This type is composed of several sub-structs for cleanliness and clarity.
/// Some may even get boxed if their fields are so rarely used that the indirection
/// turns into a memory layout optimization.
/// - This type has no methods of its own. Many of its behaviors depend on the
/// outside context of the playsim state getting passed in, so in the interest
/// of total consistency, the playsim has all actor functionality as either
/// methods or associated functions.
#[derive(Debug)]
pub struct Actor {
	pub bounce: Bounce,
	pub core: Core,
	pub fsm: StateMachine,
	pub health: Defense,
	pub monster: Monster,
	pub projectile: Projectile,
	pub readonly: Readonly,
	pub spatial: Spatial,
	pub special: Special,
	pub sounds: Sounds,
	pub trivia: Trivia,
	pub visual: Visual,
}

bitflags! {
	/// These are the overly-specific flags - generally from ZDoom's early days -
	/// which, preferably, will be generalized away to avoid unnecessary branches
	/// in hot loops.
	pub struct SpecialtyFlags: u64 {
		const BOSS_CUBE = 1 << 0;
		const FX_ROCKET = 1 << 1;
		const FX_GRENADE = 1 << 2;
		const FX_RESPAWN_INVULN = 1 << 3;
		const FX_VISIBILITY_PULSE = 1 << 4;
	}
}

// Bounce //////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Bounce {
	flags: BounceFlags,
	factor: f64,
	/// Velocity loss upon bouncing off a wall.
	wall_factor: f64,
	/// How many times has this actor bounced thus far?
	count: u32,
}

bitflags! {
	pub struct BounceFlags: u32 {
		const ACTORS = 1 << 0;
		const ALL_ACTORS = 1 << 1;
		/// When bouncing off a sector plane, if the new Z velocity is below 3.0,
		/// disable further bouncing.
		const AUTO_OFF = 1 << 2;
		const AUTO_OFF_FLOOR_ONLY = 1 << 3;
		/// The actor is allowed to bounce off ceiling geometry.
		const CEILINGS = 1 << 4;
		/// From GZ. Purpose unclear.
		const DEHACKED = 1 << 5;
		/// The actor is allowed to bounce off floor geometry.
		const FLOORS = 1 << 6;
		/// Heretic-style bouncing.
		/// Goes into `Death` f-state when bouncing on floors or ceilings.
		const FLOOR_CEILING_DEATH = 1 << 7;
		/// - Actor is treated like a projectile for various tests even if it is
		/// not flagged as such.
		/// - Actor does not lose bouncing properties even when slowing down and
		/// coming to a rest.
		const MBF = 1 << 8;
		/// If set, don't bounce off walls when on frictionless ground (e.g. ice).
		const NO_FRICTION = 1 << 9;
		/// The actor bounces on actors marked [`CoreFlags::NO_RIP`].
		const NO_RIP = 1 << 10;
		/// No noise made when bouncing at all.
		const QUIET = 1 << 11;
		/// If actor 1 is a bouncy projectile hitting actor 2, which can be hit
		/// by projectiles, bounce if set; else, explode or despawn.
		const SHOOTABLES = 1 << 12;
		/// The actor is allowed to bounce off floors/ceilings/walls set to show
		/// the skybox texture, when they would normally despawn.
		const SKY = 1 << 13;
		const USE_SEE_SOUND = 1 << 14;
		/// Change to special-purpose f-states marked `Bounce[.*]`
		/// under certain circumstances.
		const USE_FSTATES = 1 << 15;
		/// The actor is allowed to bounce off wall geometry.
		const WALLS = 1 << 16;
		/// If unset, make no noise upon bouncing off a wall.
		const WALL_SOUND = 1 << 17;
		/// The actor is allowed to bounce off liquid surfaces.
		const WATER = 1 << 18;
		/// Explodes when hitting a water surface.
		/// (RAT) Not sure why GZ has this as part of these flags...?
		const WATER_EXPLODE = 1 << 19;

		const TYPEMASK =
			Self::ACTORS.bits | Self::AUTO_OFF.bits | Self::HERETIC.bits |
			Self::WALLS.bits | Self::FLOORS.bits | Self::CEILINGS.bits |
			Self::MBF.bits;

		const DOOM =
			Self::ACTORS.bits | Self::AUTO_OFF.bits |
			Self::CEILINGS.bits | Self::FLOORS.bits | Self::WALLS.bits;
		const HERETIC =
			Self::CEILINGS.bits | Self::FLOORS.bits | Self::FLOOR_CEILING_DEATH.bits;
		const HEXEN =
			Self::ACTORS.bits |
			Self::CEILINGS.bits | Self::FLOORS.bits | Self::WALLS.bits;
		const GRENADE = Self::DOOM.bits | Self::MBF.bits;
		const CLASSIC = Self::CEILINGS.bits | Self::FLOORS.bits | Self::MBF.bits;

		const DOOM_COMPAT = Self::DOOM.bits | Self::USE_SEE_SOUND.bits;
		const HERETIC_COMPAT = Self::HERETIC.bits | Self::USE_SEE_SOUND.bits;
		const HEXEN_COMPAT = Self::HEXEN.bits | Self::USE_SEE_SOUND.bits;
	}
}

// Core ////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Core {
	pub flags: CoreFlags,
	pub flags_specialty: SpecialtyFlags,
	/// An actor's held items are kept in a singly-linked list.
	pub inventory: Option<ActorId>,
	pub name: String, // TODO: String interning.
	pub player: Option<u8>,
	/// Mostly only relevant to monster infighting (e.g. Barons and Hell Knights).
	pub species: Option<data::Handle<Species>>,

	/// The next actor in this sector's doubly-linked list.
	pub s_next: Option<ActorId>,
	/// The previous actor in this sector's doubly-linked list.
	pub s_prev: Option<ActorId>,
}

bitflags! {
	pub struct CoreFlags: u64 {
		const IN_CONVERSATION = 1;
		const NO_INTERACT = Self::IN_CONVERSATION.bits << 1;
		/// Piercing projectiles explode or despawn on contact with this actor.
		const NO_RIP = Self::NO_INTERACT.bits << 1;
		/// Seeking missile cannot home in on this actor.
		const NO_SEEK = Self::NO_RIP.bits << 1;
		const NO_SKIN = Self::NO_SEEK.bits << 1;
		const NO_TIMEFREEZE = Self::NO_SKIN.bits << 1;
	}
}

// Defense /////////////////////////////////////////////////////////////////////

/// Sub-structure for composing [`Actor`].
/// Health, incoming damage factors, damage over time state.
#[derive(Debug)]
pub struct Defense {
	pub health: i32,
	pub health_wounded: i32,
	/// D.O.T. effects being applied to this actor.
	pub dmg_over_time: Vec<DamageOverTime>,
	pub self_dmg_factor: f64,
	pub radius_dmg_factor: f64,
	/// Only projectiles with a [`rip_level`](Projectile::rip_level) of this
	/// or higher can pierce through this actor. Mostly for monsters.
	pub rip_level_min: i32,
	/// Only projectiles with a [`rip_level`](Projectile::rip_level) of this
	/// or lower can pierce through this actor. Mostly for monsters.
	pub rip_level_max: i32,
}

/// A status effect attached to an actor which deals some damage on a fixed tick
/// interval for a set amount of ticks before wearing off.
#[derive(Debug)]
pub struct DamageOverTime {
	/// Applied per tick.
	pub damage: i32,
	pub damage_type: data::Handle<DamageType>,
	pub remaining_ticks: u32,
	/// Damage is applied every `period` ticks.
	pub period: u32,
	pub source: Option<ActorId>,
}

// Monster /////////////////////////////////////////////////////////////////////

/// Sub-structure for composing [`Actor`].
#[derive(Debug)]
pub struct Monster {
	pub goal: Option<ActorId>,
	pub missile_threshold: f64,
	pub reaction_time: i32,
	pub tid_hated: Option<ThingId>,
}

bitflags! {
	pub struct MonsterFlags: u64 {
		/// Mutually exclusive with `NEVER_RESPAWN`.
		const ALWAYS_RESPAWN = 1;
		const AVOID_HAZARDS = 1 << 1;
		/// Monster is currently running away from a player.
		const FRIGHTENED = 1 << 2;
		/// Mutually exclusive with `ALWAYS_RESPAWN`.
		const NEVER_RESPAWN = 1 << 3;
		const NO_INFIGHT = 1 << 4;
		const RADIAL_VISION = 1 << 5;
		const STAY_MORPHED = 1 << 6;
		const STAY_ON_LIFT = 1 << 7;
		/// For monsters spawned at map-load time only.
		/// Prevents randomization of state lengths.
		const SYNCHRONIZED = 1 << 8;
	}
}

// Projectile //////////////////////////////////////////////////////////////////

/// Sub-structure for composing [`Actor`].
#[derive(Debug)]
pub struct Projectile {
	pub flags: ProjectileFlags,
	pub knockback: i32,
	/// See [`Defense::rip_level_min`] and [`Defense::rip_level_max`].
	pub rip_level: i32,
	/// Will be in a range from 0 to 63 inclusively.
	pub weave_index_xy: u8,
	/// Will be in a range from 0 to 63 inclusively.
	pub weave_index_z: u8,
}

bitflags! {
	pub struct ProjectileFlags: u64 {
		const CAN_HIT_OWNER = 1;
		/// Plays expiration/explosion sound at full volume.
		const FULL_VOL_DEATH = Self::CAN_HIT_OWNER.bits << 1;
		const HUG_FLOOR = Self::FULL_VOL_DEATH.bits << 1;
		const HUG_CEILING = Self::HUG_FLOOR.bits << 1;
		const KNOCKBACK = Self::HUG_CEILING.bits << 1;
		const SEEK_INVIS = Self::KNOCKBACK.bits << 1;
		const SET_MASTER_ON_HIT = Self::SEEK_INVIS.bits << 1;
		const SET_TARGET_ON_HIT = Self::SET_MASTER_ON_HIT.bits << 1;
		const SET_TRACER_ON_HIT = Self::SET_TARGET_ON_HIT.bits << 1;
		const STEP_CLIMB = Self::SET_TRACER_ON_HIT.bits << 1;
	}
}

// Read-only ///////////////////////////////////////////////////////////////////

/// Sub-structure for composing [`Actor`].
/// Details about an actor that are set at spawn time and never change.
#[derive(Debug)]
pub struct Readonly {
	/// The blueprint off which this actor is based.
	pub blueprint: data::Handle<Blueprint>,
	/// Universally-unique amongst actors.
	pub id: ActorId,
	/// From GZ; affects render order somehow. VT may cull.
	pub spawn_order: u32,
	/// The tick this actor was created on.
	pub spawn_tick: u32,
}

// Sounds //////////////////////////////////////////////////////////////////////

/// Sub-structure for composing [`Actor`].
/// Sounds emitted by an actor in response to certain triggers.
#[derive(Debug)]
pub struct Sounds {
	on_active: Option<data::Handle<Sound>>,
	on_attack: Option<data::Handle<Sound>>,
	on_bounce: Option<data::Handle<Sound>>,
	on_bounce_wall: Option<data::Handle<Sound>>,
	on_crush_pain: Option<data::Handle<Sound>>,
	on_death: Option<data::Handle<Sound>>,
	on_pain: Option<data::Handle<Sound>>,
	on_see: Option<data::Handle<Sound>>,
	on_use: Option<data::Handle<Sound>>,
}

// Spatial /////////////////////////////////////////////////////////////////////

/// Sub-structure for composing [`Actor`].
/// Position, motion, direction, physical size.
#[derive(Debug)]
pub struct Spatial {
	/// Yaw, pitch, and roll.
	pub angles: Rotator64,
	pub flags: SpatialFlags,
	pub friction: f64,
	pub gravity: f64,
	pub height: f64,
	pub max_dropoff_height: f64,
	pub max_step_height: f64,
	pub max_slope_steepness: f64,
	pub pos: DVec3,
	pub push_factor: f64,
	pub radius: f64,
	pub velocity: DVec3,
	/// In map units.
	pub water_depth: f64,
	pub water_level_boom: WaterLevel,
	pub water_level: WaterLevel,
}

bitflags! {
	pub struct SpatialFlags: u64 {
		const SOLID = 1 << 0;
		/// Essentially inert, but displayable.
		const NO_BLOCKMAP = 1 << 1;
		const IN_SCROLL_SECTOR = 1 << 2;
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaterLevel {
	None,
	Feet,
	Waist,
	Eyes,
}

// Special /////////////////////////////////////////////////////////////////////

/// Sub-structure for composing [`Actor`].
/// Fields for use by ACS and map programming, including [`thing ID`](ThingId).
#[derive(Debug)]
pub struct Special {
	pub tid: ThingId,
	pub special_args: [i32; 5],
	/// Element 0 corresponds to `AActor::specialf1`; 1 corresponds to `specialf2`.
	pub special_f: [f64; 2],
	/// Element 0 corresponds to `AActor::special1`; 1 corresponds to `special2`.
	pub special_i: [i32; 2],
}

// State machine ///////////////////////////////////////////////////////////////

/// Sub-structure for composing [`Actor`].
#[derive(Debug)]
pub struct StateMachine {
	pub state: usize,
	/// Ticks remaining in the current state.
	/// May be -1, which is treated as infinity.
	pub state_ticks: i32,
	pub frame: u8,
	pub freeze_ticks: u32,
}

// Trivia //////////////////////////////////////////////////////////////////////

/// Sub-structure for composing [`Actor`].
/// Holds fields which are not accessed by hot code.
/// May eventually get boxed to keep `Actor` small.
#[derive(Debug)]
pub struct Trivia {
	/// Only for ACS use.
	pub accuracy: i32,
	/// Only for ACS use, and context-sensitive maximum health calculations.
	pub stamina: i32,
}

// Visual //////////////////////////////////////////////////////////////////////

/// Sub-structure for composing [`Actor`].
#[derive(Debug)]
pub struct Visual {
	pub alpha: f64,
	pub color: Rgb32,
	pub flags_render: RenderFlags,
	pub floor_clip: f64,
	pub translation: u32,
	pub scale: DVec2,
	pub sprite_angle: Angle64,
	pub sprite_offset: DVec3,
	pub sprite_rotation: Angle64,
	pub sprite: i32,
	pub visible_end_angle: Angle32,
	pub visible_end_pitch: Angle32,
	pub visible_start_angle: Angle32,
	pub visible_start_pitch: Angle32,
}

bitflags! {
	pub struct RenderFlags: u64 {
		const X_FLIP = 1 << 0;
		const Y_FLIP = 1 << 1;
		const ONE_SIDED = 1 << 2;
		const FULL_BRIGHT = 1 << 3;
		const REL_MASK = 1 << 4;
		const REL_ABSOLUTE = 1 << 5;
		const REL_UPPER = 1 << 6;
		const REL_LOWER = 1 << 7;
		const REL_MID = 1 << 8;
		const CLIP_MASK = 1 << 9;
		const CLIP_FULL = 1 << 10;
		const CLIP_UPPER = 1 << 11;
		const CLIP_MID = 1 << 12;
		const CLIP_LOWER = 1 << 13;
		const DECAL_MASK = Self::REL_MASK.bits | Self::CLIP_MASK.bits;
		const SPRITE_TYPE_MASK = 1 << 14;
		const FACE_SPRITE = 1 << 15;
		const WALL_SPRITE = 1 << 16;
		const FLAT_SPRITE = 1 << 17;
		const VOXEL_SPRITE = 1 << 18;
		const INVISIBLE = 1 << 19;
		const FORCE_Y_BILLBOARD = 1 << 20;
		const FORCE_XY_BILLBOARD = 1 << 21;
		const ROLL_SPRITE = 1 << 22;
		const NO_FLIP = 1 << 23;
		const ROLL_CENTER = 1 << 24;
		const MASK_ROTATION = 1 << 25;
		const ABS_MASK_ANGLE = 1 << 26;
		const ABS_MASK_PITCH = 1 << 27;
		const INTERPOLATE_ANGLES = 1 << 28;
		const MAYBE_INVISIBLE = 1 << 29;
		const NO_INTERPOLATE = 1 << 30;
		const SPRITE_FLIP = 1 << 31;
		const ZDOOM_TRANSP = 1 << 32;
		const CAST_SPRITE_SHADOW = 1 << 33;
		const NO_INTERPOLATE_VIEW = 1 << 34;
		const NO_SPRITE_SHADOW = 1 << 35;
		const MIRROR_INVIS = 1 << 36;
		const MIRROR_VIS_ONLY = 1 << 37;
	}
}

// Details /////////////////////////////////////////////////////////////////////

/// Benefits from non-zero optimization. Represented Lith-side as a pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ActorId(NonZeroUsize);

impl From<ActorId> for usize {
	fn from(value: ActorId) -> Self {
		value.0.get()
	}
}

impl SparseSetIndex for ActorId {}

/// A holdover from Raven Software games and their Action Code Script.
/// Sometimes abbreviated to "TID". Benefits from non-zero optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ThingId(NonZeroI32);
