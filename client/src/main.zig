const builtin = @import("builtin");
const std = @import("std");

const Core = @import("Core.zig");
const devgui = @import("devgui.zig");
const gamemode = @import("gamemode.zig");

pub const c = @cImport({
    @cDefine("RATBOOM_ZIG", {});
    @cInclude("doomdef.h");
    @cInclude("doomstat.h");
    @cInclude("g_game.h");
    @cInclude("i_main.h");
    @cInclude("i_system.h");
    @cInclude("i_video.h");
    @cInclude("lprintf.h");
    @cInclude("s_sound.h");
    @cInclude("w_wad.h");
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

    cx.c.core = &cx;
    return dsdaMain(&cx.c, argc, argv);
}

export fn windowIcon(size: *i32) [*]const u8 {
    const bytes = @embedFile("viletech.png");
    size.* = bytes.len;
    return bytes;
}
