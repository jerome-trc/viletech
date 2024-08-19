//! # WADLoad
//!
//! Simple, pay-only-for-what-you-use I/O for archive formats from old video games.a

pub const wad = @import("wad.zig");

test {
    @import("std").testing.refAllDecls(wad);
}
