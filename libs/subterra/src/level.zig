//! Code used for reading, storing, manipulating, and writing Doom levels.

pub const std = @import("std");

pub const Game = if (@import("builtin").is_test)
    @import("root.zig").Game
else
    @import("root").Game;

pub const EditorNum = u16;

/// Certain important ["editor numbers"](https://zdoom.org/wiki/Editor_number).
pub const ednums = struct {
    pub const hexen_anchor: EditorNum = 3000;
    pub const hexen_spawn: EditorNum = 3001;
    pub const hexen_spawncrush: EditorNum = 3002;

    pub const doom_anchor: EditorNum = 9300;
    pub const doom_spawn: EditorNum = 9301;
    pub const doom_spawncrush: EditorNum = 9302;
    pub const doom_spawnhurt: EditorNum = 9303;
};

pub const Position = struct { x: i16, y: i16 };

// LINEDEFS ////////////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Linedef>.
/// These are cast directly from the bytes of a WAD's lump; it is recommended you
/// use the attached methods rather than accessing fields, since the methods
/// ensure that conversion from little to native endianness is performed.
pub fn LineDef(game: Game) type {
    const FlagInt = switch (game) {
        .doom64 => u32,
        else => u16,
    };

    const FlagPad = switch (game) {
        .boom => u6,
        .doom, .heretic => u7,
        .doom64 => u23,
        .hexen => void,
        .strife => u3,
    };

    const FlagT = packed struct(FlagInt) {
        /// Line blocks things (i.e. player, missiles, and monsters).
        impassible: bool,
        /// Line blocks monsters.
        block_monsters: bool,
        /// Line's two sides can have the "transparent texture".
        two_sided: bool,
        /// Upper texture is pasted onto wall from the top down instead of bottom-up.
        unpegged_upper: bool,
        /// Lower and middle textures are drawn from the bottom up instead of top-down.
        unpegged_lower: bool,
        /// If set, drawn as 1S on the map.
        secret: bool,
        /// If set, blocks sound propagation.
        block_sound: bool,
        /// If set, line is never drawn on the automap,
        /// even if the computer area map power-up is acquired.
        unmapped: bool,
        /// If set, line always appears on the automap,
        /// even if no player has seen it yet.
        premapped: bool,

        // Boom ////////////////////////////////////////////////////////////////
        /// The "use" action can activate other linedefs in the back
        /// (in Doom the "use" action only activates the closest linedef
        /// in the line of sight).
        pass_thru: if (game == .boom) bool else void,
        // Hexen ///////////////////////////////////////////////////////////////
        hexen: if (game == .hexen) packed struct(u7) {
            /// Can be activated more than once.
            multi_activate: bool,
            f0: bool,
            f1: bool,
            f2: bool,
            monster_player_activate: bool,
            unused: bool,
            blocks_all: bool,
        } else void,
        // Strife //////////////////////////////////////////////////////////////
        strife: if (game == .strife) packed struct(u4) {
            /// Can be jumped over by the player.
            railing: bool,
            blocks_floating_monsters: bool,
            /// 25% translucent foreground; 75% translucent background.
            translucent_25fg_75bg: bool,
            /// 75% translucent foreground; 25% translucent background.
            translucent_75fg_25bg: bool,
        } else void,

        _pad: FlagPad,
    };

    return extern struct {
        pub const Self = @This();
        pub const Flags = FlagT;

        /// A possible value for `special`.
        pub const pobj_line_start: u16 = 1;
        /// A possible value for `special`.
        pub const pobj_line_explicit: u16 = 5;

        v_start: u16,
        v_end: u16,
        flags: Flags,
        special: u16,
        tag: u16,
        args: if (game == .hexen) [5]u8 else void,
        right: u16,
        left: u16,

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

        /// To be used as an index into a slice of [`Vertex`].
        pub fn vertexStart(self: *const Self) u16 {
            return std.mem.littleToNative(u16, self.v_start);
        }

        /// To be used as an index into a slice of [`Vertex`].
        pub fn vertexEnd(self: *const Self) u16 {
            return std.mem.littleToNative(u16, self.v_end);
        }

        pub fn flagBits(self: *const Self) FlagInt {
            return @bitCast(self.flags);
        }

        pub fn actionSpecial(self: *const Self) u16 {
            return std.mem.littleToNative(u16, self.special);
        }

        pub fn sectorTag(self: *const Self) u16 {
            return std.mem.littleToNative(u16, self.tag);
        }

        /// a.k.a. the linedef's "front". To be used as an index into a slice of [`SideDef`].
        pub fn rightSide(self: *const Self) u16 {
            return std.mem.littleToNative(u16, self.right);
        }

        /// a.k.a. the linedef's "back". To be used as an index into a slice of [`SideDef`].
        /// Returns `null` if the LE bytes of this value match the bit pattern `0xFFFF`.
        pub fn leftSide(self: *const Self) ?u16 {
            return switch (std.mem.littleToNative(u16, self.left)) {
                0xFFFF => null,
                else => |s| s,
            };
        }
    };
}

