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
        .boom, .mbf => u6,
        .doom, .heretic => u7,
        .doom64, .doom64ex, .doom64_2020 => u23,
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

// THINGS //////////////////////////////////////////////////////////////////////

/// See <https://doomwiki.org/wiki/Thing>.
/// These are cast directly from the bytes of a WAD's lump; it is recommended you
/// use the attached methods rather than accessing fields, since the methods
/// ensure that conversion from little to native endianness is performed.
pub fn Thing(game: Game) type {
    const FlagT = switch (game) {
        .boom => packed struct(u16) {
            skill_1_2: bool,
            skill_3: bool,
            skill_4_5: bool,
            /// Alternatively "deaf", but not in terms of sound propagation.
            ambush: bool,
            non_singleplayer: bool,
            non_deathmatch: bool,
            non_coop: bool,

            _pad: u9,
        },
        .doom, .heretic => packed struct(u16) {
            skill_1_2: bool,
            skill_3: bool,
            skill_4_5: bool,
            /// Alternatively "deaf", but not in terms of sound propagation.
            ambush: bool,
            non_singleplayer: bool,

            _pad: u11,
        },
        .doom64 => packed struct(u16) {
            skill_1_2: bool,
            skill_3: bool,
            skill_4: bool,
            /// Alternatively "deaf", but not in terms of sound propagation.
            ambush: bool,
            /// Unused by the original Doom 64 engine.
            non_singleplayer: bool,
            triggered_spawn: bool,
            pickup_trigger: bool,
            kill_trigger: bool,
            secret: bool,
            no_infight: bool,

            _pad: u6,
        },
        .doom64ex => packed struct(u16) {
            skill_1_2: bool,
            skill_3: bool,
            skill_4: bool,
            /// Alternatively "deaf", but not in terms of sound propagation.
            ambush: bool,
            /// Unused by the original Doom 64 engine.
            non_singleplayer: bool,
            triggered_spawn: bool,
            pickup_trigger: bool,
            kill_trigger: bool,
            secret: bool,
            no_infight: bool,
            non_deathmatch: bool,
            non_netgame: bool,

            _pad: u4,
        },
        .doom64_2020 => packed struct(u16) {
            skill_1_2: bool,
            skill_3: bool,
            skill_4: bool,
            /// Alternatively "deaf", but not in terms of sound propagation.
            ambush: bool,
            /// Unused by the original Doom 64 engine.
            non_singleplayer: bool,
            triggered_spawn: bool,
            pickup_trigger: bool,
            kill_trigger: bool,
            secret: bool,
            no_infight: bool,
            non_deathmatch: bool,
            non_netgame: bool,
            nightmare: bool,

            _pad: u3,
        },
        .hexen => packed struct(u16) {
            skill_1_2: bool,
            skill_3: bool,
            skill_4_5: bool,
            /// Alternatively "deaf", but not in terms of sound propagation.
            ambush: bool,
            dormant: bool,
            /// If unset, this is absent to e.g. Hexen's Fighter class.
            class_1: bool,
            /// If unset, this is absent to e.g. Hexen's Cleric class.
            class_2: bool,
            /// If unset, this is absent to e.g. Hexen's Mage class.
            class_3: bool,
            singleplayer: bool,
            coop: bool,
            deathmatch: bool,

            _pad: u5,
        },
        .mbf => packed struct(u16) {
            skill_1_2: bool,
            skill_3: bool,
            skill_4_5: bool,
            /// Alternatively "deaf", but not in terms of sound propagation.
            ambush: bool,
            non_singleplayer: bool,
            non_deathmatch: bool,
            non_coop: bool,
            friendly: bool,

            _pad: u8,
        },
        .strife => packed struct(u16) {
            skill_1_2: bool,
            skill_3: bool,
            skill_4_5: bool,
            stands_still: bool,
            non_singleplayer: bool,
            ambush: bool,
            friendly: bool,
            _unused: bool,
            translucent_25: bool,
            /// Can be combined with `translucent_25` to imply complete invisiblity.
            translucent_75: bool,

            _pad: u6,
        },
    };

    return extern struct {
        pub const Self = @This();
        pub const Flags = FlagT;

        _tid: if (game == .hexen) i16 else void,
        _x: i16,
        _y: i16,
        _z: if (game == .doom64 or game == .hexen) i16 else void,
        _angle: i16,
        _ed_num: EditorNum,
        _flags: Flags,
        _action_special: if (game == .hexen) u8 else void,
        _args: if (game == .hexen) [5]u8 else void,
        _id_doom64: if (game == .doom64) i16 else void,

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

        pub fn actionSpecial(self: *const Self) u8 {
            return self._action_special;
        }

        /// In degrees. 0 is east, north is 90, et cetera.
        pub fn angle(self: *const Self) i16 {
            return std.mem.littleToNative(i16, self._angle);
        }

        pub fn args(self: *const Self) [5]u8 {
            return self._args;
        }

        pub fn editorNum(self: *const Self) EditorNum {
            return std.mem.littleToNative(u16, self._ed_num);
        }

        pub fn flags(self: *const Self) Flags {
            return @bitCast(std.mem.littleToNative(u16, @bitCast(self._flags)));
        }

        pub fn idDoom64(self: *const Self) i16 {
            return switch (game) {
                .doom64,
                .doom64ex,
                .doom64_2020,
                => std.mem.littleToNative(i16, self._id_doom64),
                else => @compileError("only callable for Doom 64 things"),
            };
        }

        pub fn position(self: *const Self) Position {
            return Position{
                .x = std.mem.littleToNative(i16, self._x),
                .y = std.mem.littleToNative(i16, self._y),
            };
        }

        pub fn thingId(self: *const Self) i16 {
            if (game == .hexen)
                return std.mem.littleToNative(i16, self._tid)
            else
                @compileError("only callable for Hexen things");
        }

        pub fn zPos(self: *const Self) i16 {
            if (game == .doom64 or game == .hexen)
                return std.mem.littleToNative(i16, self._z)
            else
                @compileError("only callable for Doom 64 or Hexen things");
        }
    };
}

