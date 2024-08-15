const builtin = @import("builtin");
const std = @import("std");

const c = @import("main.zig").c;

const Console = @import("devgui/Console.zig");
const Core = @import("Core.zig");
const imgui = @import("imgui.zig");
const MusicGui = @import("devgui/MusicGui.zig");
const PrefGui = @import("devgui/PrefGui.zig");
const VfsGui = @import("devgui/VfsGui.zig");

pub const State = enum(c_int) {
    console,
    music,
    prefs,
    vfs,
};

pub fn frameBegin(ccx: *Core.C) callconv(.C) void {
    c.igSetCurrentContext(ccx.imgui_ctx);
    imgui.impl_gl3.newFrame();
    imgui.impl_sdl2.newFrame();
    c.igNewFrame();
}

pub fn frameDraw(_: *Core.C) callconv(.C) void {
    imgui.impl_gl3.renderDrawData();
}

pub fn frameFinish(_: *Core.C) callconv(.C) void {
    c.igRender();
}

pub fn layout(ccx: *Core.C) callconv(.C) void {
    var cx = ccx.core();

    if (!ccx.devgui_open) {
        return;
    }

    if ((c.SDL_GetWindowFlags(c.sdl_window) & c.SDL_WINDOW_MINIMIZED) != 0) {
        return;
    }

    if (!c.igBeginMainMenuBar()) {
        return;
    }

    c.igPushStyleColor_Vec4(c.ImGuiCol_MenuBarBg, c.ImVec4{
        .x = 0.0,
        .y = 0.0,
        .z = 0.0,
        .w = 0.66,
    });
    c.igPushStyleColor_Vec4(c.ImGuiCol_WindowBg, c.ImVec4{
        .x = 0.0,
        .y = 0.0,
        .z = 0.0,
        .w = 0.66,
    });

    defer {
        c.igPopStyleColor(2);
        c.igEndMainMenuBar();
    }

    c.igTextUnformatted("Developer Tools", null);
    c.igSeparator();

    if (c.igMenuItem_Bool("Close", null, false, true)) {
        ccx.devgui_open = false;
    }

    const mainvp = c.igGetMainViewport() orelse {
        imgui.report_err_get_main_viewport.call();
        return;
    };

    c.igPushItemWidth(mainvp.*.Size.x * 0.15);

    const items = [_][*:0]const u8{
        "Console",
        "Music",
        "Prefs",
        "VFS",
    };

    comptime std.debug.assert(items.len == std.enums.values(State).len);

    if (c.igCombo_Str_arr("Left", @ptrCast(&cx.dgui.left), &items, items.len, -1)) {
        // ImGui misbehaves if both sides of the developer GUI draw the same tool.
        if (cx.dgui.left == cx.dgui.right) {
            inline for (@typeInfo(State).Enum.fields) |i| {
                const e = @as(State, @enumFromInt(i.value));

                if (cx.dgui.left != e) {
                    cx.dgui.right = e;
                }
            }
        }
    }

    if (c.igCombo_Str_arr("Right", @ptrCast(&cx.dgui.right), &items, items.len, -1)) {
        if (cx.dgui.left == cx.dgui.right) {
            inline for (@typeInfo(State).Enum.fields) |i| {
                const e = @as(State, @enumFromInt(i.value));

                if (cx.dgui.right != e) {
                    cx.dgui.left = e;
                }
            }
        }
    }

    c.igPopItemWidth();
    const menu_bar_height = c.igGetWindowHeight();

    switch (cx.dgui.left) {
        .console => Console.layout(cx, true, menu_bar_height),
        .music => MusicGui.layout(cx, true, menu_bar_height),
        .prefs => PrefGui.layout(cx, true, menu_bar_height),
        .vfs => VfsGui.layout(cx, true, menu_bar_height),
    }

    switch (cx.dgui.right) {
        .console => Console.layout(cx, false, menu_bar_height),
        .music => MusicGui.layout(cx, false, menu_bar_height),
        .prefs => PrefGui.layout(cx, false, menu_bar_height),
        .vfs => VfsGui.layout(cx, false, menu_bar_height),
    }

    c.igPushItemWidth(mainvp.*.Size.x * 0.15);

    if (c.igBeginCombo("ImGui", "Metrics, etc...", 0)) {
        defer c.igEndCombo();
        _ = c.igCheckbox("About", &cx.dgui.about_window);
        _ = c.igCheckbox("Debug Log", &cx.dgui.debug_log);
        _ = c.igCheckbox("Demo", &cx.dgui.demo_window);
        _ = c.igCheckbox("ID Stack Tool", &cx.dgui.id_stack_tool_window);
        _ = c.igCheckbox("Metrics", &cx.dgui.metrics_window);
        _ = c.igCheckbox("User Guide", &cx.dgui.user_guide);
    }

    c.igPopItemWidth();

    if (builtin.mode == .Debug) {
        c.igSeparator();
        imgui.textUnformatted("DEBUG BUILD");
    }

    if (cx.dgui.demo_window) {
        c.igShowDemoWindow(&cx.dgui.demo_window);
    }

    if (cx.dgui.metrics_window) {
        c.igShowMetricsWindow(&cx.dgui.metrics_window);
    }

    if (cx.dgui.debug_log) {
        c.igShowDebugLogWindow(&cx.dgui.debug_log);
    }

    if (cx.dgui.id_stack_tool_window) {
        c.igShowIDStackToolWindow(&cx.dgui.id_stack_tool_window);
    }

    if (cx.dgui.about_window) {
        c.igShowAboutWindow(&cx.dgui.about_window);
    }
}

pub fn processEvent(_: *Core.C, event: *const c.SDL_Event) callconv(.C) bool {
    return imgui.impl_sdl2.processEvent(event);
}

pub fn setup(ccx: *Core.C, window: *c.SDL_Window, sdl_gl_ctx: *anyopaque) callconv(.C) void {
    ccx.imgui_ctx = c.igCreateContext(null) orelse {
        c.I_Error("Failed to create ImGui context");
    };

    const io = c.igGetIO();
    io.*.ConfigFlags |= c.ImGuiConfigFlags_NavEnableKeyboard;

    if (!c.ImGui_ImplSDL2_InitForOpenGL(window, sdl_gl_ctx)) {
        c.I_Error("Failed to initialize ImGui SDL2 backend for OpenGL3");
    }

    if (!c.ImGui_ImplOpenGL3_Init(null)) {
        c.I_Error("Failed to initialize ImGui OpenGL3 backend");
    }

    c.igStyleColorsDark(null);
}

pub fn shutdown() callconv(.C) void {
    const im_ctx = c.igGetCurrentContext() orelse return;

    imgui.impl_gl3.shutdown();
    imgui.impl_sdl2.shutdown();
    c.igDestroyContext(im_ctx);
}

pub fn wantsKeyboard(_: *Core.C) callconv(.C) bool {
    return c.igGetIO().*.WantCaptureKeyboard;
}

pub fn wantsMouse(_: *Core.C) callconv(.C) bool {
    return c.igGetIO().*.WantCaptureMouse;
}
