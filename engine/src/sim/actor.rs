//! "Actors" are ECS entities used

use std::{num::NonZeroI32, ptr::NonNull};

use bevy::prelude::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::{
	data::asset::{self, Blueprint},
	vzs::heap::TPtr,
};

/// Strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Actor(Entity);

pub type ActorPtr = TPtr<VzsActor>;

/// An actor as understood by VZScript.
///
/// A class object, extensible like any other, principally made up of pointers to
/// Bevy components. These get filled in at spawn time and left untouched by
/// internal engine code, except when ECS storages are re-allocated, at which
/// point they are all updated the start of the sim tick.
#[derive(Debug)]
pub struct VzsActor {
	pub(crate) _readonly: NonNull<Readonly>,
	pub(crate) _transform: NonNull<Transform>,

	pub(crate) _monster: Option<NonNull<Monster>>,
	pub(crate) _projectile: Option<NonNull<Projectile>>,
}

// SAFETY: Pointers are never dereferenced by native Rust, only set.
// Only VZS modifies their contents, and this only happens when there is already
// an open Bevy query for mutating all components, so no other references can exist.
unsafe impl Send for VzsActor {}
unsafe impl Sync for VzsActor {}

// Monster /////////////////////////////////////////////////////////////////////

#[derive(Debug, Component)]
pub struct Monster {
	pub goal: Option<ActorPtr>,
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

// Readonly ////////////////////////////////////////////////////////////////////

/// Details about an actor that are set at spawn time and never change.
#[derive(Debug, Component)]
pub struct Readonly {
	/// The blueprint off which this actor is based.
	pub blueprint: asset::Handle<Blueprint>,
	/// Universally unique.
	pub id: Actor,
	/// From GZ; affects render order somehow. VileTech may cull.
	pub spawn_order: u32,
	/// The tick this actor was spawned on.
	pub spawn_tick: u32,
}

// Projectile //////////////////////////////////////////////////////////////////

#[derive(Debug, Component)]
pub struct Projectile {
	pub flags: ProjectileFlags,
	pub knockback: i32,
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

// Details /////////////////////////////////////////////////////////////////////

/// A holdover from Raven Software games and their Action Code Script (ACS).
///
/// Sometimes abbreviated to "TID". Benefits from non-zero optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThingId(NonZeroI32);

/*

TODO:

Fields which are a part of ZScript's `Actor` API that will need coverage somehow.

- [ ] readonly Actor snext -> can maybe be faked by VileTech?
- [ ] PlayerInfo Player -> store an index instead
- [ ] readonly ViewPosition ViewPos
- [X] readonly vector3 Pos -> bevy::Transform
- [ ] vector3 Prev
- [ ] uint ThruBits
- [ ] vector2 SpriteOffset
- [ ] vector3 WorldOffset
- [ ] double spriteAngle
- [ ] double spriteRotation
- [ ] float VisibleStartAngle
- [ ] float VisibleStartPitch
- [ ] float VisibleEndAngle
- [ ] float VisibleEndPitch
- [X] double Angle -> bevy::Transform
- [X] double Pitch -> bevy::Transform
- [X] double Roll -> bevy::Transform
- [ ] vector3 Vel
- [ ] double Speed
- [ ] double FloatSpeed
- [ ] SpriteID sprite
- [ ] uint8 frame
- [ ] vector2 Scale
- [ ] TextureID picnum
- [ ] double Alpha
- [ ] readonly color fillcolor
- [ ] Sector CurSector
- [ ] double CeilingZ
- [ ] double FloorZ
- [ ] double DropoffZ
- [ ] Sector floorsector
- [ ] TextureID floorpic
- [ ] int floorterrain
- [ ] Sector ceilingsector
- [ ] TextureID ceilingpic
- [ ] double Height
- [ ] readonly double Radius
- [ ] readonly double RenderRadius
- [ ] double projectilepassheight
- [ ] int tics
- [ ] readonly State CurState
- [ ] readonly int Damage
- [X] int projectilekickback
- [ ] int special1
- [ ] int special2
- [ ] double specialf1
- [ ] double specialf2
- [ ] int weaponspecial
- [ ] int Health
- [ ] uint8 movedir
- [ ] int8 visdir
- [ ] int16 movecount
- [ ] int16 strafecount
- [ ] Actor Target
- [ ] Actor Master
- [ ] Actor Tracer
- [ ] Actor LastHeard
- [ ] Actor LastEnemy
- [ ] Actor LastLookActor
- [X] int ReactionTime
- [X] int Threshold
- [ ] readonly int DefThreshold
- [ ] vector3 SpawnPoint
- [ ] uint16 SpawnAngle
- [ ] int StartHealth
- [X] uint8 WeaveIndexXY
- [X] uint8 WeaveIndexZ
- [ ] uint16 skillrespawncount
- [ ] int Args[5]
- [ ] int Mass
- [ ] int Special
- [ ] readonly int TID
- [X] readonly int TIDtoHate
- [ ] readonly int WaterLevel
- [ ] readonly double WaterDepth
- [ ] int Score
- [ ] int Accuracy
- [ ] int Stamina
- [ ] double MeleeRange
- [ ] int PainThreshold
- [ ] double Gravity
- [ ] double FloorClip
- [ ] name DamageType
- [ ] name DamageTypeReceived
- [ ] uint8 FloatBobPhase
- [ ] double FloatBobStrength
- [X] int RipperLevel
- [ ] int RipLevelMin
- [ ] int RipLevelMax
- [ ] name Species
- [ ] Actor Alternative
- [X] Actor goal
- [ ] uint8 MinMissileChance
- [ ] int8 LastLookPlayerNumber
- [ ] uint SpawnFlags
- [ ] double meleethreshold
- [ ] double maxtargetrange
- [ ] double bouncefactor
- [ ] double wallbouncefactor
- [ ] int bouncecount
- [ ] double friction
- [ ] int FastChaseStrafeCount
- [ ] double pushfactor
- [ ] int lastpush
- [ ] int activationtype
- [ ] int lastbump
- [ ] int DesignatedTeam
- [ ] Actor BlockingMobj
- [ ] Line BlockingLine
- [ ] Sector Blocking3DFloor
- [ ] Sector BlockingCeiling
- [ ] Sector BlockingFloor
- [ ] int PoisonDamage
- [ ] name PoisonDamageType
- [ ] int PoisonDuration
- [ ] int PoisonPeriod
- [ ] int PoisonDamageReceived
- [ ] name PoisonDamageTypeReceived
- [ ] int PoisonDurationReceived
- [ ] int PoisonPeriodReceived
- [ ] Actor Poisoner
- [ ] Inventory Inv
- [ ] uint8 smokecounter
- [ ] uint8 FriendPlayer
- [ ] uint Translation
- [ ] sound AttackSound
- [ ] sound DeathSound
- [ ] sound SeeSound
- [ ] sound PainSound
- [ ] sound ActiveSound
- [ ] sound UseSound
- [ ] sound BounceSound
- [ ] sound WallBounceSound
- [ ] sound CrushPainSound
- [ ] double MaxDropoffHeight
- [ ] double MaxStepHeight
- [ ] double MaxSlopeSteepness
- [ ] int16 PainChance
- [ ] name PainType
- [ ] name DeathType
- [ ] double DamageFactor
- [ ] double DamageMultiply
- [ ] Class<Actor> TelefogSourceType
- [ ] Class<Actor> TelefogDestType
- [ ] readonly State SpawnState
- [ ] readonly State SeeState
- [ ] State MeleeState
- [ ] State MissileState
- [ ] voidptr DecalGenerator
- [ ] uint8 fountaincolor
- [ ] double CameraHeight
- [ ] double CameraFOV
- [ ] double ViewAngle, ViewPitch, ViewRoll
- [ ] double RadiusDamageFactor
- [ ] double SelfDamageFactor
- [ ] double StealthAlpha
- [ ] int WoundHealth
- [ ] readonly color BloodColor
- [ ] readonly int BloodTranslation
- [ ] int RenderHidden
- [ ] int RenderRequired
- [ ] int FriendlySeeBlocks
- [ ] int16 lightlevel
- [X] readonly int SpawnTime
- [ ] uint freezetics

- [ ] meta String Obituary
- [ ] meta String HitObituary
- [ ] meta double DeathHeight
- [ ] meta double BurnHeight
- [ ] meta int GibHealth
- [ ] meta Sound HowlSound
- [ ] meta Name BloodType
- [ ] meta Name BloodType2
- [ ] meta Name BloodType3
- [ ] meta bool DontHurtShooter
- [ ] meta int ExplosionRadius
- [ ] meta int ExplosionDamage
- [ ] meta int MeleeDamage
- [ ] meta Sound MeleeSound
- [ ] meta Sound RipSound
- [ ] meta double MissileHeight
- [ ] meta Name MissileName
- [ ] meta double FastSpeed

*/
