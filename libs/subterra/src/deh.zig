//! A [DeHackEd](https://doomwiki.org/wiki/DeHackEd) parser.

const std = @import("std");

pub const CodeptrStart = fn (anytype) void;
pub const CodeptrPer = fn (anytype, frame: i32, name: []const u8) void;
pub const CodeptrEnd = fn (anytype) void;

pub const SoundsStart = fn (anytype) void;
pub const SoundsPer = fn (anytype, key: i32, val: []const u8) void;
pub const SoundsEnd = fn (anytype) void;

pub const SpritesStart = fn (anytype) void;
pub const SpritesPer = fn (anytype, key: i32, val: []const u8) void;
pub const SpritesEnd = fn (anytype) void;

pub const StringsStart = fn (anytype) void;
pub const StringsPer = fn (anytype, key: []const u8, val: []const u8) void;
pub const StringsEnd = fn (anytype) void;

pub fn parse(text: []const u8, context: anytype) Error!void {
    var lines = std.mem.splitScalar(u8, text, '\n');

    while (lines.next()) |line| {
        if (line.len == 0) continue;

        const line_trimmed = std.mem.trimLeft(u8, line, " \t\n\r");

        if (std.ascii.startsWithIgnoreCase(line_trimmed, "Patch File"))
            continue;

        var comment_split = std.mem.splitScalar(u8, line_trimmed, '#');
        const pre_comment = comment_split.next() orelse continue;

        if (pre_comment.len == 0) continue;

        var parts = std.mem.splitAny(u8, pre_comment, " \t");
        const part0 = parts.next() orelse continue;

        blk: {
            inline for (handlers) |handler| {
                if (std.ascii.startsWithIgnoreCase(part0, handler.part0)) {
                    var state = State{ .lines = &lines, .parts = &parts };
                    try handler.func(&state, context);
                    break :blk;
                }
            }

            return error.UnknownTopLevel;
        }
    }
}

/// See [`ThingBits.parse`] and [`WeaponBits.parse`].
pub const BitParseCompat = enum {
    /// - Bit mnemonics are compared case-sensitively.
    /// - The only valid delimiter between bit mnemonics is `+`.
    none,
    /// - Bit mnemonics are compared case-sensitively.
    /// - Bit mnemonics can be delimited by any one of: `+,| \t\f\r`
    boom,
};

