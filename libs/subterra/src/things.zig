const std = @import("std");

const root = @import("root.zig");

const EditorNum = root.EditorNum;
const Game = root.Game;
const Pos16 = root.Pos16;

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

        pub fn position(self: *const Self) Pos16 {
            return Pos16{
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
