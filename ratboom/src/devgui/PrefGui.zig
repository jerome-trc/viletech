const std = @import("std");

const c = @import("../main.zig").c;

const Core = @import("../Core.zig");
const imgui = @import("../imgui.zig");

pub fn layout(cx: *Core, left: bool, menu_bar_height: f32) void {
    const vp_size = if (c.igGetMainViewport()) |vp| vp.*.Size else {
        imgui.report_err_get_main_viewport.call();
        return;
    };

    if (left) {
        c.igSetNextWindowPos(.{ .x = 0.0, .y = menu_bar_height }, c.ImGuiCond_None, .{});
    } else {
        c.igSetNextWindowPos(
            .{ .x = vp_size.x * 0.5, .y = menu_bar_height },
            c.ImGuiCond_None,
            .{},
        );
    }

    c.igSetNextWindowSize(
        .{ .x = vp_size.x * 0.5, .y = vp_size.y * 0.33 },
        c.ImGuiCond_None,
    );

    if (!c.igBegin(
        "Preferences",
        null,
        c.ImGuiWindowFlags_NoTitleBar | c.ImGuiWindowFlags_NoResize,
    )) return;

    defer c.igEnd();

    if (c.igBeginTable(
        "vfsgui.files",
        3,
        c.ImGuiTableFlags_RowBg | c.ImGuiTableFlags_Borders | c.ImGuiTableColumnFlags_WidthFixed,
        .{ .x = -1.0, .y = 0.0 },
        0.0,
    )) {
        defer c.igEndTable();

        c.igTableSetupColumn(
            "##name",
            c.ImGuiTableColumnFlags_WidthFixed,
            imgui.contentRegionAvail().x * 0.45,
            0,
        );
        c.igTableSetupColumn(
            "##type",
            c.ImGuiTableColumnFlags_WidthFixed,
            imgui.contentRegionAvail().x * 0.1,
            0,
        );
        c.igTableSetupColumn(
            "##value",
            c.ImGuiTableColumnFlags_WidthFixed,
            imgui.contentRegionAvail().x * 0.45,
            0,
        );

        var iter = cx.prefs.iterator();

        while (iter.next()) |kv| {
            c.igTableNextRow(c.ImGuiTableRowFlags_None, 0.0);
            _ = c.igTableSetColumnIndex(0);
            imgui.textUnformatted(kv.key_ptr.*);
            _ = c.igTableSetColumnIndex(1);
            imgui.textUnformatted(@tagName(kv.value_ptr.*));
            _ = c.igTableSetColumnIndex(2);

            switch (kv.value_ptr.*) {
                .boolean => |v| imgui.textUnformatted(if (v) "true" else "false"),
                .float => |v| c.igText("%f", v),
                .int => |v| c.igText("%li", v),
                .string => |v| imgui.textUnformatted(v),
            }
        }
    }
}
