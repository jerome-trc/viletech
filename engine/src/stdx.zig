//! Symbols which could reasonably be a part of the standard library.

const builtin = @import("builtin");
const std = @import("std");

pub const b_per_kb = 1024;
pub const b_per_mb = b_per_kb * 1024;
pub const b_per_gb = b_per_mb * 1024;

/// Provides disambiguation.
pub const Path = []const u8;
/// Provides disambiguation.
pub const PathZ = [:0]const u8;

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

/// Hours, minutes, and seconds.
pub const HhMmSs = struct {
    hours: u64,
    minutes: u64,
    seconds: u64,

    pub fn fromNs(nanos: u64) @This() {
        const microsecs = nanos / std.time.ns_per_us;
        const millisecs = microsecs / std.time.us_per_ms;
        var secs = millisecs / std.time.ms_per_s;
        var mins = secs / std.time.s_per_min;
        const hours = mins / 60;

        secs -= (mins * 60);
        mins -= (hours * 60);

        return .{ .hours = hours, .minutes = mins, .seconds = secs };
    }
};

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
