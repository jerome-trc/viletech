//! A developer GUI for visualizing the state of the virtual file system.

const std = @import("std");

const c = @import("../main.zig").c;

const Core = @import("../Core.zig");
const imgui = @import("../imgui.zig");
const stdx = @import("../stdx.zig");

const Self = @This();

filter_buf: [256]u8,
filter_case_sensitive: bool,

pub fn init() Self {
    return Self{
        .filter_buf = [_]u8{0} ** 256,
        .filter_case_sensitive = false,
    };
}

pub fn layout(cx: *Core, left: bool, menu_bar_height: f32) void {
    var self = &cx.vfsgui;

    const vp_size = if (c.igGetMainViewport()) |vp| vp.*.Size else {
        imgui.report_err_get_main_viewport.call();
        return;
    };

    if (left) {
        c.igSetNextWindowPos(.{ .x = 0.0, .y = menu_bar_height }, c.ImGuiCond_None, .{});
    } else {
        c.igSetNextWindowPos(.{ .x = vp_size.x * 0.5, .y = menu_bar_height }, c.ImGuiCond_None, .{});
    }

    c.igSetNextWindowSize(.{ .x = vp_size.x * 0.5, .y = vp_size.y * 0.33 }, c.ImGuiCond_None);

    if (!c.igBegin("VFS", null, c.ImGuiWindowFlags_NoTitleBar | c.ImGuiWindowFlags_NoResize)) {
        return;
    }

    defer c.igEnd();

    if (imgui.inputText("Filter##vfsgui.filter", self.filterBufSlice(), .{}, null, null)) {}
    c.igSameLine(0.0, -1.0);
    _ = c.igCheckbox("aA##vfsgui.filter_case_sensitive", &self.filter_case_sensitive);

    if (self.filter_case_sensitive) {
        c.igSetItemTooltip("Filtering: Case Sensitively");
    } else {
        c.igSetItemTooltip("Filtering: Case Insensitively");
    }

    if (c.igBeginTable(
        "vfsgui.files",
        2,
        c.ImGuiTableFlags_RowBg | c.ImGuiTableFlags_Borders | c.ImGuiTableColumnFlags_WidthFixed,
        .{ .x = -1.0, .y = 0.0 },
        0.0,
    )) scroll: {
        defer c.igEndTable();

        c.igTableSetupColumn(
            "##path",
            c.ImGuiTableColumnFlags_WidthFixed,
            imgui.contentRegionAvail().x * 0.8,
            0,
        );
        c.igTableSetupColumn(
            "##size",
            c.ImGuiTableColumnFlags_WidthFixed,
            0.0,
            0,
        );

        const num_entries = std.math.lossyCast(usize, c.numlumps);

        const clipper = imgui.Clipper.init() catch {
            imgui.report_err_clipper_ctor.call();
            break :scroll;
        };
        defer clipper.deinit();
        clipper.begin(num_entries, 16.0);

        var popup_shown = false;

        while (clipper.step()) {
            var i = clipper.displayStart();
            var l = i;

            while ((i < clipper.displayEnd()) and (l < num_entries)) {
                defer l += 1;

                const lmp: c.LumpNum = @intCast(l);
                const entryName = std.mem.sliceTo(c.W_LumpName(lmp).?, 0);

                const filter = std.mem.sliceTo(&self.filter_buf, 0);

                const filter_find = if (self.filter_case_sensitive)
                    std.mem.indexOf(u8, entryName, filter)
                else
                    std.ascii.indexOfIgnoreCase(entryName, filter);

                if (filter_find) |_| {} else if (filter.len < 1) {} else {
                    continue;
                }

                c.igTableNextRow(c.ImGuiTableRowFlags_None, 0.0);
                defer i += 1;

                _ = c.igTableSetColumnIndex(0);
                imgui.textUnformatted(entryName);
                tryContextMenu(cx, &popup_shown, lmp);

                _ = c.igTableSetColumnIndex(1);

                const size = c.W_LumpLength(lmp);

                if (size == 0) {
                    imgui.textUnformatted("0 B");
                    continue;
                }

                var s: f32 = @floatFromInt(size);
                var unit: [:0]const u8 = "B";

                if (s > 1024.0) {
                    s /= 1024.0;
                    unit = "kB";
                } else {
                    c.igText("%.2f %s", s, unit.ptr);
                    continue;
                }

                if (s > 1024.0) {
                    s /= 1024.0;
                    unit = "MB";
                }

                if (s > 1024.0) {
                    s /= 1024.0;
                    unit = "GB";
                }

                c.igText("%.2f %s", s, unit.ptr);
            }
        }
    }
}

fn filterBufSlice(self: *Self) [:0]u8 {
    return self.filter_buf[0..(@sizeOf(@TypeOf(self.filter_buf)) - 1) :0];
}

fn tryContextMenu(cx: *Core, popup_shown: *bool, lump: c.LumpNum) void {
    if (!c.igBeginPopupContextItem("##vfs.context", c.ImGuiPopupFlags_MouseButtonRight)) {
        return;
    }

    defer c.igEndPopup();
    const lump_len = c.W_LumpLength(lump);

    if (popup_shown.* or lump_len <= 0) {
        return;
    }

    popup_shown.* = true;

    if (lump_len < 4) {
        return;
    }

    var magic4: [4]u8 = undefined;
    c.W_ReadLumpN(lump, &magic4, 4);
    const magic = std.mem.bytesToValue(u32, &magic4);

    var is_music = false;
    is_music = is_music or (magic == stdx.asciiId('M', 'U', 'S', 0x1A));
    is_music = is_music or (magic == stdx.asciiId('M', 'T', 'h', 'd'));
    is_music = is_music or (magic == stdx.asciiId('R', 'I', 'F', 'F'));
    is_music = is_music or (magic == stdx.asciiId('M', 'I', 'D', 'S'));
    // TODO: whatever raw formats dsda-doom supports.

    if (is_music and c.igButton("Play Music", .{ .x = 0.0, .y = 0.0 })) {
        c.S_ChangeMusInfoMusic(@ptrCast(&cx.c), lump, @intFromBool(true));
    }
}
