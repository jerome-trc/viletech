-- Array utilities.
-- Pre-exported globally in all Lua contexts.

local record array
	contains: function<T>(arr: {T}, elem: T): boolean
	-- Returns `nil` if `elem` isn't found in this array.
	-- Otherwise, returns the element's index.
	find: function<T>(arr: {T}, elem: T): integer
	-- Returns the first element which, when passed to `comparator`,
	-- outputs a return value of `true`.
	find_ex: function<T>(arr: {T}, comparator: function(T): boolean): T
	is_empty: function<T>(arr: {T}): boolean
	-- Returns an iterator function that yields the results of calling
	-- `callback` with the argument `arr[i]` for every element.
	map: function<T>(arr: {T}, callback: function(T): T): (function(): T)
	-- Returns an iterator function that yields each element from `arr`
	-- which, when passed to `predicate`, outputs `true`.
	-- Only returns `nil` when at the end of the array.
	where: function<T>(arr: {T}, predicate: function(T): boolean): (function(): T)

	-- Sets every element in this array to `nil`, leaving it empty.
	clear: function<T>(arr: {T})
	-- Remove range [index, index + count) of elements from the array.
	-- If no `count` argument is given, it defaults to 1.
	delete: function<T>(arr: {T}, index: integer, count: integer)
	-- Adds `count` new default elements of type T to the end of the array.
	grow: function<T>(arr: {T}, count: integer)
	-- Any elements after `index` are moved towards the end of the array.
	insert: function<T>(arr: {T}, index: integer, elem: T)
	-- Removes the last element from the array.
	-- Returns true if there was anything to pop.
	pop: function<T>(arr: {T}): T
	-- Returns the index of the newly-pushed element.
	push: function<T>(arr: {T}, elem: T): integer
	-- Returns the index of the last newly-pushed element.
	push_multi: function<T>(arr: {T}, ...: T): integer
	-- Note that elements removed by shrinkage are lost.
	resize: function<T>(arr: {T}, new_size: integer)

	append: function<T>(arr: {T}, other: {T})
	copy_shallow: function<T>(arr: {T}): {T}
	-- `a2[i]` will clobber `a1[i]`. Does not recur if `T` is a table type.
	merge: function<T>(a1: {T}, a2: {T}): {T}
	move: function<T>(from: {T}, to: {T})
end

-- Accessors -------------------------------------------------------------------

function array.contains<T>(arr: {T}, elem: T): boolean
	for _, v in ipairs(arr) do
		if v == elem then return true end
	end

	return false
end

function array.find<T>(arr: {T}, elem: T): integer
	for i, v in ipairs(arr) do
		if v == elem then
			return i
		end
	end

	return nil
end

function array.find_ex<T>(arr: {T}, comparator: function(T): boolean): T
	for _, v in ipairs(arr) do
		if comparator(v) then
			return v
		end
	end

	return nil
end

function array.is_empty<T>(arr: {T}): boolean
	return #arr < 1
end

function array.map<T>(arr: {T}, callback: function(T): T): (function(): T)
	local i = 0
	local n = #arr

	return function(): T
		i = i + 1

		if i <= n then
			return callback(arr[i])
		end

		return nil
	end
end

function array.where<T>(arr: {T}, predicate: function(T): boolean): (function(): T)
	local i = 0
	local n = #arr

	return function(): T
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

-- Mutators --------------------------------------------------------------------

function array.clear<T>(arr: {T})
	for i, _ in ipairs(arr) do
		arr[i] = nil
	end
end

function array.delete<T>(arr: {T}, index: integer, count: integer)
	local i = count or 1

	while i > 0 do
		arr[index] = nil
		i = i - 1
	end
end

function array.grow<T>(arr: {T}, count: integer)
	for _ = 0, count do
		local e: T
		array.push(arr, e)
	end
end

function array.insert<T>(arr: {T}, index: integer, elem: T)
	table.insert(arr, index, elem)
end

function array.pop<T>(arr: {T}): boolean
	local ret = #arr > 0
	if ret then arr[#arr] = nil end
	return ret
end

function array.push<T>(arr: {T}, elem: T): integer
	arr[#arr] = elem
	return #arr - 1
end

function array.push_multi<T>(arr: {T}, ...: T): integer
	for i = 1, select('#', ...) do
		array.push(arr, select(i, ...))
	end

	return #arr - 1
end

function array.resize<T>(arr: {T}, new_size: integer)
	while #arr > new_size do
		array.pop(arr)
	end

	while #arr < new_size do
		local t: T
		array.push(arr, t)
	end
end

-- Miscellaneous ---------------------------------------------------------------

function array.append<T>(arr: {T}, other: {T})
	for _, v in ipairs(other) do
		array.push(arr, v)
	end
end

function array.copy_shallow<T>(arr: {T}): {T}
	local ret: {T} = {}

	for i, v in ipairs(arr) do
		ret[i] = v
	end

	return ret
end

function array.merge<T>(a1: {T}, a2: {T}): {T}
	local ret = {}

	for i, v in ipairs(a1) do
		ret[i] = v
	end

	for i, v in ipairs(a2) do
		ret[i] = v
	end

	return ret
end

function array.move<T>(from: {T}, to: {T})
	for i, v in ipairs(from) do
		to[i] = v
		from[i] = nil
	end
end

return array
