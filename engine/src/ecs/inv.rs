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

use std::collections::VecDeque;

use shipyard::{Component, EntityId};

/// Allows the entity to carry `Item` entities.
#[derive(Debug, Component)]
pub struct Inventory {
	items: VecDeque<EntityId>,
}

/// Marks the entity as an inventory item.
#[derive(Debug, Component)]
pub struct Item {
	flags: ItemFlags,
	amount: u32,
	max_amount: u32,
	interhub_amount: u32,
}

bitflags::bitflags! {
	pub struct ItemFlags : u8 {
		const NONE = 0;
		const INVBAR = 1 << 0;
		const KEEP_DEPLETED = 1 << 1;
		const AUTO_ACTIVATE = 1 << 2;
		const NO_DROP = 1 << 3;
		const NO_CLEAR = 1 << 4;
		const NO_RESPAWN = 1 << 5;
	}
}
