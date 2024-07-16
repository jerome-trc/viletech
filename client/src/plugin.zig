//! Prototypes for functions that can be exposed by plugins.

const Core = @import("Core.zig");

pub const OnGameStart = *const fn (*Core) void;
pub const OnGameClose = *const fn (*Core) void;
