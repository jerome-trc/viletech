//! Abstractions over ImGui for assembling the developer GUI menu bar and windows.

const std = @import("std");
const log = std.log.scoped(.devgui);

const c = @import("root").c;

const Console = @import("devgui/Console.zig");
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

        if (c.igMenuItem_Bool("Close", null, false, true)) {
            display.dgui.open = false;
        }

        const mainvp = c.igGetMainViewport() orelse {
            reportErrClipperCtor.call();
            return;
        };

        c.igPushItemWidth(mainvp.*.Size.x * 0.15);

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

        c.igPopItemWidth();
        const menu_bar_height = c.igGetWindowHeight();

        switch (display.dgui.left) {
            .console => Console.draw(cx, true, menu_bar_height),
            .vfs => {}, // TODO
        }

        switch (display.dgui.right) {
            .console => Console.draw(cx, false, menu_bar_height),
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
    }
}

pub var reportErrGetMainViewport = std.once(doReportErrGetMainViewport);

fn doReportErrGetMainViewport() void {
    log.err("`igGetMainViewport` failed", .{});
}

pub var reportErrClipperCtor = std.once(doReportErrClipperCtor);

fn doReportErrClipperCtor() void {
    log.err("`ImGuiListClipper::ImGuiListClipper` failed", .{});
}
