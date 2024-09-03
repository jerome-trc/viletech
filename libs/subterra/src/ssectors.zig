const std = @import("std");

/// See <https://doomwiki.org/wiki/Subsector>.
/// These are cast directly from the bytes of a WAD's lump; it is recommended you
/// use the attached methods rather than accessing fields, since the methods
/// ensure that conversion from little to native endianness is performed.
pub fn Subsector(IntT: type) type {
    switch (IntT) {
        u16, u32 => {},
        else => @compileError("only `u16` (vanilla) and `u32` (GL) are valid"),
    }

    return extern struct {
        const Self = @This();

        pub const Int = IntT;

        _seg_count: u16,
        _seg: u16,

        /// Caller guarantees that `bytes.len` is divisible by `@sizeOf(@This())`.
        pub fn fromBytes(bytes: []align(@alignOf(Self)) const u8) []const Self {
            return std.mem.bytesAsSlice(Self, bytes);
        }

        /// Caller guarantees that `bytes.len` is divisible by `@sizeOf(@This())`.
        pub fn fromBytesMut(bytes: []align(@alignOf(Self)) u8) []Self {
            return std.mem.bytesAsSlice(Self, bytes);
        }

        pub fn fromBytesLossy(bytes: []align(@alignOf(Self)) const u8) []const Self {
            const count = bytes.len / @sizeOf(Self);
            return std.mem.bytesAsSlice(Self, bytes[0..(count * @sizeOf(Self))]);
        }

        pub fn fromBytesLossyMut(bytes: []align(@alignOf(Self)) u8) []Self {
            const count = bytes.len / @sizeOf(Self);
            return std.mem.bytesAsSlice(Self, bytes[0..(count * @sizeOf(Self))]);
        }

        pub fn segCount(self: *const Self) u16 {
            return std.mem.littleToNative(u16, self._seg_count);
        }

        /// To be used as an index into a slice of segs.
        pub fn firstSeg(self: *const Self) u16 {
            return std.mem.littleToNative(u16, self._seg);
        }
    };
}

test "SSECTORS, fromBytes" {
    const bytes = [8]u8{
        // First subsector of Plutonia MAP01...
        0x08, 0x00, 0x00, 0x00,
        // Second subsector of Plutonia MAP02...
        0x05, 0x00, 0x08, 0x00,
    };
    const ssectors = Subsector(u16).fromBytes(@alignCast(bytes[0..]));

    try std.testing.expectEqual(ssectors.len, 2);

    try std.testing.expectEqual(8, ssectors[0].segCount());
    try std.testing.expectEqual(0, ssectors[0].firstSeg());
    try std.testing.expectEqual(5, ssectors[1].segCount());
    try std.testing.expectEqual(8, ssectors[1].firstSeg());
}
