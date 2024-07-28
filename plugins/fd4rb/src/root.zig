//! Final Doomer for RatBoom.

const std = @import("std");
const log = std.log.scoped(.fd4rb);

const ratboom = @import("ratboom");
const PCore = ratboom.PCore;

export fn onLoad(_: PCore) void {
    log.info("Initialized successfully.", .{});
}

comptime {
    std.debug.assert(@typeInfo(ratboom.OnLoad).Pointer.child == @TypeOf(onLoad));
}
