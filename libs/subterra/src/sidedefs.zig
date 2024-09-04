const std = @import("std");

const root = @import("root.zig");
const Game = root.Game;
const Pos16 = root.Pos16;

/// See <https://doomwiki.org/wiki/Sidedef>.
/// These are cast directly from the bytes of a WAD's lump; it is recommended you
/// use the attached methods rather than accessing fields, since the methods
/// ensure that conversion from little to native endianness is performed.
pub fn SideDef(comptime game: Game) type {
    const Tex = switch (game) {
        .doom64 => u16,
        else => [8]u8,
    };

    return extern struct {
        const Self = @This();

        pub const TexT = Tex;

        pub const TexRet = switch (Tex) {
            [8]u8 => []const u8,
            else => u16,
        };

        _offs_x: i16,
        _offs_y: i16,
        _tex_top: TexT,
        _tex_bottom: TexT,
        _tex_mid: TexT,
        _sector: u16,

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

        pub fn offset(self: *const Self) Pos16 {
            return Pos16{
                .x = std.mem.littleToNative(i16, self._offs_x),
                .y = std.mem.littleToNative(i16, self._offs_y),
            };
        }

        /// To be used as an index into a slice of sectors.
        pub fn sector(self: *const Self) u16 {
            return std.mem.littleToNative(u16, self._sector);
        }

        pub fn textureBottom(self: *const Self) TexRet {
            return switch (TexT) {
                [8]u8 => return std.mem.sliceTo(self._tex_bottom[0..], 0),
                else => return std.mem.littleToNative(u16, self._tex_bottom),
            };
        }

        pub fn textureMid(self: *const Self) TexRet {
            return switch (TexT) {
                [8]u8 => return std.mem.sliceTo(self._tex_mid[0..], 0),
                else => return std.mem.littleToNative(u16, self._tex_mid),
            };
        }

        pub fn textureTop(self: *const Self) TexRet {
            return switch (TexT) {
                [8]u8 => return std.mem.sliceTo(self._tex_top[0..], 0),
                else => return std.mem.littleToNative(u16, self._tex_top),
            };
        }
    };
}

test "SIDEDEFS, fromBytes" {
    const bytes = [_]u8{
        // Side-def 3 (from 0) of Doom 2 MAP01...
        0x20, 0x00, 0x08, 0x00, 0x2D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2D, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x53, 0x57, 0x32, 0x42, 0x52, 0x43, 0x4F, 0x4D, 0x08, 0x00,
        // Side-def 9 (from 0) of Doom 2 MAP01...
        0x00, 0x00, 0x00, 0x00, 0x50, 0x49, 0x50, 0x45, 0x34, 0x00, 0x00, 0x00, 0x50, 0x49, 0x50,
        0x45, 0x34, 0x00, 0x00, 0x00, 0x2D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    };
    const sidedefs = SideDef(.doom).fromBytes(@alignCast(bytes[0..]));

    try std.testing.expectEqual(2, sidedefs.len);

    try std.testing.expectEqual(Pos16{ .x = 32, .y = 8 }, sidedefs[0].offset());
    try std.testing.expectEqual(8, sidedefs[0].sector());
    try std.testing.expectEqualStrings("-", sidedefs[0].textureBottom());
    try std.testing.expectEqualStrings("SW2BRCOM", sidedefs[0].textureMid());
    try std.testing.expectEqualStrings("-", sidedefs[0].textureTop());

    try std.testing.expectEqual(Pos16{ .x = 0, .y = 0 }, sidedefs[1].offset());
    try std.testing.expectEqual(0, sidedefs[1].sector());
    try std.testing.expectEqualStrings("PIPE4", sidedefs[1].textureBottom());
    try std.testing.expectEqualStrings("-", sidedefs[1].textureMid());
    try std.testing.expectEqualStrings("PIPE4", sidedefs[1].textureTop());
}