// VERTEXES ////////////////////////////////////////////////////////////////////

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
        var min = Position{ .x = 0, .y = 0 };
        var max = Position{ .x = 0, .y = 0 };

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

    pub fn position(self: *const Vertex) Position {
        return Position{
            .x = std.mem.littleToNative(i16, self.x),
            .y = std.mem.littleToNative(i16, self.y),
        };
    }
};

/// See [`Vertex.bounds`].
pub const Bounds = struct {
    min: Position,
    max: Position,
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

test "THINGS, semantic check" {
    // Ensure that all flag fields are properly padded, et cetera.
    const boom: Thing(.boom) = undefined;
    const doom: Thing(.doom) = undefined;
    const doom64: Thing(.doom64) = undefined;
    const doom64ex: Thing(.doom64ex) = undefined;
    const doom64_2020: Thing(.doom64_2020) = undefined;
    // If the Doom format compiles, the Heretic format compiles.
    const hexen: Thing(.hexen) = undefined;
    const mbf: Thing(.mbf) = undefined;
    const strife: Thing(.strife) = undefined;

    _ = .{ boom, doom, doom64, doom64ex, doom64_2020, hexen, mbf, strife };
}

test "THINGS, Hexen, fromBytes" {
    const bytes = [_]u8{
        // Thing 118 (from 0) of Hexen's MAP01...
        0x02, 0x00, 0xA0, 0xFE, 0x70, 0xF9, 0x00, 0x00, 0x2D, 0x00,
        0x81, 0x1F, 0xE7, 0x07, 0x50, 0x0E, 0x01, 0x00, 0x00, 0x00,
    };
    const things = Thing(.hexen).fromBytes(@alignCast(bytes[0..]));

    try std.testing.expectEqual(1, things.len);

    try std.testing.expectEqual(2, things[0].thingId());
    try std.testing.expectEqual(-352, things[0].position().x);
    try std.testing.expectEqual(-1680, things[0].position().y);
    try std.testing.expectEqual(0, things[0].zPos());
    try std.testing.expectEqual(45, things[0].angle());
    try std.testing.expectEqual(8065, things[0].editorNum());
    const flags = things[0].flags();
    try std.testing.expectEqual(2023, @as(u16, @bitCast(flags)));
    try std.testing.expect(
        flags.skill_1_2 and
            flags.skill_3 and
            flags.skill_4_5 and
            !flags.ambush and
            !flags.dormant and
            flags.class_1 and
            flags.class_2 and
            flags.class_3 and
            flags.singleplayer and
            flags.coop and
            flags.deathmatch,
    );
    try std.testing.expectEqual(80, things[0].actionSpecial());
    try std.testing.expectEqual([5]u8{ 14, 1, 0, 0, 0 }, things[0].args());
}

test "VERTEXES, fromBytes" {
    const bytes = [8]u8{
        // First vertex of E1M1...
        0x40, 0x04, 0xA0, 0xF1,
        // Second vertex of E1M1...
        0x00, 0x04, 0xA0, 0xF1,
    };
    const vertexes = Vertex.fromBytes(@alignCast(bytes[0..]));

    try std.testing.expectEqual(
        Position{ .x = 1088, .y = -3680 },
        vertexes[0].position(),
    );
    try std.testing.expectEqual(
        Position{ .x = 1024, .y = -3680 },
        vertexes[1].position(),
    );
}
