const std = @import("std");

const gamemode = @import("gamemode.zig");

pub const c = @cImport({
    @cDefine("RATBOOM_ZIG", {});
    @cInclude("i_main.h");
    @cInclude("i_system.h");
    @cUndef("RATBOOM_ZIG");

    @cDefine("CIMGUI_USE_SDL2", {});
    @cDefine("CIMGUI_DEFINE_ENUMS_AND_STRUCTS", {});
    @cInclude("cimgui.h");
    @cInclude("cimgui_impl.h");
    @cUndef("CIMGUI_USE_SDL2");
    @cUndef("CIMGUI_DEFINE_ENUMS_AND_STRUCTS");
});

pub const Core = struct {
    pub const C = struct {
        core: *Core,
        saved_gametick: i32,
    };

    c: C,
};

extern "C" fn dsdaMain(
    ccx: *Core.C,
    argc: c_int,
    argv: [*][*:0]u8,
) c_int;

export fn zigMain(argc: c_int, argv: [*][*:0]u8) c_int {
    gamemode.start();

    var cx = Core{
        .c = Core.C{
            .core = undefined,
            .saved_gametick = -1,
        },
    };

    cx.c.core = &cx;
    return dsdaMain(&cx.c, argc, argv);
}

export fn windowIcon(size: *i32) [*]const u8 {
    const bytes = @embedFile("viletech.png");
    size.* = bytes.len;
    return bytes;
}
