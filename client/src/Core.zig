const c = @import("main.zig").c;

const devgui = @import("devgui.zig");

const Self = @This();

pub const C = extern struct {
    core: *Self,
    devgui_open: bool,
    imgui_ctx: *c.ImGuiContext,
    saved_gametick: i32,
};

pub const DevGui = struct {
    left: devgui.State,
    right: devgui.State,

    demo_window: bool = false,
    metrics_window: bool = false,
    debug_log: bool = false,
    id_stack_tool_window: bool = false,
    about_window: bool = false,
    user_guide: bool = false,
};

c: C,
dgui: DevGui,
