-- Helpers for arrays (tables consisting only of sequential values).
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

--- @class arraylib
local array = {}

-- Non-mutating ----------------------------------------------------------------

--- @generic T
--- @param arr T[]
--- @param elem T
--- @return boolean
--- @nodiscard
function array.contains(arr, elem)
	for _, v in ipairs(arr) do
		if v == elem then return true end
	end

	return false
end

--- Returns `nil` if `elem` isn't found in this array.
--- Otherwise, returns the element's index.
--- @generic T
--- @param arr T[]
--- @param elem T
--- @return nil|integer
--- @nodiscard
function array.find(arr, elem)
	for i, v in ipairs(arr) do
		if v == elem then
			return i
		end
	end

	return nil
end

--- Returns the first element which, when passed to `predicate`,
--- outputs a return value of `true`.
--- @generic T
--- @param arr T[]
--- @param predicate fun(T): boolean
--- @return nil|integer
--- @nodiscard
function array.find_ex(arr, predicate)
	for _, v in ipairs(arr) do
		if predicate(v) then
			return v
		end
	end

	return nil
end

--- Shortcut for `#arr < 1`.
--- @generic T
--- @param arr T[]
--- @return boolean
--- @nodiscard
function array.is_empty(arr)
	return #arr < 1
end

--- Returns an iterator function that yields the results of calling
--- `callback` with the argument `arr[i]` for every element.
--- @generic T
--- @generic O
--- @param arr T[]
--- @param callback fun(T): O
--- @return fun(): O
--- @nodiscard
function array.map(arr, callback)
	local i = 0
	local n = #arr

	return function()
		i = i + 1

		if i <= n then
			return callback(arr[i])
		end

		return nil
	end
end

--- Returns an iterator function that yields each element from `arr`
--- which, when passed to `predicate`, outputs `true`.
--- Only returns `nil` when at the end of the array.
--- @generic T
--- @param arr T[]
--- @param predicate fun(T): boolean
--- @return fun(): T
--- @nodiscard
function array.where(arr, predicate)
	local i = 0
	local n = #arr

	return function()
		while not predicate(arr[i]) do
			i = i + 1

			if i <= n then
				return arr[i]
			else
				break
			end
		end

		return nil
	end
end

-- Mutating --------------------------------------------------------------------

--- Sets every element in this array to `nil`, leaving it empty.
--- @generic T
--- @param arr T[]
function array.clear(arr)
	for i, _ in ipairs(arr) do
		arr[i] = nil
	end
end

--- Remove range [index, index + count) of elements from the array.
--- If no `count` argument is given, it defaults to 1.
--- @generic T
--- @param arr T[]
--- @param index integer
--- @param count integer
function array.delete(arr, index, count)
	local i = count or 1

	while i > 0 do
		arr[index] = nil
		i = i - 1
	end
end

--- Adds `count` new default elements of type `T` to the end of the array.
--- @generic T
--- @param arr T[]
--- @param count integer
function array.grow(arr, count)
	for _ = 0, count do
		local e
		array.push(arr, e)
	end
end

--- Wraps https://www.lua.org/manual/5.1/manual.html#pdf-table.insert.
--- @generic T
--- @param arr T[]
--- @param index integer
--- @param elem T
function array.insert(arr, index, elem)
	table.insert(arr, index, elem)
end

--- Removes the last element from the array.
--- @generic T
--- @param arr T[]
--- @return boolean anything_popped `true` if there was anything to pop.
function array.pop(arr)
	local ret = #arr > 0
	if ret then arr[#arr] = nil end
	return ret
end

--- @generic T
--- @param arr T[]
--- @return integer end The index of the newly-pushed element.
function array.push(arr, elem)
	arr[#arr] = elem
	return #arr - 1
end

--- @generic T
--- @param arr T[]
--- @param ... T
--- @return integer last The index of the last newly-pushed element.
function array.push_multi(arr, ...)
	for i = 1, select('#', ...) do
		array.push(arr, select(i, ...))
	end

	return #arr - 1
end

--- Note that elements removed by shrinkage are lost.
--- @generic T
--- @param arr T[]
function array.resize(arr, new_size)
	while #arr > new_size do
		array.pop(arr)
	end

	while #arr < new_size do
		local t
		array.push(arr, t)
	end
end

-- Miscellaneous ---------------------------------------------------------------

--- Pushes a shallow copy of every element from `other` onto `arr`.
--- @generic T
--- @param arr T[]
--- @param other T[]
function array.append(arr, other)
	for _, v in ipairs(other) do
		array.push(arr, v)
	end
end

--- @generic T
--- @param arr T[]
--- @return T[]
function array.copy_shallow(arr)
	local ret = {}

	for i, v in ipairs(arr) do
		ret[i] = v
	end

	return ret
end

--- Be aware that `a2[i]` will clobber `a1[i]`.
--- Does not recur if `T` is a table-based type.
--- @generic T
--- @param a1 T[]
--- @param a2 T[]
--- @return T[] new_table
function array.merge(a1, a2)
	local ret = {}

	for i, v in ipairs(a1) do
		ret[i] = v
	end

	for i, v in ipairs(a2) do
		ret[i] = v
	end

	return ret
end

--- For every index in `from`, `to[i]` becomes `from[i]` and `from[i]` becomes nil.
--- @generic T
--- @param from T[]
--- @param to T[]
function array.move(from, to)
	for i, v in ipairs(from) do
		to[i] = v
		from[i] = nil
	end
end

return array
