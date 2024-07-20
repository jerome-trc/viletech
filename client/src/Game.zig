//! Symbols fundamental to the "Doom the game" but not the engine's plumbing.

const builtin = @import("builtin");
const std = @import("std");
const log = std.log.scoped(.game);

const actor = @import("sim/actor.zig");
const BoomRng = @import("BoomRng.zig");
const Core = @import("Core.zig");
const flecs = @import("flecs.zig");
const Frontend = @import("Frontend.zig");
const Path = @import("stdx.zig").Path;
const plugin = @import("plugin.zig");

const Self = @This();

pub const Compat = enum {
    doom_v1_2,
    doom_v1_666,
    doom2_v1_9,
    ult_doom,
    final_doom,
    dos_doom,
    tas_doom,
    boom_compat,
    boom_v2_01,
    boom_v2_02,
    lxdoom1,
    mbf,
    prboom1,
    prboom2,
    prboom3,
    prboom4,
    prboom5,
    prboom6,
    placeholder18,
    placeholder19,
    placeholder20,
    mbf21,

    const boom = Compat.boom_v2_01;
    const best = Compat.mbf21;

    pub fn boomCompat(self: Compat) bool {
        return @intFromEnum(self) <= @intFromEnum(Compat.boomCompat);
    }

    pub fn demoCompat(self: Compat) bool {
        return @intFromEnum(self) < @intFromEnum(Compat.boomCompat);
    }

    pub fn prettyName(self: Compat) [:0]const u8 {
        return switch (self) {
            .doom_v1_2 => "Doom v1.2",
            .doom_v1_666 => "Doom v1.666",
            .doom2_v1_9 => "Doom & Doom 2 v1.9",
            .ult_doom => "Ultimate Doom & Doom95",
            .final_doom => "Final Doom",
            .dos_doom => "DosDoom 0.47",
            .tas_doom => "TASDoom",
            .boom_compat => "Boom's Compatibility Mode",
            .boom_v2_01 => "Boom v2.01",
            .boom_v2_02 => "Boom v2.02",
            .lxdoom1 => "LxDoom v1.3.2+",
            .mbf => "Marine's Best Friend",
            .prboom1 => "PrBoom 2.03beta?",
            .prboom2 => "PrBoom 2.1.0-2.1.1",
            .prboom3 => "PrBoom 2.2.x",
            .prboom4 => "PrBoom 2.3.x",
            .prboom5 => "PrBoom 2.4.0",
            .prboom6 => "PrBoom Latest",
            .placeholder18, .placeholder19, .placeholder20 => "",
            .mbf21 => "MBF 21",
        };
    }
};

pub const Skill = enum {
    /// i.e. "I'm too young to die."
    l1,
    /// i.e. "Hey, not too rough."
    l2,
    /// i.e. "Hurt me plenty."
    l3,
    /// i.e. "Ultra-Violence."
    l4,
    /// i.e. "Nightmare!"
    l5,
};

pub const Tick = i32;

pub const Rules = struct {
    compat: Compat,
    skill: Skill,
};

const dynlib_ext = switch (builtin.os.tag) {
    .linux => ".so",
    .windows => ".dll",
    else => @compileError("unsupported OS"),
};

plugin: struct {
    libs: std.ArrayList(std.DynLib),
    /// Paths are owned by this structure and null-terminated for ImGui's benefit.
    paths: std.ArrayList(Path),
},

boomrng: BoomRng,
boom_basetick: Tick,
compat: Compat,
demo_insurance: u8,
game_tick: Tick,
level_time: Tick,
/// Sum of intermission times in game ticks at second resolution.
level_times_total: Tick,
true_basetick: Tick,
world: flecs.World,

pub fn init(cx: *Core, load_order: []Frontend.Item) !Self {
    var self = Self{
        .plugin = .{
            .libs = std.ArrayList(std.DynLib).init(cx.alloc),
            .paths = std.ArrayList(Path).init(cx.alloc),
        },
        .boomrng = BoomRng.init(false),
        .boom_basetick = 0,
        .compat = .mbf21,
        .demo_insurance = 0,
        .game_tick = 0,
        .level_time = 0,
        .level_times_total = 0,
        .true_basetick = 0,
        .world = try flecs.World.init(),
    };

    self.world.defineComponent(actor.Space) catch unreachable;

    for (load_order) |item| {
        if (!item.enabled) {
            continue;
        }

        const path = item.path;

        if (std.ascii.eqlIgnoreCase(std.fs.path.extension(path), dynlib_ext)) {
            try self.plugin.paths.append(try cx.alloc.dupeZ(u8, path));
            var dynlib = try std.DynLib.open(path);
            log.info("Loaded plugin: {s}", .{path});

            if (dynlib.lookup(plugin.OnGameStart, "onGameStart")) |func| {
                func(cx);
            }

            try self.plugin.libs.append(dynlib);
            continue;
        }

        try cx.fs.mount(path);
        cx.alloc.free(path);
    }

    cx.fs.initHashChains();

    cx.alloc.free(load_order);
    return self;
}

pub fn deinit(self: *Self, cx: *Core) !void {
    for (self.plugin.libs.items) |*dynlib| {
        if (dynlib.lookup(plugin.OnGameClose, "onGameClose")) |func| {
            func(cx);
        }

        dynlib.close();
    }

    for (self.plugin.paths.items) |path| {
        cx.alloc.free(path);
    }

    self.plugin.libs.deinit();
    self.plugin.paths.deinit();
    try self.world.deinit();
}
