--- @meta

--- Note that these functions are always available, but do nothing if the engine
--- isn't being run in developer mode (`-d` or `--dev`).
debug = {
	--- Returns the number of bytes currently used by the Lua state.
	--- @return integer
	--- @nodiscard
	mem = function() end,
}
