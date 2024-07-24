const builtin = @import("builtin");
const std = @import("std");

const c = @import("main.zig").c;

const Console = @import("devgui/Console.zig");
const devgui = @import("devgui.zig");

const Self = @This();
pub const DebugAllocator = std.heap.GeneralPurposeAllocator(.{});
const StreamWriter = std.io.BufferedWriter(4096, std.fs.File.Writer);

pub const C = extern struct {
    core: *Self,
    devgui_open: bool,
    imgui_ctx: *c.ImGuiContext,
    saved_gametick: i32,
};

pub const DevGui = struct {
    left: devgui.State,
    right: devgui.State,

    demo_window: bool = false,
    metrics_window: bool = false,
    debug_log: bool = false,
    id_stack_tool_window: bool = false,
    about_window: bool = false,
    user_guide: bool = false,
};

c: C,
alloc: std.mem.Allocator,
console: Console,
dgui: DevGui,
gpa: ?*DebugAllocator,
stderr_file: std.fs.File.Writer,
stderr_bw: StreamWriter,
stdout_file: std.fs.File.Writer,
stdout_bw: StreamWriter,

pub fn init(gpa: ?*DebugAllocator) !Self {
    const alloc = if (gpa) |g| g.allocator() else std.heap.c_allocator;

    const stderr_file = std.io.getStdErr().writer();
    const stdout_file = std.io.getStdOut().writer();

    return Self{
        .c = Self.C{
            .core = undefined,
            .devgui_open = if (builtin.mode == .Debug) true else false,
            .imgui_ctx = undefined,
            .saved_gametick = -1,
        },
        .alloc = alloc,
        .console = try Console.init(alloc),
        .dgui = Self.DevGui{
            .left = devgui.State.console,
            .right = devgui.State.vfs,
        },
        .gpa = gpa,
        .stderr_file = stderr_file,
        .stderr_bw = std.io.bufferedWriter(stderr_file),
        .stdout_file = stdout_file,
        .stdout_bw = std.io.bufferedWriter(stdout_file),
    };
}

pub fn deinit(self: *Self) void {
    self.stdout_bw.flush() catch {};
    self.stderr_bw.flush() catch {};

    self.console.deinit();
    c.igDestroyContext(self.c.imgui_ctx);

    if (self.gpa) |gpa| {
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
