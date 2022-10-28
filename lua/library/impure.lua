--- @meta

--- @param path string The virtual file system path to a Lua file.
--- @return any module If the Lua module compiles successfully and returns anything, that will be returned. Otherwise, returns `nil`.
--- @nodiscard
function import(path) end

--- @class impure
impure = {
	--- The engine's own version.
	--- @return number major
	--- @return number minor
	--- @return number revision
	--- @nodiscard
	version = function() end,

	--- @param msg string A Rust-like format string (only supports `{}`). Gets prefixed with `[INFO]`.
	--- @param ... any Values to format into `msg`.
	log = function(msg, ...) end,

	--- @param msg string A Rust-like format string (only supports `{}`). Gets prefixed with `[WARN]`.
	--- @param ... any Values to format into `msg`.
	warn = function(msg, ...) end,

	--- @param msg string A Rust-like format string (only supports `{}`). Gets prefixed with `[ERROR]`.
	--- @param ... any Values to format into `msg`.
	err = function(msg, ...) end,

	--- @param msg string A Rust-like format string (only supports `{}`). Gets prefixed with `[DEBUG]`.
	--- @param ... any Values to format into `msg`.
	debug = function(msg, ...) end,
}

return impure
