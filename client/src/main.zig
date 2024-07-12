const std = @import("std");
const log = std.log.scoped(.viletech);
const meta = @import("meta");

const args = @import("zig-args");
const sdl = @import("sdl2");

const Core = @import("Core.zig");
const imgui = @import("imgui.zig");
const gamemode = @import("gamemode.zig");

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

    var window = try sdl.createWindow(
        "",
        .{ .centered = {} },
        .{ .centered = {} },
        640,
        480,
        .{
            .vis = .shown,
        },
    );
    defer window.destroy();

    var renderer = try sdl.createRenderer(window, null, .{ .accelerated = true });
    defer renderer.destroy();

    const im_ctx = c.igCreateContext(null) orelse return;
    defer c.igDestroyContext(im_ctx);

    const im_io = c.igGetIO();
    im_io.*.ConfigFlags |= c.ImGuiConfigFlags_NavEnableKeyboard;

    _ = imgui.implSdl2.initForSdlRenderer(window, renderer);
    defer imgui.implSdl2.shutdown();
    _ = imgui.implSdlRenderer2.init(renderer);
    defer imgui.implSdlRenderer2.shutdown();

    c.igStyleColorsDark(null);

    outer: while (true) {
        while (sdl.pollNativeEvent()) |native_event| {
            _ = imgui.implSdl2.processEvent(@ptrCast(&native_event));

            switch (sdl.Event.from(native_event)) {
                .quit => break :outer,
                else => {},
            }
        }

        imgui.implSdlRenderer2.newFrame();
        imgui.implSdl2.newFrame();
        c.igNewFrame();

        var b = true;
        c.igShowDemoWindow(&b);

        c.igRender();

        renderer.setScale(
            im_io.*.DisplayFramebufferScale.x,
            im_io.*.DisplayFramebufferScale.y,
        ) catch {};

        try renderer.setColorRGB(0x00, 0x00, 0x00);
        try renderer.clear();
        imgui.implSdlRenderer2.renderDrawData(c.igGetDrawData(), renderer);
        renderer.present();
    }
}
