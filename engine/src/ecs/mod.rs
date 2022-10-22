//! Playsim entity components.

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

mod actor;
pub use actor::Actor;

mod bleed;
pub use bleed::Bleed;

mod bounce;
pub use bounce::Bounce;
pub use bounce::BounceFlags;

mod collider;
pub use collider::BoxCollider;

mod cheat;
pub use cheat::Cheat;

mod defense;
pub use defense::DamageFactor;
pub use defense::Defense;
pub use defense::DefenseFlags;

mod health;
pub use health::Health;

mod inv;
pub use inv::Inventory;
pub use inv::Item;
pub use inv::ItemFlags;

mod mons;
pub use mons::Monster;
pub use mons::MonsterFlags;

mod motion;
pub use motion::Motion;
pub use motion::MotionFlags;

mod player;
pub use player::Player;

mod proj;
pub use proj::Projectile;

mod spec;
pub use spec::SpecialVars;

mod transform;
pub use transform::Transform;

mod weap;
pub use weap::Weapon;
pub use weap::WeaponFlags;

use crate::data::game::AssetIndex;

/// Every entity is guaranteed to have this component.
#[derive(Debug, shipyard::Component)]
pub struct Common {
	/// For display to the user. Known in GZDoom as a "tag".
	name: String,
	/// The sim tic on which this entity was spawned.
	spawned_tic: u32,
	blueprint: AssetIndex,
}

/// A template used to instantiate entities.
#[derive(Debug)]
pub struct Blueprint {
	common: Common,

	actor: Option<Actor>,
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
	special_vars: Option<SpecialVars>,
	weapon: Option<Weapon>,
}
