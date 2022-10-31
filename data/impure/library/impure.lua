--- @meta

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

	--- Does nothing if not running with the developer mode launch arguments.
	--- @param msg string A Rust-like format string (only supports `{}`). Gets prefixed with `[DEBUG]`.
	--- @param ... any Values to format into `msg`.
	debug = function(msg, ...) end,
}

return impure
