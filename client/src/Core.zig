const builtin = @import("builtin");
const std = @import("std");

const Console = @import("devgui/Console.zig");
const Frontend = @import("Frontend.zig");
const game = @import("game.zig");
const BoomRng = @import("BoomRng.zig");
const platform = @import("platform.zig");
const zdfs = @import("zdfs.zig");

const Self = @This();
const StreamWriter = std.io.BufferedWriter(4096, std.fs.File.Writer);

const DebugAllocator = std.heap.GeneralPurposeAllocator(.{});

pub const SceneTag = enum {
    exit,
    frontend,
    game,
};

/// An untagged union is used here on the assumption that the engine is compiled
/// in ReleaseSafe mode. Thus most code can easily get at any field without worrying
/// about UB, while hot paths can locally disable safety.
pub const Scene = union {
    frontend: Frontend,
    /// Includes menus.
    game: struct {
        compat: game.Compat,
        demo_insurance: u8,
        boom_basetick: game.Tick,
        game_tick: game.Tick,
        level_time: game.Tick,
        /// Sum of intermission times in game ticks at second resolution.
        level_times_total: game.Tick,
        boomrng: BoomRng,
        true_basetick: game.Tick,
    },
};

allo: ?DebugAllocator,
fs: zdfs.VirtualFs,

stderr_file: std.fs.File.Writer,
stderr_bw: StreamWriter,
stdout_file: std.fs.File.Writer,
stdout_bw: StreamWriter,

displays: std.ArrayList(platform.Display),
console: Console,

scene_tag: SceneTag,
scene: Scene,

pub fn init() !Self {
    const stderr_file = std.io.getStdErr().writer();
    const stdout_file = std.io.getStdOut().writer();

    var gpa: ?DebugAllocator = if (builtin.mode == .Debug) DebugAllocator{} else null;
    const allo = if (gpa) |*g| g.allocator() else std.heap.c_allocator;

    return Self{
        .allo = gpa,
        .fs = try zdfs.VirtualFs.init(),
        .stderr_file = stderr_file,
        .stderr_bw = std.io.bufferedWriter(stderr_file),
        .stdout_file = stdout_file,
        .stdout_bw = std.io.bufferedWriter(stdout_file),
        .displays = std.ArrayList(platform.Display).init(std.heap.c_allocator),
        .console = try Console.init(allo),
        .scene_tag = .frontend,
        .scene = Scene{ .frontend = try Frontend.init(allo) },
    };
}

pub fn deinit(self: *Self) void {
    self.stdout_bw.flush() catch {};
    self.stderr_bw.flush() catch {};

    self.fs.deinit();
    self.displays.deinit();
    self.console.deinit();

    switch (self.scene_tag) {
        .exit => {},
        .frontend => self.scene.frontend.deinit(),
        .game => {},
    }

    if (self.allo) |*allo| {
        _ = allo.detectLeaks();
        _ = allo.deinit();
    }
}

pub fn allocator(self: *const Self) std.mem.Allocator {
    if (self.allo) |a| {
        return a.allocator();
    } else {
        return std.heap.c_allocator;
    }
}

pub fn eprintln(self: *Self, comptime format: []const u8, args: anytype) !void {
    try self.stderr_bw.writer().print(format ++ "\n", args);
    try self.stderr_bw.flush();
}

pub fn println(self: *Self, comptime format: []const u8, args: anytype) !void {
    try self.stdout_bw.writer().print(format ++ "\n", args);
    try self.stdout_bw.flush();
}

pub fn boomCompat(self: *const Self) bool {
    return @intFromEnum(self.scene.game.compat) <= @intFromEnum(game.Compat.boom_compat);
}

pub fn demoCompat(self: *const Self) bool {
    return @intFromEnum(self.scene.game.compat) < @intFromEnum(game.Compat.boom_compat);
}

pub fn boomLogicTick(self: *const Self) game.Tick {
    return self.scene.game.game_tick - self.scene.game.boom_basetick;
}
