pub const gfx = @import("gfx.zig");
pub const level = @import("level.zig");
pub const mus = @import("mus.zig");

pub const Game = enum {
    boom, // Not a game per se...
    doom,
    doom64,
    doom64ex,
    doom64_2020,
    heretic,
    hexen,
    mbf, // Also not a game...
    strife,
};

test {
    const std = @import("std");
    std.testing.refAllDecls(gfx);
    std.testing.refAllDecls(level);
}
