-- Library configuration module for the Lua language server
-- (https://github.com/sumneko/lua-language-server).

_G.name = "VileTech Engine API"
_G.words = {'viletech%.%w+'}

_G.configs = {
	{
		key = 'Lua.runtime.version',
		action = 'set',
		value = "LuaJIT"
	}
}

local disabled_builtins = {
	'coroutine',
	'debug',
	'ffi',
	'io',
	'jit',
	'os',
	'package.loaders',
	'table.clear',
	'table.new',
	'utf8',
}

for _, name in ipairs(disabled_builtins) do
	table.insert(configs, {
		key = 'Lua.runtime.builtin',
		action = 'prop',
		prop = name,
		value = 'disable'
	})
end