pub const ThingBits = packed struct(u64) {
    // Doom+ ///////////////////////////////////////////////////////////////////
    special: bool,
    solid: bool,
    shootable: bool,
    no_sector: bool,
    no_blockmap: bool,
    ambush: bool,
    just_hit: bool,
    just_attacked: bool,
    spawn_ceiling: bool,
    no_gravity: bool,
    dropoff: bool,
    pickup: bool,
    noclip: bool,
    slide: bool,
    float: bool,
    teleport: bool,
    missile: bool,
    dropped: bool,
    shadow: bool,
    no_blood: bool,
    corpse: bool,
    in_float: bool,
    count_kill: bool,
    count_item: bool,
    skull_fly: bool,
    not_dmatch: bool,
    /// Has the DeHackEd mnemonic `TRANSLATION`.
    translation_1: bool,
    /// Has the DeHackEd mnemonic `UNUSED1`.
    translation_2: bool,
    /// Has the DeHackEd mnemonic `UNUSED2`.
    touchy: bool,
    /// Has the DeHackEd mnenonic `UNUSED3`.
    bounces: bool,
    /// Has the DeHackEd mnemonics `FRIEND` and `UNUSED4`.
    friendly: bool,
    translucent: bool,
    // MBF21+ //////////////////////////////////////////////////////////////////
    lo_grav: bool,
    short_mrange: bool,
    dmg_ignored: bool,
    no_radius_dmg: bool,
    force_radius_dmg: bool,
    higher_mprob: bool,
    range_half: bool,
    no_threshold: bool,
    long_melee: bool,
    boss: bool,
    map07_boss1: bool,
    map07_boss2: bool,
    e1m8_boss: bool,
    e2m8_boss: bool,
    e3m8_boss: bool,
    e4m8_boss: bool,
    rip: bool,
    full_vol_sounds: bool,

    _pad: u14,

    /// Note that this works even if `prop_val` only contains an integer.
    pub fn parse(comptime compat: BitParseCompat, prop_val: []const u8) Error!ThingBits {
        const eql = comptime switch (compat) {
            .none => std.static_string_map.defaultEql,
            .boom => std.ascii.eqlIgnoreCase,
        };

        const delimiters = comptime switch (compat) {
            .none => "+",
            .boom => ",+| \t\x0C\r",
        };

        const name_to_bit = comptime std.StaticStringMapWithEql(u64, eql).initComptime(.{
            // Doom+
            .{ "SPECIAL", 0x00000001 },
            .{ "SOLID", 0x00000002 },
            .{ "SHOOTABLE", 0x00000004 },
            .{ "NOSECTOR", 0x00000008 },
            .{ "NOBLOCKMAP", 0x00000010 },
            .{ "AMBUSH", 0x00000020 },
            .{ "JUSTHIT", 0x00000040 },
            .{ "JUSTATTACKED", 0x00000080 },
            .{ "SPAWNCEILING", 0x00000100 },
            .{ "NOGRAVITY", 0x00000200 },
            .{ "DROPOFF", 0x00000400 },
            .{ "PICKUP", 0x00000800 },
            .{ "NOCLIP", 0x00001000 },
            .{ "SLIDE", 0x00002000 },
            .{ "FLOAT", 0x00004000 },
            .{ "TELEPORT", 0x00008000 },
            .{ "MISSILE", 0x00010000 },
            .{ "DROPPED", 0x00020000 },
            .{ "SHADOW", 0x00040000 },
            .{ "NOBLOOD", 0x00080000 },
            .{ "CORPSE", 0x00100000 },
            .{ "INFLOAT", 0x00200000 },
            .{ "COUNTKILL", 0x00400000 },
            .{ "COUNTITEM", 0x00800000 },
            .{ "SKULLFLY", 0x01000000 },
            .{ "NOTDMATCH", 0x02000000 },
            .{ "TRANSLATION", 0x04000000 },
            .{ "TRANSLATION1", 0x04000000 },
            .{ "UNUSED1", 0x08000000 },
            .{ "TRANSLATION2", 0x08000000 },
            .{ "TOUCHY", 0x10000000 },
            .{ "UNUSED2", 0x10000000 },
            .{ "BOUNCES", 0x20000000 },
            .{ "UNUSED3", 0x20000000 },
            .{ "FRIEND", 0x40000000 },
            .{ "FRIENDLY", 0x40000000 },
            .{ "UNUSED4", 0x40000000 },
            .{ "TRANSLUCENT", 0x80000000 },
            // MBF21+
            .{ "LOGRAV", 0x00000001 },
            .{ "SHORTMRANGE", 0x00000002 },
            .{ "DMGIGNORED", 0x00000004 },
            .{ "NORADIUSDMG", 0x00000008 },
            .{ "FORCERADIUSDMG", 0x00000010 },
            .{ "HIGHERMPROB", 0x00000020 },
            .{ "RANGEHALF", 0x00000040 },
            .{ "NOTHRESHOLD", 0x00000080 },
            .{ "LONGMELEE", 0x00000100 },
            .{ "BOSS", 0x00000200 },
            .{ "MAP07BOSS1", 0x00000400 },
            .{ "MAP07BOSS2", 0x00000800 },
            .{ "E1M8BOSS", 0x00001000 },
            .{ "E2M8BOSS", 0x00002000 },
            .{ "E3M8BOSS", 0x00004000 },
            .{ "E4M6BOSS", 0x00008000 },
            .{ "E4M8BOSS", 0x00010000 },
            .{ "RIP", 0x00020000 },
            .{ "FULLVOLSOUNDS", 0x00040000 },
        });

        if (std.fmt.parseInt(u64, prop_val, 0)) |bits| {
            return @bitCast(bits);
        } else |_| {}

        var bits: u64 = 0;
        var iter = std.mem.splitAny(u8, prop_val, delimiters);

        while (iter.next()) |ident| {
            if (ident.len == 0) return error.EmptyBitName;

            if (name_to_bit.get(std.mem.trim(u8, ident, ",+| \t\x0C\r"))) |bit| {
                bits |= bit;
            } else {
                return error.UnknownThingMnemonic;
            }
        }

        return @bitCast(bits);
    }
};

pub const WeaponBits = packed struct(u64) {
    no_thrust: bool,
    silent: bool,
    no_auto_fire: bool,
    flee_melee: bool,
    auto_switch_from: bool,
    no_auto_switch_to: bool,

    _pad: u58,

    pub fn parse(comptime compat: BitParseCompat, prop_val: []const u8) Error!WeaponBits {
        const eql = comptime switch (compat) {
            .none => std.mem.eql,
            .boom => std.ascii.eqlIgnoreCase,
        };

        const delimiters = comptime switch (compat) {
            .none => "+",
            .boom => ",+| \t\x0C\r",
        };

        const name_to_bit = comptime std.StaticStringMapWithEql(u64, eql).initComptime(.{
            .{ "NOTHRUST", 0x00000001 },
            .{ "SILENT", 0x00000002 },
            .{ "NOAUTOFIRE", 0x00000004 },
            .{ "FLEEMELEE", 0x00000008 },
            .{ "AUTOSWITCHFROM", 0x00000010 },
            .{ "NOAUTOSWITCHTO", 0x00000020 },
        });

        if (std.fmt.parseInt(u64, prop_val, 0)) |bits| {
            return @bitCast(bits);
        } else |_| {}

        var bits: u64 = 0;
        var iter = std.mem.splitAny(u8, prop_val, delimiters);

        while (iter.next()) |ident| {
            if (ident.len == 0) return error.EmptyBitName;

            if (name_to_bit.get(std.mem.trim(ident, ",+| \t\x0C\r"))) |bit| {
                bits |= bit;
            } else {
                return error.UnknownThingMnemonic;
            }
        }

        return @bitCast(bits);
    }
};

