//! Symbols which could reasonably be a part of the standard library.

const builtin = @import("builtin");
const std = @import("std");

/// Provides disambiguation.
pub const Path = [:0]const u8;

/// Packs 4 `u8`s into one `u32`. For use in checking magic numbers.
pub fn asciiId(b0: u8, b1: u8, b2: u8, b3: u8) u32 {
    const b0_32: u32 = b0;
    const b1_32: u32 = b1;
    const b2_32: u32 = b2;
    const b3_32: u32 = b3;

    return if (builtin.cpu.arch.endian() == .little)
        (b0_32 | (b1_32 << 8) | (b2_32 << 16) | (b3_32 << 24))
    else
        (b3_32 | (b2_32 << 8) | (b1_32 << 16) | (b0_32 << 24));
}

test "asciiId" {
    const id = asciiId('M', 'T', 'h', 'd');
    try std.testing.expectEqual(1684558925, id);
}

/// Exactly the same as `std.StringHashMap` but for NUL-terminated strings.
pub fn ZStringHashMap(V: type) type {
    return std.HashMap(
        [:0]const u8,
        V,
        struct {
            pub fn eql(self: @This(), a: [:0]const u8, b: [:0]const u8) bool {
                _ = self;
                return std.mem.eql(a, b);
            }

            pub fn hash(self: @This(), s: [:0]const u8) u64 {
                _ = self;
                return std.hash.Wyhash.hash(0, s);
            }
        },
        std.hash_map.default_max_load_percentage,
    );
}
