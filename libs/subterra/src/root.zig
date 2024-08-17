pub const level = @import("level.zig");
pub const mus = @import("mus.zig");

pub const Game = enum {
    boom, // Not a game per se...
    doom,
    doom64,
    heretic,
    hexen,
    strife,
};

test {
    @import("std").testing.refAllDecls(level);
}
