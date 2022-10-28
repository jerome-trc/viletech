-- Helpers for maps (tables only consisting of string-key/value pairs).
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
along with this program.  If not, see <http://www.gnu.org/licenses/>.

]]

--- @class map
local map = {}

-- Non-mutating ----------------------------------------------------------------

--- @generic T
--- @param m { [string]: T }
--- @return integer size The number of non-sequential pairs in this map.
function map.size(m)
	local ret = 0

	for _, _ in pairs(m) do
		ret = ret + 1
	end

	return ret
end

--- @generic T
--- @param m { [string]: T }
--- @param val T
--- @return boolean
function map.contains_val(m, val)
	for _, v in pairs(m) do
		if v == val then
			return true
		end
	end

	return false
end

-- Mutating --------------------------------------------------------------------

--- Sets every element in this map to `nil`, leaving it empty.
--- @generic T
--- @param m { [string]: T }
function map.clear(m)
	for k, _ in pairs(m) do
		m[k] = nil
	end
end

-- Miscellaneous ---------------------------------------------------------------

--- @generic T
--- @param m { [string]: T } Left untouched by this function.
--- @return T[]
function map.to_array(m)
	local ret = {}

	for _, v in pairs(m) do
		table.insert(ret, v)
	end

	return ret
end

return map
