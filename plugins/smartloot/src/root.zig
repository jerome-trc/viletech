const std = @import("std");
const log = std.log.scoped(.smartloot);

const viletech = @import("viletech");

const Core = viletech.Core;
const plugin = viletech.plugin;

export fn onGameStart(cx: *Core) void {
    _ = cx;
    log.info("SmartLoot initialized successfully.", .{});
}

comptime {
    std.debug.assert(@typeInfo(plugin.OnGameStart).Pointer.child == @TypeOf(onGameStart));
}
