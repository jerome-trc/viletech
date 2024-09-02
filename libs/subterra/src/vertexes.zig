const std = @import("std");

const Pos16 = if (@import("builtin").is_test)
    @import("root.zig").Pos16
else
    @import("root").Pos16;

/// See <https://doomwiki.org/wiki/Vertex>.
/// These are cast directly from the bytes of a WAD's lump; it is recommended you
/// use the attached methods rather than accessing fields, since the methods
/// ensure that conversion from little to native endianness is performed.
pub const Vertex = extern struct {
    x: i16,
    y: i16,

    /// Caller guarantees that `bytes.len` is divisible by `@sizeOf(@This())`.
    pub fn fromBytes(bytes: []align(@alignOf(Vertex)) const u8) []const Vertex {
        return std.mem.bytesAsSlice(Vertex, bytes);
    }

    /// Caller guarantees that `bytes.len` is divisible by `@sizeOf(@This())`.
    pub fn fromBytesMut(bytes: []align(@alignOf(Vertex)) u8) []Vertex {
        return std.mem.bytesAsSlice(Vertex, bytes);
    }

    pub fn fromBytesLossy(bytes: []align(@alignOf(Vertex)) const u8) []const Vertex {
        const count = bytes.len / @sizeOf(Vertex);
        return std.mem.bytesAsSlice(Vertex, bytes[0..(count * @sizeOf(Vertex))]);
    }

    pub fn fromBytesLossyMut(bytes: []align(@alignOf(Vertex)) u8) []Vertex {
        const count = bytes.len / @sizeOf(Vertex);
        return std.mem.bytesAsSlice(Vertex, bytes[0..(count * @sizeOf(Vertex))]);
    }

    pub fn bounds(verts: []const Vertex) Bounds {
        var min = Pos16{ .x = 0, .y = 0 };
        var max = Pos16{ .x = 0, .y = 0 };

        for (verts) |vert| {
            if (vert.x < min.x) {
                min.x = vert.x;
            } else if (vert.x > max.x) {
                max.x = vert.x;
            }

            if (vert.y < min.y) {
                min.y = vert.y;
            } else if (vert.y > max.y) {
                max.y = vert.y;
            }
        }

        return Bounds{ .min = min, .max = max };
    }

    pub fn position(self: *const Vertex) Pos16 {
        return Pos16{
            .x = std.mem.littleToNative(i16, self.x),
            .y = std.mem.littleToNative(i16, self.y),
        };
    }
};

/// See [`Vertex.bounds`].
pub const Bounds = struct {
    min: Pos16,
    max: Pos16,
};

test "VERTEXES, fromBytes" {
    const bytes = [8]u8{
        // First vertex of E1M1...
        0x40, 0x04, 0xA0, 0xF1,
        // Second vertex of E1M1...
        0x00, 0x04, 0xA0, 0xF1,
    };
    const vertexes = Vertex.fromBytes(@alignCast(bytes[0..]));

    try std.testing.expectEqual(
        Pos16{ .x = 1088, .y = -3680 },
        vertexes[0].position(),
    );
    try std.testing.expectEqual(
        Pos16{ .x = 1024, .y = -3680 },
        vertexes[1].position(),
    );
}
