//! Wrapper around cImGui's API for more ergonomic use by Zig code.

const std = @import("std");

const c = @import("root").c;
const sdl = @import("sdl2");

const platform = @import("platform.zig");

pub const Error = error{
    ContextCreateFail,
};

pub const Context = c.ImGuiContext;

pub const implSdl2 = struct {
    pub fn initForSdlRenderer(window: sdl.Window, renderer: sdl.Renderer) bool {
        return c.ImGui_ImplSDL2_InitForSDLRenderer(@ptrCast(window.ptr), @ptrCast(renderer.ptr));
    }

    pub fn newFrame() void {
        return c.ImGui_ImplSDL2_NewFrame();
    }

    pub fn processEvent(event: *const c.SDL_Event) bool {
        return c.ImGui_ImplSDL2_ProcessEvent(event);
    }

    pub fn shutdown() void {
        return c.ImGui_ImplSDL2_Shutdown();
    }
};

pub const implSdlRenderer2 = struct {
    pub fn init(renderer: sdl.Renderer) bool {
        return c.ImGui_ImplSDLRenderer2_Init(@ptrCast(renderer.ptr));
    }

    pub fn newFrame() void {
        c.ImGui_ImplSDLRenderer2_NewFrame();
    }

    pub fn renderDrawData(draw_data: [*c]c.ImDrawData, renderer: sdl.Renderer) void {
        c.ImGui_ImplSDLRenderer2_RenderDrawData(draw_data, @ptrCast(renderer.ptr));
    }

    pub fn shutdown() void {
        c.ImGui_ImplSDLRenderer2_Shutdown();
    }
};
