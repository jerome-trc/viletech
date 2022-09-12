-- Map utilities.

local record map
	size: function<K, V>(m: {K:V}): integer
	contains_key: function<K, V>(m: {K:V}, key: K): boolean
	contains_val: function<K, V>(m: {K:V}, val: V): boolean

	-- Sets every element in this map to `nil`, leaving it empty.
	clear: function<K, V>(m: {K:V})

	-- Copies every value 
	to_array: function<K, V>(m: {K:V}): {V}
end

-- Accessors -------------------------------------------------------------------

function map.size<K, V>(m: {K:V}): integer
	local ret = 0

	for _, _ in pairs(m) do
		ret = ret + 1
	end

	return ret
end

function map.contains_key<K, V>(m: {K:V}, key: K): boolean
	return m[key] ~= nil
end

function map.contains_val<K, V>(m: {K:V}, val: V): boolean
	for _, v in pairs(m) do
		if v == val then
			return true
		end
	end

	return false
end

-- Mutators --------------------------------------------------------------------

function map.clear<K, V>(m: {K:V})
	for k, _ in pairs(m) do
		m[k] = nil
	end
end

-- Miscellaneous ---------------------------------------------------------------

function map.to_array<K, V>(m: {K:V}): {V}
	local ret = {}

	for _, v in pairs(m) do
		table.insert(ret, v)
	end

	return ret
end

return map
