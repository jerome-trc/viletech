//! Parsers for all manner of languages in the id Tech 1-descendant ecosystem.

pub const doomtools = @import("doomtools.zig");

test {
    @import("std").testing.refAllDeclsRecursive(@This());
}
