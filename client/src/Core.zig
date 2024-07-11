const std = @import("std");

const zdfs = @import("zdfs.zig");

const Self = @This();
const StreamWriter = std.io.BufferedWriter(4096, std.fs.File.Writer);

allo: std.mem.Allocator,
fs: zdfs.VirtualFs,

stderr_file: std.fs.File.Writer,
stderr_bw: StreamWriter,
stdout_file: std.fs.File.Writer,
stdout_bw: StreamWriter,

pub fn init() !Self {
    const stderr_file = std.io.getStdErr().writer();
    const stdout_file = std.io.getStdOut().writer();

    return .{
        .allo = std.heap.c_allocator,
        .fs = try zdfs.VirtualFs.init(),
        .stderr_file = stderr_file,
        .stderr_bw = std.io.bufferedWriter(stderr_file),
        .stdout_file = stdout_file,
        .stdout_bw = std.io.bufferedWriter(stdout_file),
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
