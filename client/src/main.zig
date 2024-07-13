const builtin = @import("builtin");
const std = @import("std");
const log = std.log.scoped(.main);
const meta = @import("meta");

const args = @import("zig-args");
const sdl = @import("sdl2");

const Core = @import("Core.zig");
const imgui = @import("imgui.zig");
const gamemode = @import("gamemode.zig");
const platform = @import("platform.zig");

comptime {
    if (builtin.mode == .ReleaseFast or builtin.mode == .ReleaseSmall) {
        @compileError("ReleaseFast and ReleaseSmall builds are currently unsupported");
    }
}

pub const c = @cImport({
    @cDefine("CIMGUI_USE_SDL2", {});
    @cDefine("CIMGUI_DEFINE_ENUMS_AND_STRUCTS", {});
    @cInclude("cimgui.h");
    @cInclude("cimgui_impl.h");
    @cUndef("CIMGUI_USE_SDL2");
    @cUndef("CIMGUI_DEFINE_ENUMS_AND_STRUCTS");

    @cInclude("zdfs/zdfs.h");
});

const Params = struct {
    help: bool = false,
    version: bool = false,

    pub const shorthands = .{
        .h = "help",
        .V = "version",
    };

    pub const meta = .{
        .usage_summary = "[options...]",
        .option_docs = .{
            .help = "Print this usage information and then exit",
            .version = "Print version/compile information and then exit",
        },
    };
};

const Verbs = union(enum) {};

pub fn main() !void {
    var cx = try Core.init();
    defer cx.deinit();

    const opts = try args.parseWithVerbForCurrentProcess(Params, Verbs, std.heap.page_allocator, .print);
    defer opts.deinit();

    if (opts.options.help) {
        try args.printHelp(Params, "viletech", cx.stdout_file);
        return;
    }

    if (opts.options.version) {
        try cx.println("{s} {s}", .{ meta.version, meta.commit });
        return;
    }

    gamemode.start();

    try sdl.init(.{
        .video = true,
        .events = true,
        .audio = true,
    });
    defer sdl.quit();

    try sdl.image.init(.{ .png = true });
    defer sdl.image.quit();

    try cx.displays.append(try platform.Display.init());

    const imgui_io = c.igGetIO();
    imgui_io.*.ConfigFlags |= c.ImGuiConfigFlags_NavEnableKeyboard;

    c.igStyleColorsDark(null);

    outer: while (true) {
        while (sdl.pollNativeEvent()) |native_event| {
            _ = imgui.implSdl2.processEvent(@ptrCast(&native_event));

            switch (sdl.Event.from(native_event)) {
                .quit => break :outer,
                .window => |event| if (platform.onWindowEvent(&cx, event)) return else {},
                else => {},
            }
        }

        for (cx.displays.items) |*display| {
            display.newFrame();
            c.igShowDemoWindow(null);
            try display.finishFrame(imgui_io);
        }
    }
}
