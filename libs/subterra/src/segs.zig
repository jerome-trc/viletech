const std = @import("std");

/// See <https://doomwiki.org/wiki/Seg>.
/// These are cast directly from the bytes of a WAD's lump; it is recommended you
/// use the attached methods rather than accessing fields, since the methods
/// ensure that conversion from little to native endianness is performed.
pub const Seg = extern struct {
    pub const Direction = enum {
        /// This seg runs along the left of a linedef.
        back,
        /// This seg runs along the right of a linedef.
        front,
    };

    _v_start: u16,
    _v_end: u16,
    _angle: i16,
    _linedef: u16,
    _direction: i16,
    _offset: i16,

    /// Caller guarantees that `bytes.len` is divisible by `@sizeOf(@This())`.
    pub fn fromBytes(bytes: []align(@alignOf(Seg)) const u8) []const Seg {
        return std.mem.bytesAsSlice(Seg, bytes);
    }

    /// Caller guarantees that `bytes.len` is divisible by `@sizeOf(@This())`.
    pub fn fromBytesMut(bytes: []align(@alignOf(Seg)) u8) []Seg {
        return std.mem.bytesAsSlice(Seg, bytes);
    }

    pub fn fromBytesLossy(bytes: []align(@alignOf(Seg)) const u8) []const Seg {
        const count = bytes.len / @sizeOf(Seg);
        return std.mem.bytesAsSlice(Seg, bytes[0..(count * @sizeOf(Seg))]);
    }

    pub fn fromBytesLossyMut(bytes: []align(@alignOf(Seg)) u8) []Seg {
        const count = bytes.len / @sizeOf(Seg);
        return std.mem.bytesAsSlice(Seg, bytes[0..(count * @sizeOf(Seg))]);
    }

    /// Returns a binary angle measurement (or "BAMS",
    /// see <https://en.wikipedia.org/wiki/Binary_angular_measurement>).
    pub fn angle(self: *const Seg) i16 {
        return std.mem.littleToNative(i16, self._angle);
    }

    pub fn direction(self: *const Seg) Direction {
        return switch (std.mem.littleToNative(i16, self._direction)) {
            0 => .front,
            else => .back,
        };
    }

    /// To be used as an index into a slice of linedefs.
    pub fn linedef(self: *const Seg) u16 {
        return std.mem.littleToNative(u16, self._linedef);
    }

    pub fn offset(self: *const Seg) i16 {
        return std.mem.littleToNative(i16, self._offset);
    }

    /// To be used as an index into a slice of level vertices.
    pub fn vertexStart(self: *const Seg) u16 {
        return std.mem.littleToNative(u16, self._v_start);
    }

    /// To be used as an index into a slice of level vertices.
    pub fn vertexEnd(self: *const Seg) u16 {
        return std.mem.littleToNative(u16, self._v_end);
    }
};

/// See [`SegGl`].
pub const SegGlVersion = enum {
    v1,
    v3,
    v5,
};

/// See <https://glbsp.sourceforge.net/specs.php>.
pub fn SegGl(comptime version: SegGlVersion) type {
    return extern struct {
        const Self = @This();

        pub const Index = switch (version) {
            .v1 => u16,
            .v3, .v5 => u32,
        };

        _v_start: Index,
        _v_end: Index,
        _linedef: u16,
        _side: u16,
        _partner_seg: Index,

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

        /// To be used as an index into a slice of linedefs.
        pub fn linedef(self: *const Self) u16 {
            return std.mem.littleToNative(u16, self._linedef);
        }

        pub fn partnerSeg(self: *const Self) Index {
            return std.mem.littleToNative(Index, self._partner_seg);
        }

        pub fn side(self: *const Self) u16 {
            return std.mem.littleToNative(u16, self._side);
        }

        pub fn vertexStart(self: *const Self) Index {
            return std.mem.littleToNative(Index, self._v_start);
        }

        pub fn vertexStartIsGl(self: *const Self) Index {
            const v = self.vertexStart();

            return switch (version) {
                .v1 => v & (1 << 15) != 0,
                .v3 => v & (1 << 30) != 0,
                .v5 => v & (1 << 31) != 0,
            };
        }

        pub fn vertexEnd(self: *const Self) Index {
            return std.mem.littleToNative(Index, self._v_end);
        }

        pub fn vertexEndIsGl(self: *const Self) bool {
            const v = self.vertexEnd();

            return switch (version) {
                .v1 => v & (1 << 15) != 0,
                .v3 => v & (1 << 30) != 0,
                .v5 => v & (1 << 31) != 0,
            };
        }
    };
}

test "SEGS, semantic check" {
    const vanilla: Seg = undefined;
    const gl_v1: SegGl(.v1) = undefined;
    const gl_v3: SegGl(.v3) = undefined;
    const gl_v5: SegGl(.v5) = undefined;
    _ = .{ vanilla, gl_v1, gl_v3, gl_v5 };
}

test "SEGS, fromBytes" {
    const bytes = [24]u8{
        // First seg of TNT MAP01...
        0xC1, 0x00, 0xD3, 0x01, 0x00, 0xA0, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00,
        // Second seg of TNT MAP01...
        0xD4, 0x01, 0xD7, 0x00, 0x00, 0x40, 0x1B, 0x01, 0x00, 0x00, 0x20, 0x00,
    };
    const segs = Seg.fromBytes(@alignCast(bytes[0..]));

    try std.testing.expectEqual(segs.len, 2);

    try std.testing.expectEqual(193, segs[0].vertexStart());
    try std.testing.expectEqual(467, segs[0].vertexEnd());
    try std.testing.expectEqual(-24576, segs[0].angle());
    try std.testing.expectEqual(258, segs[0].linedef());
    try std.testing.expectEqual(.front, segs[0].direction());
    try std.testing.expectEqual(0, segs[0].offset());

    try std.testing.expectEqual(468, segs[1].vertexStart());
    try std.testing.expectEqual(215, segs[1].vertexEnd());
    try std.testing.expectEqual(16384, segs[1].angle());
    try std.testing.expectEqual(283, segs[1].linedef());
    try std.testing.expectEqual(.front, segs[1].direction());
    try std.testing.expectEqual(32, segs[1].offset());
}
