-- General/miscellaneous helpers.
-- Pre-exported globally in all Lua contexts.

--[[

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
along with this program. If not, see <http://www.gnu.org/licenses/>.

]]

local serpent = require('viletech.lua.serpent')

local utils = {}

--- Wraps `serpent.block`, adding indentation, an 8-level recursion limit,
--- eliminating code output, and formatting numbers to six decimal places.
--- Use whenever you just want a to-string of some Lua thing.
--- @param obj any
--- @return string
--- @nodiscard
utils.repr = function(obj)
	return serpent.block(obj, {
		indent = '\t',
		maxlevel = 8,
		nocode = true,
		numformat = '%.6g'
	})
end

return utils