pub const Error = error{
    DoomVersionMalformed,
    DoomVersionMissingValue,
    EmptyBitName,
    PatchFormatMalformed,
    PatchFormatMissingValue,
    UnknownThingProp,
    UnknownThingMnemonic,
    UnknownTopLevel,
    User,
    ThingMissingNum,
    ThingKeyMalformed,
    ThingPropMalformed,
} || std.fmt.ParseIntError;

const State = struct {
    line: []const u8,
    lines: *std.mem.SplitIterator(u8, .scalar),
    parts: *std.mem.SplitIterator(u8, .any),
};

const Handler = fn (
    state: *State,
    context: anytype,
) Error!void;

// TODO: worth trying to use a `StaticStringMap` performance-wise?
const handlers = [_]struct { part0: []const u8, func: Handler }{
    .{ .part0 = "Frame", .func = processFrame },
    .{ .part0 = "Sprite", .func = processSprite },
    .{ .part0 = "Thing", .func = processThing },
    .{ .part0 = "Pointer", .func = processPointer },
    .{ .part0 = "Sound", .func = processSound },
    .{ .part0 = "Ammo", .func = processAmmo },
    .{ .part0 = "Weapon", .func = processWeapon },
    .{ .part0 = "Cheat", .func = processCheat },
    .{ .part0 = "Misc", .func = processMisc },
    .{ .part0 = "Text", .func = processText },
    // Boom extensions
    .{ .part0 = "[STRINGS]", .func = processStrings },
    .{ .part0 = "[PARS]", .func = processPars },
    .{ .part0 = "[CODEPTR]", .func = processCodeptr },
    .{ .part0 = "[HELPER]", .func = processHelper },
    .{ .part0 = "[SPRITES]", .func = processSprites },
    .{ .part0 = "[SOUNDS]", .func = processSounds },
    .{ .part0 = "[MUSIC]", .func = processMusic },
    // Meta
    .{ .part0 = "Doom", .func = processDoomVersion },
    .{ .part0 = "Patch", .func = processPatchFormat },
};

