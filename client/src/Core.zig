const builtin = @import("builtin");
const std = @import("std");

const Console = @import("devgui/Console.zig");
const Frontend = @import("Frontend.zig");
const Game = @import("Game.zig");
const Path = @import("stdx.zig").Path;
const platform = @import("platform.zig");
const zdfs = @import("zdfs.zig");

const Self = @This();
const StreamWriter = std.io.BufferedWriter(4096, std.fs.File.Writer);

const DebugAllocator = std.heap.GeneralPurposeAllocator(.{});

pub const SceneTag = enum {
    frontend,
    game,
};

/// An untagged union is used here on the assumption that the engine is compiled
/// in ReleaseSafe mode. Thus most code can easily get at any field without worrying
/// about UB, while hot paths can locally disable safety.
pub const Scene = union {
    frontend: Frontend,
    /// Includes menus.
    game: Game,
};

pub const Transition = union(enum) {
    none,
    exit,
    frontend,
    game: struct {
        load_order: std.ArrayList(Frontend.Item).Slice,
    },
};

/// Only non-null in debug builds for detecting leaks.
gpa: ?DebugAllocator,
alloc: std.mem.Allocator,

fs: zdfs.VirtualFs,

stderr_file: std.fs.File.Writer,
stderr_bw: StreamWriter,
stdout_file: std.fs.File.Writer,
stdout_bw: StreamWriter,

displays: std.ArrayList(platform.Display),
console: Console,

scene_tag: SceneTag,
scene: Scene,
transition: Transition,

pub fn init() !Self {
    const stderr_file = std.io.getStdErr().writer();
    const stdout_file = std.io.getStdOut().writer();

    var gpa: ?DebugAllocator = if (builtin.mode == .Debug) DebugAllocator{} else null;
    const alloc = if (gpa) |*g| g.allocator() else std.heap.c_allocator;

    return Self{
        .gpa = gpa,
        .alloc = alloc,
        .fs = try zdfs.VirtualFs.init(),
        .stderr_file = stderr_file,
        .stderr_bw = std.io.bufferedWriter(stderr_file),
        .stdout_file = stdout_file,
        .stdout_bw = std.io.bufferedWriter(stdout_file),
        .displays = std.ArrayList(platform.Display).init(std.heap.c_allocator),
        .console = try Console.init(alloc),
        .scene_tag = .frontend,
        .scene = Scene{ .frontend = try Frontend.init(alloc) },
        .transition = .none,
    };
}

pub fn deinit(self: *Self) void {
    self.stdout_bw.flush() catch {};
    self.stderr_bw.flush() catch {};

    self.fs.deinit();
    self.displays.deinit();
    self.console.deinit();

    switch (self.scene_tag) {
        .frontend => self.scene.frontend.deinit(),
        .game => {},
    }

    if (self.gpa) |*gpa| {
        _ = gpa.detectLeaks();
        _ = gpa.deinit();
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

pub fn deinitScene(self: *Self) !void {
    switch (self.scene_tag) {
        .frontend => self.scene.frontend.deinit(),
        .game => try self.scene.game.deinit(self),
    }
}

pub fn boomCompat(self: *const Self) bool {
    return @intFromEnum(self.scene.game.compat) <= @intFromEnum(Game.Compat.boom_compat);
}

pub fn demoCompat(self: *const Self) bool {
    return @intFromEnum(self.scene.game.compat) < @intFromEnum(Game.Compat.boom_compat);
}

pub fn boomLogicTick(self: *const Self) Game.Tick {
    return self.scene.game.game_tick - self.scene.game.boom_basetick;
}