// NODES ///////////////////////////////////////////////////////////////////////

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

    x: i16,
    y: i16,
    delta_x: i16,
    delta_y: i16,
    aabb_r: Aabb,
    aabb_l: Aabb,
    child_r: i16,
    child_l: i16,

    pub fn segStart(self: *const Node) Position {
        return Position{
            .x = std.mem.littleToNative(i16, self.x),
            .y = std.mem.littleToNative(i16, self.y),
        };
    }

    pub fn segDelta(self: *const Node) Position {
        return Position{
            .x = std.mem.littleToNative(i16, self.delta_x),
            .y = std.mem.littleToNative(i16, self.delta_y),
        };
    }

    pub fn segEnd(self: *const Node) Position {
        return Position{
            .x = std.mem.littleToNative(i16, self.x) + std.mem.littleToNative(i16, self.delta_x),
            .y = std.mem.littleToNative(i16, self.y) + std.mem.littleToNative(i16, self.delta_y),
        };
    }

    pub fn aabbL(self: *const Node) Aabb {
        return Aabb{
            .top = std.mem.littleToNative(i16, self.aabb_l.top),
            .bottom = std.mem.littleToNative(i16, self.aabb_l.bottom),
            .left = std.mem.littleToNative(i16, self.aabb_l.left),
            .right = std.mem.littleToNative(i16, self.aabb_l.right),
        };
    }

    pub fn aabbR(self: *const Node) Aabb {
        return Aabb{
            .top = std.mem.littleToNative(i16, self.aabb_r.top),
            .bottom = std.mem.littleToNative(i16, self.aabb_r.bottom),
            .left = std.mem.littleToNative(i16, self.aabb_r.left),
            .right = std.mem.littleToNative(i16, self.aabb_r.right),
        };
    }

    pub fn childLeft(self: *const Node) Child {
        const child = std.mem.littleToNative(i16, self.child_l);

        return if (child < 0)
            Child{ .subsector = @intCast(child & 0x7FFF) }
        else
            Child{ .subnode = @intCast(child) };
    }

    pub fn childRight(self: *const Node) Child {
        const child = std.mem.littleToNative(i16, self.child_r);

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

// Unit tests //////////////////////////////////////////////////////////////////

test "LINEDEFS, semantic check" {
    // Ensure that all flag fields are properly padded, et cetera.
    const boom: LineDef(.boom) = undefined;
    const doom: LineDef(.doom) = undefined;
    const doom64: LineDef(.doom64) = undefined;
    const heretic: LineDef(.heretic) = undefined;
    const hexen: LineDef(.hexen) = undefined;
    const strife: LineDef(.strife) = undefined;

    _ = .{ boom, doom, doom64, heretic, hexen, strife };
}

test "LINEDEFS, fromBytes" {
    const bytes = [_]u8{
        // 4th line of Entryway...
        0x05, 0x00, 0x06, 0x00, 0x01, 0x00, 0x67,
        0x00, 0x02, 0x00, 0x03, 0x00, 0xFF, 0xFF,
        // 6th line of Entryway...
        0x07, 0x00, 0x08, 0x00, 0x24, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x05, 0x00, 0x06, 0x00,
    };
    const linedefs = LineDef(.doom).fromBytes(@alignCast(bytes[0..]));

    try std.testing.expectEqual(2, linedefs.len);

    try std.testing.expectEqual(5, linedefs[0].vertexStart());
    try std.testing.expectEqual(6, linedefs[0].vertexEnd());
    try std.testing.expectEqual(1, linedefs[0].flagBits());
    try std.testing.expectEqual(103, linedefs[0].actionSpecial());
    try std.testing.expectEqual(2, linedefs[0].sectorTag());
    try std.testing.expectEqual(3, linedefs[0].rightSide());
    try std.testing.expectEqual(null, linedefs[0].leftSide());

    try std.testing.expectEqual(7, linedefs[1].vertexStart());
    try std.testing.expectEqual(8, linedefs[1].vertexEnd());
    try std.testing.expectEqual(36, linedefs[1].flagBits());
    try std.testing.expectEqual(0, linedefs[1].actionSpecial());
    try std.testing.expectEqual(0, linedefs[1].sectorTag());
    try std.testing.expectEqual(5, linedefs[1].rightSide());
    try std.testing.expectEqual(6, linedefs[1].leftSide());

    const junk = bytes ++ [_]u8{ 0x13, 0x37 };
    // Expected to not panic...
    _ = LineDef(.doom).fromBytesLossy(@alignCast(junk[0..]));
}

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
