const std = @import("std");

const Core = @import("Core.zig");
const Doom = @import("Doom.zig");

const Self = @This();

cx: *Core,

doom: struct {
    compat: Doom.Compat,
},

pub fn init(cx: *Core) Self {
    return Self{ .cx = cx, .doom = .{ .compat = .mbf21 } };
}

pub fn doomArgs(self: *const Self) std.mem.Allocator.Error!std.ArrayList(?[*:0]u8) {
    var ret = std.ArrayList(?[*:0]u8).init(self.cx.alloc);

    switch (self.doom.compat) {
        .unspecified => unreachable,
        else => |l| {
            const flag = try std.fmt.allocPrintZ(self.cx.alloc, "-cl", .{});
            const num = try std.fmt.allocPrintZ(self.cx.alloc, "{}", .{l});
            try ret.append(flag.ptr);
            try ret.append(num.ptr);
        },
    }

    return ret;
}

pub fn ui(_: *Self) void {
    // Soon!
}
