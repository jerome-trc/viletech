const builtin = @import("builtin");
const std = @import("std");
const log = std.log.scoped(.frontend);

const c = @import("main.zig").c;

const Core = @import("Core.zig");
const Game = @import("Game.zig");
const imgui = @import("imgui.zig");
const Path = @import("stdx.zig").Path;

const Self = @This();

const ItemArray = std.ArrayList(Item);

/// What the caller should do after having drawn a frontend frame.
pub const Outcome = enum {
    none,
    exit,
    start_game,
};

pub const Item = struct {
    /// Will always be absolute.
    path: Path,
    enabled: bool,
};

allo: std.mem.Allocator,
absolute_paths: bool,
game_rules: Game.Rules,
load_order: ItemArray,
modal_open: bool,

pub fn init(allocator: std.mem.Allocator) !Self {
    return Self{
        .allo = allocator,
        .absolute_paths = false,
        .game_rules = Game.Rules{ .compat = Game.Compat.mbf21, .skill = .l4 },
        .load_order = ItemArray.init(allocator),
        .modal_open = false,
    };
}

pub fn deinit(self: *Self) void {
    self.load_order.deinit();
}

pub fn addToLoadOrder(self: *Self, path: []const u8) !void {
    try self.load_order.append(Item{
        .path = try self.allo.dupeZ(u8, path),
        .enabled = true,
    });
}

