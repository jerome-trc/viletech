//! Calls into SDL2 to process events, present frames, and manage audio.

const builtin = @import("builtin");
const std = @import("std");
const log = std.log.scoped(.platform);
const meta = @import("meta");

const c = @import("root").c;
const sdl = @import("sdl2");

const Core = @import("Core.zig");
const devgui = @import("devgui.zig");
const imgui = @import("imgui.zig");

const window_icon = @embedFile("viletech.png");

/// Ties an SDL window to its associated state.
pub const Display = struct {
    const Self = @This();

    window: sdl.Window,
    renderer: sdl.Renderer,
    im_ctx: *imgui.Context,
    dgui: struct {
        open: bool,

        left: devgui.State,
        right: devgui.State,

        demo_window: bool = false,
        metrics_window: bool = false,
        debug_log: bool = false,
        id_stack_tool_window: bool = false,
        about_window: bool = false,
        user_guide: bool = false,
    },

    pub fn init() !Self {
        const window = try sdl.createWindow(
            "VileTech " ++ meta.version,
            .{ .centered = {} },
            .{ .centered = {} },
            1024,
            768,
            .{
                .resizable = true,
                .vis = .shown,
                .allow_high_dpi = true,
            },
        );

        const renderer = try sdl.createRenderer(window, null, .{ .accelerated = true });

        if (sdl.c.SDL_RWFromConstMem(window_icon.ptr, window_icon.len)) |rwop| {
            defer _ = sdl.c.SDL_RWclose(rwop);

            if (sdl.c.IMG_LoadPNG_RW(rwop)) |surface| {
                sdl.c.SDL_SetWindowIcon(window.ptr, surface);
            }
        }

        const im_ctx = c.igCreateContext(null) orelse return error.ContextCreateFail;
        c.igSetCurrentContext(im_ctx);

        _ = imgui.implSdl2.initForSdlRenderer(window, renderer);
        _ = imgui.implSdlRenderer2.init(renderer);

        return Self{
            .window = window,
            .renderer = renderer,
            .im_ctx = im_ctx,
            .dgui = .{ .open = builtin.mode == .Debug, .left = .console, .right = .vfs },
        };
    }

    pub fn deinit(self: *Self) void {
        c.igSetCurrentContext(self.im_ctx);
        imgui.implSdl2.shutdown();
        imgui.implSdlRenderer2.shutdown();
        c.igDestroyContext(self.im_ctx);
        self.renderer.destroy();
        self.window.destroy();
    }

    pub fn windowIdIs(self: *const Self, other: u32) bool {
        const id = self.window.getID() catch {
            std.log.warn("SDL window {} ID retrieval failed", .{self.window.ptr});
            return false;
        };

        return id == other;
    }

    pub fn newFrame(self: *Self) void {
        c.igSetCurrentContext(self.im_ctx);
        imgui.implSdlRenderer2.newFrame();
        imgui.implSdl2.newFrame();
        c.igNewFrame();
    }

    pub fn finishFrame(self: *Self, imgui_io: [*c]c.ImGuiIO) !void {
        c.igRender();

        self.renderer.setScale(
            imgui_io.*.DisplayFramebufferScale.x,
            imgui_io.*.DisplayFramebufferScale.y,
        ) catch {};

        try self.renderer.setColorRGB(0x00, 0x00, 0x00);
        try self.renderer.clear();
        imgui.implSdlRenderer2.renderDrawData(c.igGetDrawData(), self.renderer);
        self.renderer.present();
    }
};

pub fn onWindowEvent(cx: *Core, event: sdl.WindowEvent) bool {
    var app_exit = false;

    switch (event.type) {
        .close => {
            for (cx.displays.items, 0..) |*display, i| {
                if (!display.windowIdIs(event.window_id)) {
                    continue;
                }

                display.deinit();
                _ = cx.displays.swapRemove(i);
                app_exit = cx.displays.items.len == 0;
                break;
            }
        },
        else => {},
    }

    return app_exit;
}
