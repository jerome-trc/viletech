//! Wrapper around cImGui's API for more ergonomic use by Zig code.

const std = @import("std");
const log = std.log.scoped(.imgui);

const c = @import("main.zig").c;
const sdl = @import("sdl2");

const platform = @import("platform.zig");

pub const Error = error{
    ClipperInitFail,
    ContextCreateFail,
};

pub const Context = c.ImGuiContext;

pub const impl_sdl2 = struct {
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

pub const impl_sdlrenderer2 = struct {
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

pub const Clipper = packed struct {
    const Self = @This();

    ptr: [*c]c.ImGuiListClipper,

    pub fn init() !Self {
        return Self{ .ptr = c.ImGuiListClipper_ImGuiListClipper() orelse return error.ClipperInitFail };
    }

    pub fn deinit(self: Self) void {
        c.ImGuiListClipper_destroy(self.ptr);
    }

    pub fn begin(self: Self, num_items: usize, items_height: f32) void {
        c.ImGuiListClipper_Begin(
            self.ptr,
            std.math.lossyCast(c_int, num_items),
            items_height,
        );
    }

    pub fn step(self: Self) bool {
        return c.ImGuiListClipper_Step(self.ptr);
    }

    pub fn displayStart(self: Self) usize {
        return @intCast(self.ptr.*.DisplayStart);
    }

    pub fn displayEnd(self: Self) usize {
        return @intCast(self.ptr.*.DisplayEnd);
    }
};

pub const Vec2 = extern struct { x: f32 = 0.0, y: f32 = 0.0 };

pub fn contentRegionAvail() Vec2 {
    var ret: Vec2 = .{};
    c.igGetContentRegionAvail(@ptrCast(&ret));
    return ret;
}

pub fn inputText(
    label: [*:0]const u8,
    buf: [:0]u8,
    flags: InputTextFlags,
    callback: c.ImGuiInputTextCallback,
    user_data: ?*anyopaque,
) bool {
    return c.igInputText(
        label,
        @ptrCast(buf.ptr),
        buf.len,
        @bitCast(flags),
        callback,
        user_data,
    );
}

pub const InputTextFlags = packed struct(i32) {
    chars_decimal: bool = false,
    chars_hexadecimal: bool = false,
    chars_scientific: bool = false,
    chars_uppercase: bool = false,
    chars_no_blank: bool = false,
    allow_tab_input: bool = false,
    enter_returns_true: bool = false,
    escape_clears_all: bool = false,
    ctrl_enter_for_new_line: bool = false,
    read_only: bool = false,
    password: bool = false,
    always_overwrite: bool = false,
    auto_select_all: bool = false,
    parse_empty_ref_val: bool = false,
    display_empty_ref_val: bool = false,
    no_horizontal_scroll: bool = false,
    no_undo_redo: bool = false,
    callback_completion: bool = false,
    callback_history: bool = false,
    callback_always: bool = false,
    callback_char_filter: bool = false,
    callback_resize: bool = false,
    callback_edit: bool = false,

    _padding: u9 = 0,
};

pub fn pushStyleCompact() void {
    const style = c.igGetStyle();

    c.igPushStyleVar_Vec2(c.ImGuiStyleVar_FramePadding, .{
        .x = style.*.FramePadding.x,
        .y = @round(style.*.FramePadding.y * 0.6),
    });
    c.igPushStyleVar_Vec2(c.ImGuiStyleVar_ItemSpacing, .{
        .x = style.*.ItemSpacing.x,
        .y = @round(style.*.ItemSpacing.y * 0.6),
    });
}

pub fn popStyleCompact() void {
    c.igPopStyleVar(2);
}

pub fn textUnformatted(text: []const u8) void {
    c.igTextUnformatted(text.ptr, text.ptr + text.len);
}

// One-off error reporting /////////////////////////////////////////////////////

pub var reportErrGetMainViewport = std.once(doReportErrGetMainViewport);

fn doReportErrGetMainViewport() void {
    log.err("`igGetMainViewport` failed", .{});
}

pub var reportErrClipperCtor = std.once(doReportErrClipperCtor);

fn doReportErrClipperCtor() void {
    log.err("`ImGuiListClipper::ImGuiListClipper` failed", .{});
}
