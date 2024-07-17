//! Prototypes for functions that can be exposed by plugins.

const Core = @import("Core.zig");

pub const OnGameStart = *const fn (*Core) callconv(.C) void;
pub const OnGameClose = *const fn (*Core) callconv(.C) void;