pub fn draw(cx: *Core) Outcome {
    var self = &cx.scene.frontend;

    if (!c.igBegin("Launcher", null, c.ImGuiWindowFlags_MenuBar)) {
        return Outcome.none;
    }

    defer c.igEnd();
    imgui.textUnformatted("Hint: drag-and-drop files here to add them to the load order.");

    if (c.igBeginMenuBar()) {
        defer c.igEndMenuBar();

        if (c.igButton("Exit", .{ .x = 0.0, .y = 0.0 })) {
            return Outcome.exit;
        }

        if (c.igButton("Start", .{ .x = 0.0, .y = 0.0 })) {
            // Check to make sure all files in load order exist.
            for (self.load_order.items) |*item| {
                if (std.fs.accessAbsoluteZ(item.path, .{})) |_| {} else |_| {
                    if (c.igBeginPopupModal(
                        "Error",
                        &self.modal_open,
                        c.ImGuiWindowFlags_None,
                    )) {
                        defer c.igEndPopup();
                        // TODO: full message has to be allocated somewhere...
                        imgui.textUnformatted("File does not exist: ");
                    }

                    return Outcome.none;
                }
            }

            return Outcome.start_game;
        }
    }

    const cra = imgui.contentRegionAvail();

    if (c.igBeginChild_Str(
        "frontend.files",
        .{ .x = cra.x / 2.0, .y = 0.0 },
        c.ImGuiChildFlags_None,
        c.ImGuiWindowFlags_None,
    )) {
        _ = c.igCheckbox("Absolute Paths", &self.absolute_paths);

        if (c.igBeginTable(
            "frontend.files.table",
            2,
            c.ImGuiTableFlags_RowBg | c.ImGuiTableFlags_Borders | c.ImGuiTableColumnFlags_WidthFixed,
            .{ .x = -1.0, .y = 0.0 },
            0.0,
        )) {
            defer c.igEndTable();

            c.igTableSetupColumn(
                "##path",
                c.ImGuiTableColumnFlags_WidthFixed,
                imgui.contentRegionAvail().x * 0.8,
                0,
            );
            c.igTableSetupColumn(
                "##controls",
                c.ImGuiTableColumnFlags_WidthFixed,
                0.0,
                0,
            );

            var popupShown = false;

            for (0.., self.load_order.items) |i, *item| {
                c.igTableNextRow(c.ImGuiTableRowFlags_None, 0.0);

                _ = c.igTableSetColumnIndex(0);

                if (self.absolute_paths) {
                    _ = c.igCheckbox(@ptrCast(item.path), &item.enabled);
                } else {
                    const basename = std.fs.path.basename(item.path);
                    _ = c.igCheckbox(@ptrCast(basename), &item.enabled);
                }

                if (!popupShown and c.igBeginPopupContextItem(
                    "loadorder.popup",
                    c.ImGuiPopupFlags_MouseButtonRight,
                )) {
                    defer c.igEndPopup();
                    popupShown = true;

                    if (c.igButton("Show in File Explorer", .{ .x = 0.0, .y = 0.0 })) {
                        openDirInFileExplorer(self, item.path) catch {};
                    }
                }

                c.igSetItemTooltip("Enabled");

                _ = c.igTableSetColumnIndex(1);

                if (c.igButton("X", .{ .x = 16.0, .y = 0.0 })) {
                    const removed = self.load_order.orderedRemove(i);
                    self.allo.free(removed.path);
                    break;
                }
                c.igSetItemTooltip("Remove");

                c.igSameLine(0.0, -1.0);
                c.igBeginDisabled(i == 0);

                if (c.igArrowButton("up", c.ImGuiDir_Up)) {
                    std.mem.swap(Item, &self.load_order.items[i - 1], &self.load_order.items[i]);
                }
                c.igSetItemTooltip("Up");

                c.igEndDisabled();
                c.igSameLine(0.0, -1.0);
                c.igBeginDisabled(i == (self.load_order.items.len - 1));

                if (c.igArrowButton("down", c.ImGuiDir_Down)) {
                    std.mem.swap(Item, &self.load_order.items[i + 1], &self.load_order.items[i]);
                }
                c.igSetItemTooltip("Down");

                c.igEndDisabled();
            }
        }
    }
    c.igEndChild();

    c.igSameLine(0.0, -1.0);

    if (c.igBeginChild_Str(
        "frontend.options",
        .{ .x = cra.x / 2.0, .y = 0.0 },
        c.ImGuiChildFlags_None,
        c.ImGuiWindowFlags_None,
    )) {
        c.igSetNextItemOpen(true, c.ImGuiCond_None);

        if (c.igTreeNode_Str("Game Options")) {
            defer c.igTreePop();

            imgui.textUnformatted("Difficulty");

            // TODO: different text if Heretic/Hexen/etc.
            if (c.igRadioButton_Bool("I'm too young to die.", self.game_rules.skill == .l1)) {
                self.game_rules.skill = .l1;
            }
            if (c.igRadioButton_Bool("Hey, not too rough.", self.game_rules.skill == .l2)) {
                self.game_rules.skill = .l2;
            }
            if (c.igRadioButton_Bool("Hurt me plenty.", self.game_rules.skill == .l3)) {
                self.game_rules.skill = .l3;
            }
            if (c.igRadioButton_Bool("Ultra-Violence.", self.game_rules.skill == .l4)) {
                self.game_rules.skill = .l4;
            }
            if (c.igRadioButton_Bool("Nightmare!", self.game_rules.skill == .l5)) {
                self.game_rules.skill = .l5;
            }

            if (c.igBeginCombo(
                "Compatibility Level",
                self.game_rules.compat.prettyName(),
                c.ImGuiComboFlags_None,
            )) {
                defer c.igEndCombo();

                inline for (std.meta.fields(Game.Compat)) |compat| {
                    const e: Game.Compat = @enumFromInt(compat.value);
                    const name = e.prettyName();

                    if (name.len != 0) {
                        if (c.igSelectable_Bool(
                            name,
                            self.game_rules.compat == e,
                            c.ImGuiSelectableFlags_None,
                            .{ .x = 0.0, .y = 0.0 },
                        )) {
                            self.game_rules.compat = e;
                        }
                    }
                }
            }
        }
    }
    c.igEndChild();

    return Outcome.none;
}

fn openDirInFileExplorer(self: *Self, path: []const u8) !void {
    var d: std.fs.Dir = undefined;
    var dir = path;
    const dirOpenOpts = .{ .no_follow = true };

    // Try to open `path` as a directory. If this fails, it's probably a WAD or
    // similar, so try to open the directory it's in.
    d = std.fs.openDirAbsolute(dir, dirOpenOpts) catch blk: {
        dir = std.fs.path.dirname(path) orelse {
            log.err("Failed to get directory of path: {s}", .{path});
            return;
        };

        d = std.fs.openDirAbsolute(dir, dirOpenOpts) catch {
            log.err("Failed to open directory: {s}", .{dir});
            return;
        };
        break :blk d;
    };

    d.close();

    switch (builtin.os.tag) {
        .linux => {
            var proc = std.process.Child.init(&[_][]const u8{ "xdg-open", dir }, self.allo);
            _ = try proc.spawnAndWait();
        },
        else => @compileError("unsupported OS"),
    }
}
