const std = @import("std");

pub const c = @cImport({
    @cDefine("RATBOOM_ZIG", {});
    @cInclude("i_main.h");
    @cInclude("i_system.h");
    @cUndef("RATBOOM_ZIG");
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
