//! State living in `main`'s stack frame that gets passed through the application.

const builtin = @import("builtin");
const std = @import("std");
const StreamWriter = std.io.BufferedWriter(4096, std.fs.File.Writer);

const Doom = @import("Doom.zig");
const Frontend = @import("Frontend.zig");

const Self = @This();

pub const Scene = union(enum) {
    entry: void,
    frontend: Frontend,
    /// Attached are arguments to pass to the C code (for now).
    doom: std.ArrayList(?[*:0]u8),
    editor: void,
    hellblood: void,
};

pub const Transition = enum {
    none,
    entry_to_frontend,
    frontend_to_doom,
    frontend_to_editor,
    frontend_to_exit,
    frontend_to_hellblood,
};

alloc: std.mem.Allocator,
scene: Scene,

stderr_file: std.fs.File.Writer,
stderr_bw: StreamWriter,
stdout_file: std.fs.File.Writer,
stdout_bw: StreamWriter,

transition: Transition,

pub fn init(alloc: std.mem.Allocator) !Self {
    const stderr_file = std.io.getStdErr().writer();
    const stdout_file = std.io.getStdOut().writer();

    return Self{
        .alloc = alloc,
        .scene = Scene{ .entry = {} },
        .transition = .entry_to_frontend,
        .stderr_file = stderr_file,
        .stderr_bw = std.io.bufferedWriter(stderr_file),
        .stdout_file = stdout_file,
        .stdout_bw = std.io.bufferedWriter(stdout_file),
    };
}

pub fn deinit(self: *Self) !void {
    self.stdout_bw.flush() catch {};
    self.stderr_bw.flush() catch {};
}

pub fn eprintln(
    self: *Self,
    comptime format: []const u8,
    args: anytype,
) StreamWriter.Error!void {
    try self.stderr_bw.writer().print(format ++ "\n", args);
    try self.stderr_bw.flush();
}

pub fn println(
    self: *Self,
    comptime format: []const u8,
    args: anytype,
) StreamWriter.Error!void {
    try self.stdout_bw.writer().print(format ++ "\n", args);
    try self.stdout_bw.flush();
}
