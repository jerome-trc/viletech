//! Final Doomer for RatBoom.

const std = @import("std");
const log = std.log.scoped(.fd4rb);

const ratboom = @import("ratboom");
const PCore = ratboom.PCore;

export fn onLoad(pcx: PCore) void {
    if (pcx.raven) {
        log.info("Won't initialize due to Heretic or Hexen.", .{});
        return;
    }

    log.info("Initialized successfully.", .{});
}

comptime {
    std.debug.assert(@typeInfo(ratboom.OnLoad).Pointer.child == @TypeOf(onLoad));
}
