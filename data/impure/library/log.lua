--- @meta

--- @class loglib

log = {
	--- The given message will be prefixed with `[INFO]` and a timestamp.
	--- @param msg string A Rust-like format string (only supports `{}`).
	--- @param ... any Values to format into `msg`.
	info = function(msg, ...) end,

	--- The given message will be prefixed with `[WARN]` and a timestamp.
	--- @param msg string A Rust-like format string (only supports `{}`).
	--- @param ... any Values to format into `msg`.
	warn = function(msg, ...) end,

	--- The given message will be prefixed with `[ERROR]` and a timestamp.
	--- @param msg string A Rust-like format string (only supports `{}`).
	--- @param ... any Values to format into `msg`.
	err = function(msg, ...) end,

	--- The given message will be prefixed with `[DEBUG]` and a timestamp.
	--- Does nothing if not running with the developer mode launch arguments.
	--- @param msg string A Rust-like format string (only supports `{}`).
	--- @param ... any Values to format into `msg`.
	debug = function(msg, ...) end,
}

return log