fn processAmmo(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processCheat(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processCodeptr(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processDoomVersion(state: *State, context: anytype) Error!void {
    if (!std.meta.hasMethod(@TypeOf(context), "doomVersion")) {
        return;
    }

    const kw_version = state.parts.next() orelse return error.DoomVersionMalformed;
    if (!std.mem.eql(u8, kw_version, "version")) return error.DoomVersionMalformed;

    const eq_sign = state.parts.next() orelse return error.DoomVersionMalformed;
    if (!std.mem.eql(u8, eq_sign, "=")) return error.DoomVersionMalformed;

    const val = state.parts.next() orelse return error.DoomVersionMissingValue;
    context.doomVersion(val) catch return error.User;
}

fn processFrame(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processHelper(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processMisc(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processMusic(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processPars(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processPatchFormat(state: *State, context: anytype) Error!void {
    if (!std.meta.hasMethod(@TypeOf(context), "patchFormat")) {
        return;
    }

    const kw_format = state.parts.next() orelse return error.PatchFormatMalformed;
    if (!std.mem.eql(u8, kw_format, "format")) return error.PatchFormatMalformed;

    const eq_sign = state.parts.next() orelse return error.PatchFormatMalformed;
    if (!std.mem.eql(u8, eq_sign, "=")) return error.PatchFormatMalformed;

    const val = state.parts.next() orelse return error.PatchFormatMissingValue;
    context.patchFormat(val) catch return error.User;
}

fn processPointer(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processSound(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processSounds(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processSprite(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processSprites(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processStrings(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processText(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

fn processThing(state: *State, context: anytype) Error!void {
    const Context = @TypeOf(context);

    const ix_str = state.parts.next() orelse return error.ThingMissingNum;
    const ix = try std.fmt.parseInt(i32, ix_str, 0);
    var lparen_split = std.mem.splitScalar(u8, state.line, '(');
    _ = lparen_split.next();

    const thing_key = if (lparen_split.next()) |after_lparen|
        std.mem.trimRight(u8, after_lparen, ") \t")
    else
        null;

    if (std.meta.hasMethod(Context, "onThingStart")) {
        context.onThingStart(ix, thing_key) catch return error.User;
    }

    if (std.meta.hasMethod(Context, "perThingProp")) {
        while (state.lines.next()) |line| {
            if (line.len == 0) break;
            var prop_parts = std.mem.splitScalar(u8, line, '=');

            const key = std.mem.trim(
                u8,
                prop_parts.next() orelse return error.ThingPropMalformed,
                " \t",
            );
            const val = std.mem.trim(
                u8,
                prop_parts.next() orelse return error.ThingPropMalformed,
                " \t#",
            );

            if (std.mem.startsWith(u8, key, "#")) continue;

            context.perThingProp(key, val) catch return error.User;
        }
    }

    if (std.meta.hasMethod(Context, "onThingEnd")) {
        context.onThingEnd() catch return error.User;
    }
}

fn processWeapon(_: *State, _: anytype) Error!void {
    @panic("not yet implemented");
}

/// This is deliberately public to act as a demonstration of what methods [`parse`]
/// checks for on its `context` parameter. All are optional, and it will also
/// never check for any fields or non-method declarations.
pub const TestContext = struct {
    seen_doom_version: bool = false,
    seen_patch_format: bool = false,
    seen_thing: bool = false,

    pub fn doomVersion(self: *TestContext, val: []const u8) anyerror!void {
        self.seen_doom_version = true;
        const int = try std.fmt.parseInt(i32, val, 10);
        try std.testing.expectEqual(2021, int);
    }

    pub fn patchFormat(self: *TestContext, val: []const u8) anyerror!void {
        self.seen_patch_format = true;
        const int = try std.fmt.parseInt(i32, val, 10);
        try std.testing.expectEqual(6, int);
    }

    pub fn onThingStart(_: *TestContext, index: i32, key: ?[]const u8) anyerror!void {
        try std.testing.expectEqual(1337, index);
        try std.testing.expectEqualStrings("Dear Onion", key.?);
    }

    pub fn perThingProp(_: *TestContext, key: []const u8, val: []const u8) anyerror!void {
        if (std.mem.eql(u8, key, "ID #")) {
            try std.testing.expectEqualStrings("3008", val);
        } else if (std.mem.eql(u8, key, "Hit points")) {
            try std.testing.expectEqualStrings("100", val);
        } else if (std.mem.eql(u8, key, "Bits")) {
            const bits = try ThingBits.parse(.none, val);

            try std.testing.expect(bits.solid and
                bits.shootable and
                bits.just_attacked and
                bits.no_gravity and
                bits.float and
                bits.count_kill);
        } else {
            std.debug.print("unexpected key: {s}\n", .{key});
            return error.TestUnexpectedResult;
        }
    }

    pub fn onThingEnd(self: *TestContext) anyerror!void {
        self.seen_thing = true;
    }
};

test "smoke" {
    const sample =
        \\Patch File for DeHackEd v3.0
        \\# Created by hand by jerome-trc
        \\# Note: Use the pound sign ('#') to start comment lines.
        \\
        \\  Doom version = 2021
        \\Patch format = 6
        \\
        \\ Thing 1337 (Dear Onion)
        \\   ID # = 3008
        \\#$Editor category = Monsters\n
        \\Hit points = 100 #
        \\Bits = SOLID+SHOOTABLE+JUSTATTACKED+NOGRAVITY+FLOAT+COUNTKILL
        \\
        \\Frame 1100
        \\Duration = -1
        \\Sprite number = 245
        \\Next frame = 0
        \\
        \\Weapon 6 ( Aranea Imperatrix (Spider Empress) )
        \\Deselect frame = 1453
        \\Select frame = 1452
        \\Bobbing frame = 1451
        \\Shooting frame = 1454
        \\Firing frame = 1549
        \\Ammo per shot = 10
        \\Carousel icon = ECHOES
        \\
        \\Ammo 2 (Nuisances Unknown)
        \\Max ammo = 150
        \\Per ammo = 10
        \\
        \\[PARS]
        \\par 1 230
        \\
        \\[CODEPTR]
        \\FRAME 1131 = Braachsel
        \\
        \\[SOUNDS]
        \\709 = UNSTOP
        \\
        \\[SPRITES]
        \\256 = SOON
        \\
        \\[STRINGS]
        \\EN_EL_SALON = In The Classroom
    ;

    var context = TestContext{};

    try parse(sample, &context);
    try std.testing.expect(context.seen_doom_version);
    try std.testing.expect(context.seen_patch_format);
    try std.testing.expect(context.seen_thing);
}
