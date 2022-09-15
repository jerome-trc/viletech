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

use crate::data::game::AssetIndex;

/// An entity's tie to a state machine, and all supporting data.
#[derive(Debug, shipyard::Component)]
pub struct Actor {
	/// Indexes into [`crate::data::game::Namespace::state_machines`].
	state_machine: AssetIndex,
	/// Indexes into [`crate::game::ActorStateMachine::states`].
	state: usize,
	/// Tics remaining until state index is advanced.
	/// See also [`crate::game::ActorState::INFINITE_DURATION`].
	state_clock: i32,
	/// Movement, state advances are suspended for this many tics.
	/// Script calls to `tick` still go through.
	freeze_tics: u32
}
