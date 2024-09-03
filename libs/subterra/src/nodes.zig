const std = @import("std");

const Pos16 = @import("root.zig").Pos16;

/// See <https://doomwiki.org/wiki/Node>.
/// These are cast directly from the bytes of a WAD's lump; it is recommended you
/// use the attached methods rather than accessing fields, since the methods
/// ensure that conversion from little to native endianness is performed.
pub const Node = extern struct {
    /// An axis-aligned bounding box.
    pub const Aabb = extern struct {
        top: i16,
        bottom: i16,
        left: i16,
        right: i16,
    };

    pub const Child = union(enum) {
        subsector: usize,
        subnode: usize,
    };

    _x: i16,
    _y: i16,
    _delta_x: i16,
    _delta_y: i16,
    _aabb_r: Aabb,
    _aabb_l: Aabb,
    _child_r: i16,
    _child_l: i16,

    pub fn segStart(self: *const Node) Pos16 {
        return Pos16{
            .x = std.mem.littleToNative(i16, self._x),
            .y = std.mem.littleToNative(i16, self._y),
        };
    }

    pub fn segDelta(self: *const Node) Pos16 {
        return Pos16{
            .x = std.mem.littleToNative(i16, self._delta_x),
            .y = std.mem.littleToNative(i16, self._delta_y),
        };
    }

    pub fn segEnd(self: *const Node) Pos16 {
        return Pos16{
            .x = std.mem.littleToNative(i16, self._x) + std.mem.littleToNative(i16, self._delta_x),
            .y = std.mem.littleToNative(i16, self._y) + std.mem.littleToNative(i16, self._delta_y),
        };
    }

    pub fn aabbL(self: *const Node) Aabb {
        return Aabb{
            .top = std.mem.littleToNative(i16, self._aabb_l.top),
            .bottom = std.mem.littleToNative(i16, self._aabb_l.bottom),
            .left = std.mem.littleToNative(i16, self._aabb_l.left),
            .right = std.mem.littleToNative(i16, self._aabb_l.right),
        };
    }

    pub fn aabbR(self: *const Node) Aabb {
        return Aabb{
            .top = std.mem.littleToNative(i16, self._aabb_r.top),
            .bottom = std.mem.littleToNative(i16, self._aabb_r.bottom),
            .left = std.mem.littleToNative(i16, self._aabb_r.left),
            .right = std.mem.littleToNative(i16, self._aabb_r.right),
        };
    }

    pub fn childLeft(self: *const Node) Child {
        const child = std.mem.littleToNative(i16, self._child_l);

        return if (child < 0)
            Child{ .subsector = @intCast(child & 0x7FFF) }
        else
            Child{ .subnode = @intCast(child) };
    }

    pub fn childRight(self: *const Node) Child {
        const child = std.mem.littleToNative(i16, self._child_r);

        return if (child < 0)
            Child{ .subsector = @intCast(child & 0x7FFF) }
        else
            Child{ .subnode = @intCast(child) };
    }

    /// Caller guarantees that `bytes.len` is divisible by `@sizeOf(@This())`.
    pub fn fromBytes(bytes: []align(@alignOf(Node)) const u8) []const Node {
        return std.mem.bytesAsSlice(Node, bytes);
    }

    /// Caller guarantees that `bytes.len` is divisible by `@sizeOf(@This())`.
    pub fn fromBytesMut(bytes: []align(@alignOf(Node)) u8) []Node {
        return std.mem.bytesAsSlice(Node, bytes);
    }

    pub fn fromBytesLossy(bytes: []align(@alignOf(Node)) const u8) []const Node {
        const count = bytes.len / @sizeOf(Node);
        return std.mem.bytesAsSlice(Node, bytes[0..(count * @sizeOf(Node))]);
    }

    pub fn fromBytesLossyMut(bytes: []align(@alignOf(Node)) u8) []Node {
        const count = bytes.len / @sizeOf(Node);
        return std.mem.bytesAsSlice(Node, bytes[0..(count * @sizeOf(Node))]);
    }
};

test "NODES, fromBytes" {
    const bytes = [_]u8{
        // First node of The Gantlet...
        0x00, 0x05, 0x58, 0x06, 0x00, 0xFF, 0x00,
        0x00, 0xB8, 0x06, 0x58, 0x06, 0x00, 0x04,
        0x00, 0x05, 0x58, 0x06, 0xB8, 0x05, 0x00,
        0x04, 0x00, 0x05, 0x00, 0x80, 0x01, 0x80,
        // Third node of The Gantlet...
        0x10, 0x05, 0x38, 0x06, 0x00, 0x00, 0x80,
        0xFF, 0x38, 0x06, 0xB8, 0x05, 0x00, 0x05,
        0x10, 0x05, 0xB8, 0x06, 0xF8, 0x05, 0x40,
        0x05, 0xC0, 0x05, 0x03, 0x80, 0x01, 0x00,
    };
    const nodes = Node.fromBytes(@alignCast(bytes[0..]));

    try std.testing.expectEqual(2, nodes.len);

    try std.testing.expectEqual(1280, nodes[0].segStart().x);
    try std.testing.expectEqual(1624, nodes[0].segStart().y);
    try std.testing.expectEqual(-256, nodes[0].segDelta().x);
    try std.testing.expectEqual(0, nodes[0].segDelta().y);

    try std.testing.expectEqual(
        Node.Aabb{
            .top = 1720,
            .bottom = 1624,
            .left = 1024,
            .right = 1280,
        },
        nodes[0].aabbR(),
    );
    try std.testing.expectEqual(
        Node.Aabb{
            .top = 1624,
            .bottom = 1464,
            .left = 1024,
            .right = 1280,
        },
        nodes[0].aabbL(),
    );

    try std.testing.expectEqual(Node.Child{ .subsector = 1 }, nodes[0].childLeft());
    try std.testing.expectEqual(Node.Child{ .subsector = 0 }, nodes[0].childRight());

    try std.testing.expectEqual(Node.Child{ .subnode = 1 }, nodes[1].childLeft());
    try std.testing.expectEqual(Node.Child{ .subsector = 3 }, nodes[1].childRight());
}
