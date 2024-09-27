//! Wraps a DoomTools lexer to provide a C-esque preprocessor.
//!
//! This code is adapted from DoomTools. See `/legal/doomtools.txt`.

const std = @import("std");

const Deque = @import("deque").Deque;

const Lexer = @import("Lexer.zig");

const Self = @This();

lexer: Lexer,
line_beginning: bool,
if_stack: Deque(bool),
macros: std.StringHashMapUnmanaged([]const u8),

pub fn init(alloc: std.mem.Allocator, text: []const u8) !Self {
    return Self{
        .lexer = try Lexer.init(text),
        .line_beginning = true,
        .if_stack = try Deque(bool).init(alloc),
        .macros = std.StringHashMapUnmanaged([]const u8){},
    };
}

pub fn deinit(self: *Self) void {
    self.if_stack.deinit();
    self.macros.deinit(self.if_stack.allocator);
}

test "smoke" {
    const sample =
        \\#include <doom19>
        \\#include <friendly>
        \\
        \\// copy BFG pickup into backpack
        \\thing MTF_BACKPACK : thing MTF_BFG
        \\{
        \\	//keep ednum, though
        \\	ednum 8
        \\}
        \\
    ;

    var preproc = try Self.init(std.testing.allocator, sample);
    defer preproc.deinit();
}
