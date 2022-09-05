/*
Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use crate::{data::game::AssetIndex, game::DamageOverTime, gfx::Rgb32};
use bitflags::bitflags;
use glam::{Vec2, Vec3};
use shipyard::EntityId;
use std::collections::LinkedList;

#[derive(Default)]
pub struct Blueprint {
	common: Common,
	bleed: Option<Bleed>,
	bounce: Option<Bounce>,
	box_collider: Option<BoxCollider>,
	defense: Option<Defense>,
	health: Option<Health>,
	inventory: Option<Inventory>,
	item: Option<Item>,
	monster: Option<Monster>,
	motion: Option<Motion>,
	player: Option<Player>,
	projectile: Option<Projectile>,
	weapon: Option<Weapon>,
	special_args: Option<SpecialArgs>,
}

pub struct Bleed {
	color: Rgb32,
	blueprints: [Option<AssetIndex>; 3]
}

pub struct Bounce {
	flags: BounceFlags,
	/// How many times can this entity bounce before expiring?
	/// If the blueprint doesn't specify otherwise, this is effectively infinite.
	max_count: u32,
	lostvel_wall: f32,
	lostvel_floorceil: f32,
	/// How many times has this entity bounced thus far?
	bounces: u32,
}

bitflags! {
	pub struct BounceFlags: u16 {
		const NONE = 0;
		const ON_WALLS = 1 << 0;
		const ON_FLOORS = 1 << 1;
		const ON_CEILINGS = 1 << 2;
		const AUTO_OFF = 1 << 3;
		const AUTO_OFF_FLOOR_ONLY = 1 << 4;
		const LIKE_HERETIC = 1 << 5;
		const ON_ACTORS = 1 << 6;
		const ON_UNRIPPABLE = 1 << 7;
		const NO_WALL_SOUND = 1 << 8;
		const STATE_RESOLVE = 1 << 9;
		const MBF = 1 << 15;
	}
}

pub struct BoxCollider {
	/// Applies on both axes.
	width: f32,
	height: f32,
}

pub struct Cheat {}

#[derive(Default)]
pub struct Common {
	/// Known in GZDoom as a "tag".
	name: String,
	/// Game tics since this entity was spawned.
	lifetime: u32,
}

pub struct Defense {
	damage_factors: Vec<(AssetIndex, f32)>,
}

pub struct Health {
	current: i32,
	max: i32,
	dmg_over_time: Vec<DamageOverTime>,
	gib_health: i32,
}

/// Allows the entity to carry `Item` entities.
pub struct Inventory {
	list: LinkedList<EntityId>,
}

bitflags! {
	pub struct InventoryFlags : u8 {
		const NONE = 0;
		const INVBAR = 1 << 0;
		const KEEP_DEPLETED = 1 << 1;
		const AUTO_ACTIVATE = 1 << 2;
		const UNDROPPABLE = 1 << 3;
		const UNCLEARABLE = 1 << 4;
		const NO_RESPAWN = 1 << 5;
	}
}

/// Marks the entity as an inventory item.
pub struct Item {
	amount: u32,
	max_amount: u32,
	interhub_amount: u32,
}

pub struct Monster {
	flags: MonsterFlags,
	species: AssetIndex,
	/// In game-tics.
	reaction_time: u32,
	/// ::0 is the key for the level on which the trigger should fire when this
	/// monster is killed. ::1 is true if the trigger is secondary, false if primary.
	/// e.g. ("MAP07", false) for the Mancubus, ("MAP07", true) for the Arachnotron.
	boss_triggers: Vec<(String, bool)>,
}

bitflags! {
	pub struct MonsterFlags : u8 {
		const NONE = 0;
		const AMBUSH = 1 << 0;
		const AVOID_HAZARDS = 1 << 1;
		const FAST_RETALIATE = 1 << 2;
		const NEVER_RESPAWN = 1 << 3;
		const NO_SPLASH_ALERT = 1 << 4;
	}
}

pub struct Motion {
	flags: MotionFlags,
	mass: u32,
	velocity: Vec3,
	accel: Vec3,
	friction: f32,
	gravity: f32,
	max_step_height: f32,
	max_dropoff_height: f32,
	max_slope_steepness: f32,
}

bitflags! {
	pub struct MotionFlags : u8 {
		const NONE = 0;
		const NOGRAVITY = 1 << 0;
		const FALLDAMAGE = 1 << 1;
	}
}

/// This entity is currently under the control of a player.
pub struct Player {
	index: u8,
}

pub struct Projectile {}

pub struct SpecialArgs {
	args: [i32; 5],
}

pub struct Transform {
	pos: Vec3,
	angle: f32,
	pitch: f32,
	roll: f32,
}

/// Allows an `Item` entity to be wielded as a weapon.
pub struct Weapon {
	flags: WeaponFlags,
	bob_style: WeaponBobStyle,
	bob_range: Vec2,
	slot_priority: i16,
	fallback_priority: i16,
	crosshair: AssetIndex,
}

#[repr(u8)]
enum WeaponBobStyle {
	Normal,
	Alpha,
	Smooth,
	InverseNormal,
	InverseAlpha,
	InverseSmooth,
}

bitflags! {
	pub struct WeaponFlags : u8 {
		const NONE = 0;
		const NO_AUTOAIM = 1 << 0;
		/// If wielding a
		const NO_PICKUP_AUTOSWITCH = 1 << 1;
		/// If wielding this weapon, and ammo is collected for another weapon which
		/// is not `WEAK` and not marked `NO_FROMWEAK_AUTOSWITCH`, automatically
		/// switch to that weapon at the first possible opportunity.
		const WEAK = 1 << 2;
		const NO_FROMWEAK_AUTOSWITCH = 1 << 3;
		const NOBOB = 1 << 4;
		const ALLOW_DEATH_INPUT = 1 << 5;
	}
}
