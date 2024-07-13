const std = @import("std");

const Prng = @import("Prng.zig");
const platform = @import("platform.zig");
const zdfs = @import("zdfs.zig");

const Self = @This();
const StreamWriter = std.io.BufferedWriter(4096, std.fs.File.Writer);

pub const GameTick = i32;

pub const Compat = enum {
    doomV1p2,
    doomV1p666,
    doom2V1p9,
    ultDoom,
    finalDoom,
    dosDoom,
    tasDoom,
    boomCompat,
    boomV2p01,
    boomV2p02,
    lxDoom1,
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

    const boom = @This().boomV2p01;
    const best = @This().mbf21;
};

/// An untagged union is used here on the assumption that the engine is compiled
/// in ReleaseSafe mode. Thus most code can easily get at any field without worrying
/// about UB, while hot paths can locally disable safety.
pub const Scene = union {
    frontend: struct {
        // ???
    },
    /// Includes menus.
    game: struct {
        compat: Compat,
        game_tick: GameTick,
        boom_basetick: GameTick,
        true_basetick: GameTick,
        /// In game ticks.
        level_time: GameTick,
        /// Sum of intermission times in game ticks at second resolution.
        level_times_total: GameTick,
        prng: Prng,
    },
};

allo: std.mem.Allocator,
fs: zdfs.VirtualFs,

stderr_file: std.fs.File.Writer,
stderr_bw: StreamWriter,
stdout_file: std.fs.File.Writer,
stdout_bw: StreamWriter,

displays: std.ArrayList(platform.Display),

scene: Scene,

pub fn init() !Self {
    const stderr_file = std.io.getStdErr().writer();
    const stdout_file = std.io.getStdOut().writer();

    return Self{
        .allo = std.heap.c_allocator,
        .fs = try zdfs.VirtualFs.init(),
        .stderr_file = stderr_file,
        .stderr_bw = std.io.bufferedWriter(stderr_file),
        .stdout_file = stdout_file,
        .stdout_bw = std.io.bufferedWriter(stdout_file),
        .displays = std.ArrayList(platform.Display).init(std.heap.c_allocator),
        .scene = Scene{ .frontend = .{} },
    };
}

pub fn deinit(self: *Self) void {
    self.stdout_bw.flush() catch {};
    self.stderr_bw.flush() catch {};

    self.fs.deinit();
}

pub fn eprintln(self: *Self, comptime format: []const u8, args: anytype) !void {
    try self.stderr_bw.writer().print(format ++ "\n", args);
    try self.stderr_bw.flush();
}

pub fn println(self: *Self, comptime format: []const u8, args: anytype) !void {
    try self.stdout_bw.writer().print(format ++ "\n", args);
    try self.stdout_bw.flush();
}

pub fn boom_compat(self: *const Self) bool {
    return @intFromEnum(self.scene.game.compat) <= @intFromEnum(Compat.boomCompat);
}

pub fn demo_compat(self: *const Self) bool {
    return @intFromEnum(self.scene.game.compat) < @intFromEnum(Compat.boomCompat);
}

pub fn boom_logictic(self: *const Self) GameTick {
    self.scene.game.game_tick - self.scene.game.boom_basetick;
}
