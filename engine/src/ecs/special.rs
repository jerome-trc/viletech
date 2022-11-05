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

/// Primarily for use by ACS behaviours. An entity won't have this component
/// unless the map or blueprint specifies one of the fields within.
#[derive(Debug, shipyard::Component)]
pub struct SpecialVars {
	tid: i32,
	special: i32,
	special_i: [i32; 2],
	special_f: [f64; 2],
	args: [i32; 5],
}
