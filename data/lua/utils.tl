-- General/miscellaneous helpers.
-- Pre-exported globally in all Lua contexts.

local serpent = import('/impure/lua/serpent.lua')

global function repr(obj: any): string
	return serpent.block(obj, {
		indent = '\t',
		maxlevel = 8,
		nocode = true,
		numformat = '%.6g',
	})
end
