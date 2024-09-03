const std = @import("std");

pub const Sector = extern struct {
    _height_floor: i16,
    _height_ceil: i16,
    _tex_floor: [8]u8 align(1),
    _tex_ceil: [8]u8 align(1),
    _light_level: u16,
    _special: u16,
    _trigger: u16,

    /// Caller guarantees that `bytes.len` is divisible by `@sizeOf(@This())`.
    pub fn fromBytes(bytes: []align(@alignOf(Sector)) const u8) []const Sector {
        return std.mem.bytesAsSlice(Sector, bytes);
    }

    /// Caller guarantees that `bytes.len` is divisible by `@sizeOf(@This())`.
    pub fn fromBytesMut(bytes: []align(@alignOf(Sector)) u8) []Sector {
        return std.mem.bytesAsSlice(Sector, bytes);
    }

    pub fn fromBytesLossy(bytes: []align(@alignOf(Sector)) const u8) []const Sector {
        const count = bytes.len / @sizeOf(Sector);
        return std.mem.bytesAsSlice(Sector, bytes[0..(count * @sizeOf(Sector))]);
    }

    pub fn fromBytesLossyMut(bytes: []align(@alignOf(Sector)) u8) []Sector {
        const count = bytes.len / @sizeOf(Sector);
        return std.mem.bytesAsSlice(Sector, bytes[0..(count * @sizeOf(Sector))]);
    }

    pub fn heightCeiling(self: *const Sector) i16 {
        return std.mem.littleToNative(i16, self._height_ceil);
    }

    pub fn heightFloor(self: *const Sector) i16 {
        return std.mem.littleToNative(i16, self._height_floor);
    }

    pub fn lightLevel(self: *const Sector) u16 {
        return std.mem.littleToNative(u16, self._light_level);
    }

    pub fn special(self: *const Sector) u16 {
        return std.mem.littleToNative(u16, self._special);
    }

    pub fn textureCeiling(self: *const Sector) []const u8 {
        return std.mem.sliceTo(self._tex_ceil[0..], 0);
    }

    pub fn textureFloor(self: *const Sector) []const u8 {
        return std.mem.sliceTo(self._tex_floor[0..], 0);
    }

    /// a.k.a. the "tag".
    pub fn trigger(self: *const Sector) u16 {
        return std.mem.littleToNative(u16, self._trigger);
    }
};

test "SECTORS, fromBytes" {
    const bytes = [_]u8{
        // First sector of E1M1...
        0xB0, 0xFF, 0xD8, 0x00, 0x4E, 0x55, 0x4B, 0x41, 0x47, 0x45, 0x33, 0x00,
        0x46, 0x5F, 0x53, 0x4B, 0x59, 0x31, 0x00, 0x00, 0xFF, 0x00, 0x07, 0x00,
        0x00, 0x00, 0xC8,
        // Second sector of E1M1...
        0xFF, 0xD8, 0x00, 0x46, 0x4C, 0x4F, 0x4F, 0x52, 0x37,
        0x5F, 0x31, 0x46, 0x5F, 0x53, 0x4B, 0x59, 0x31, 0x00, 0x00, 0xFF, 0x00,
        0x00, 0x00, 0x00, 0x00,
    };
    const sectors = Sector.fromBytes(@alignCast(bytes[0..]));

    try std.testing.expectEqual(2, sectors.len);

    try std.testing.expectEqual(-80, sectors[0].heightFloor());
    try std.testing.expectEqual(216, sectors[0].heightCeiling());
    try std.testing.expectEqualStrings("NUKAGE3", sectors[0].textureFloor());
    try std.testing.expectEqualStrings("F_SKY1", sectors[0].textureCeiling());
    try std.testing.expectEqual(255, sectors[0].lightLevel());
    try std.testing.expectEqual(7, sectors[0].special());
    try std.testing.expectEqual(0, sectors[0].trigger());

    try std.testing.expectEqual(-56, sectors[1].heightFloor());
    try std.testing.expectEqual(216, sectors[1].heightCeiling());
    try std.testing.expectEqualStrings("FLOOR7_1", sectors[1].textureFloor());
    try std.testing.expectEqualStrings("F_SKY1", sectors[1].textureCeiling());
    try std.testing.expectEqual(255, sectors[1].lightLevel());
    try std.testing.expectEqual(0, sectors[1].special());
    try std.testing.expectEqual(0, sectors[1].trigger());
}
