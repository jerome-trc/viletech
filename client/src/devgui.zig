//! Abstractions over ImGui for assembling the developer GUI menu bar and windows.

const c = @import("root").c;

const Core = @import("Core.zig");
const Display = @import("platform.zig").Display;

pub const State = enum(c_int) {
    console,
    vfs,
};

pub fn draw(cx: *Core, display: *Display) void {
    if (!display.dgui.open) {
        return;
    }

    if (display.dgui.demo_window) {
        c.igShowDemoWindow(&display.dgui.demo_window);
    }

    if (display.dgui.metrics_window) {
        c.igShowMetricsWindow(&display.dgui.metrics_window);
    }

    if (display.dgui.debug_log) {
        c.igShowDebugLogWindow(&display.dgui.debug_log);
    }

    if (display.dgui.id_stack_tool_window) {
        c.igShowIDStackToolWindow(&display.dgui.id_stack_tool_window);
    }

    if (display.dgui.about_window) {
        c.igShowAboutWindow(&display.dgui.about_window);
    }

    if (c.igBeginMainMenuBar()) {
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

        if (c.igMenuItem_Bool("Close", null, false, false)) {
            display.dgui.open = false;
        }

        if (c.igGetMainViewport()) |vp| {
            c.igPushItemWidth(vp.*.Size.x * 0.15);
        }

        const items = [2][*c]const u8{
            "Console",
            "VFS",
        };

        if (c.igCombo_Str_arr("Left", @ptrCast(&display.dgui.left), &items, items.len, -1)) {
            // ImGui misbehaves if both sides of the developer GUI draw the same tool.
            if (display.dgui.left == display.dgui.right) {
                inline for (@typeInfo(State).Enum.fields) |i| {
                    const e = @as(State, @enumFromInt(i.value));

                    if (display.dgui.left != e) {
                        display.dgui.right = e;
                    }
                }
            }
        }

        if (c.igCombo_Str_arr("Right", @ptrCast(&display.dgui.right), &items, items.len, -1)) {
            if (display.dgui.left == display.dgui.right) {
                inline for (@typeInfo(State).Enum.fields) |i| {
                    const e = @as(State, @enumFromInt(i.value));

                    if (display.dgui.right != e) {
                        display.dgui.left = e;
                    }
                }
            }
        }

        switch (display.dgui.left) {
            .console => {}, // TODO
            .vfs => {}, // TODO
        }

        switch (display.dgui.right) {
            .console => {}, // TODO
            .vfs => {}, // TODO
        }

        if (c.igBeginCombo("ImGui", "Metrics, etc...", 0)) {
            defer c.igEndCombo();
            _ = c.igCheckbox("About", &display.dgui.about_window);
            _ = c.igCheckbox("Debug Log", &display.dgui.debug_log);
            _ = c.igCheckbox("Demo", &display.dgui.demo_window);
            _ = c.igCheckbox("ID Stack Tool", &display.dgui.id_stack_tool_window);
            _ = c.igCheckbox("Metrics", &display.dgui.metrics_window);
            _ = c.igCheckbox("User Guide", &display.dgui.user_guide);
        }

        c.igPopItemWidth();
    }

    _ = cx;
}
