const builtin = @import("builtin");
const std = @import("std");

const Core = @import("Core.zig");
const deh = @import("deh.zig");
const devgui = @import("devgui.zig");
const exports = @import("exports.zig");
const gamemode = @import("gamemode.zig");

comptime {
    std.testing.refAllDeclsRecursive(exports);
}

pub const c = @cImport({
    @cDefine("RATBOOM_ZIG", {});
    @cInclude("doomdef.h");
    @cInclude("doomstat.h");
    @cInclude("g_game.h");
    @cInclude("i_main.h");
    @cInclude("i_system.h");
    @cInclude("i_video.h");
    @cInclude("lprintf.h");
    @cInclude("m_random.h");
    @cInclude("p_map.h");
    @cInclude("s_sound.h");
    @cInclude("w_wad.h");
    @cInclude("dsda/aim.h");
    @cUndef("RATBOOM_ZIG");

    @cDefine("CIMGUI_DEFINE_ENUMS_AND_STRUCTS", {});
    @cDefine("CIMGUI_USE_OPENGL3", {});
    @cDefine("CIMGUI_USE_SDL2", {});
    @cInclude("cimgui.h");
    @cInclude("cimgui_impl.h");
    @cUndef("CIMGUI_DEFINE_ENUMS_AND_STRUCTS");
    @cUndef("CIMGUI_USE_OPENGL3");
    @cUndef("CIMGUI_USE_SDL2");
});

pub const std_options = std.Options{ .logFn = logFn };

extern "C" fn dsdaMain(
    ccx: *Core.C,
    argc: c_int,
    argv: [*][*:0]u8,
) c_int;

export fn zigMain(argc: c_int, argv: [*][*:0]u8) c_int {
    gamemode.start();

    var gpa: if (builtin.mode == .Debug) Core.DebugAllocator else void = undefined;
    var core_alloc: std.mem.Allocator = undefined;

    if (builtin.mode == .Debug) {
        gpa = Core.DebugAllocator{};
        core_alloc = gpa.allocator();
    } else {
        gpa = {};
        core_alloc = std.heap.c_allocator;
    }

    var cx = Core.init(if (builtin.mode == .Debug) &gpa else null) catch return 1;
    defer cx.deinit();

    if (builtin.mode == .Debug) {
        std.log.scoped(.ratboom).info("*** DEBUG BUILD ***", .{});
    }

    return dsdaMain(&cx.c, argc, argv);
}

fn logFn(
    comptime message_level: std.log.Level,
    comptime scope: @TypeOf(.enum_literal),
    comptime format: []const u8,
    args: anytype,
) void {
    const Console = @import("devgui/Console.zig");

    blk: {
        const level_txt = comptime message_level.asText();
        const prefix2 = if (scope == .default) ": " else "(" ++ @tagName(scope) ++ "): ";
        const msg = std.fmt.allocPrint(
            Console.std_log.allocator,
            level_txt ++ prefix2 ++ format,
            args,
        ) catch break :blk;

        Console.std_log_mutex.lock();
        defer Console.std_log_mutex.unlock();
        Console.std_log.pushBack(msg) catch {};
    }

    std.log.defaultLog(message_level, scope, format, args);
}

export fn windowIcon(size: *i32) [*]const u8 {
    const bytes = @embedFile("viletech.png");
    size.* = bytes.len;
    return bytes;
}
