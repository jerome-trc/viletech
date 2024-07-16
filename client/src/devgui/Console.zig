const std = @import("std");
const log = std.log.scoped(.devgui);

const c = @import("../main.zig").c;

const ccmds = @import("ccmds.zig");
const Core = @import("../Core.zig");
const Deque = @import("../deque.zig").Deque;
const devgui = @import("../devgui.zig");
const imgui = @import("../imgui.zig");

const Self = @This();

pub const CommandArgs = std.process.ArgIteratorGeneral(.{
    .comments = false,
    .single_quotes = true,
});

pub const Command = struct {
    name: []const u8,
    func: *const fn (*Core, *const Command, *CommandArgs) void,
};

const HistoryItem = union(enum) {
    info: []const u8,
    submission: []const u8,
    toast: []const u8,
};

const commands = std.StaticStringMap(Command).initComptime(.{
    .{ "exit", Command{ .name = "exit", .func = &ccmds.exit } },
    .{ "quit", Command{ .name = "quit", .func = &ccmds.exit } },
});

comptime {
    for (commands.keys(), commands.values()) |k, v| {
        std.debug.assert(std.mem.eql(u8, k, v.name));
    }
}

allo: std.mem.Allocator,
input_buf: [256]u8,
history: Deque(HistoryItem),
prev_inputs: Deque([]const u8),

pub fn init(allocator: std.mem.Allocator) !Self {
    return Self{
        .allo = allocator,
        .input_buf = [_]u8{0} ** 256,
        .history = try Deque(HistoryItem).init(allocator),
        .prev_inputs = try Deque([]const u8).init(allocator),
    };
}

pub fn deinit(self: *Self) void {
    self.history.deinit();
    self.prev_inputs.deinit();
}

pub fn draw(cx: *Core, left: bool, menu_bar_height: f32) void {
    var self = &cx.console;

    const vp_size = if (c.igGetMainViewport()) |vp| vp.*.Size else {
        imgui.reportErrGetMainViewport.call();
        return;
    };

    if (left) {
        c.igSetNextWindowPos(.{ .x = 0.0, .y = menu_bar_height }, c.ImGuiCond_None, .{});
    } else {
        c.igSetNextWindowPos(.{ .x = vp_size.x * 0.5, .y = menu_bar_height }, c.ImGuiCond_None, .{});
    }

    c.igSetNextWindowSize(.{ .x = vp_size.x * 0.5, .y = vp_size.y * 0.33 }, c.ImGuiCond_None);

    if (!c.igBegin("Console", null, c.ImGuiWindowFlags_NoTitleBar | c.ImGuiWindowFlags_NoResize)) {
        return;
    }

    defer c.igEnd();

    const footer_height_to_reserve = c.igGetStyle().*.ItemSpacing.y + c.igGetFrameHeightWithSpacing();

    scroll: {
        if (c.igBeginChild_Str("console.scroll", .{
            .x = 0.0,
            .y = -footer_height_to_reserve,
        }, c.ImGuiChildFlags_None, c.ImGuiWindowFlags_HorizontalScrollbar)) {
            defer c.igEndChild();

            const clipper = imgui.Clipper.init() catch {
                imgui.reportErrClipperCtor.call();
                break :scroll;
            };
            defer clipper.deinit();
            clipper.begin(self.history.len(), -1.0);

            while (clipper.step()) {
                for (clipper.displayStart()..clipper.displayEnd()) |i| {
                    switch ((self.history.get(i) orelse unreachable).*) {
                        HistoryItem.info => |str| {
                            c.igTextUnformatted(str.ptr, str.ptr + str.len);
                        },
                        HistoryItem.submission => |str| {
                            c.igTextUnformatted(str.ptr, str.ptr + str.len);
                        },
                        HistoryItem.toast => |str| {
                            c.igTextUnformatted(str.ptr, str.ptr + str.len);
                        },
                    }
                }
            }
        }
    }

    if (imgui.inputText("##console.input", self.inputBufSlice(), .{
        .callback_completion = true,
        .callback_history = true,
        .enter_returns_true = true,
    }, inputTextCallback, null)) {
        submit(cx);
    }

    c.igSameLine(0.0, -1.0);

    if (c.igButton("Submit", .{ .x = 0.0, .y = 0.0 })) {
        submit(cx);
    }

    c.igSetItemDefaultFocus();
}

pub fn logHelp(self: *Self, comptime format: []const u8, args: anytype) void {
    errdefer reportConsoleHistoryFail.call();
    const p = std.fmt.allocPrint(self.allo, format, args) catch return;
    self.history.pushBack(HistoryItem{ .info = p }) catch return;
}

pub fn logInfo(cx: *Core, comptime format: []const u8, args: anytype) void {
    errdefer reportConsoleHistoryFail.call();
    cx.eprintln(format, args) catch return;
    const p = std.fmt.allocPrint(cx.console.allo, format, args) catch return;
    cx.console.history.pushBack(HistoryItem{ .info = p }) catch return;
}

pub fn logSubmission(cx: *Core, comptime format: []const u8, args: anytype) void {
    errdefer reportConsoleHistoryFail.call();
    cx.eprintln(format, args) catch return;
    const p = std.fmt.allocPrint(cx.console.allo, format, args) catch return;
    cx.console.history.pushBack(HistoryItem{ .submission = p }) catch return;
}

pub fn logToast(cx: *Core, comptime format: []const u8, args: anytype) void {
    errdefer reportConsoleHistoryFail.call();
    cx.eprintln(format, args) catch return;
    const p = std.fmt.allocPrint(cx.console.allo, format, args) catch return;
    cx.console.history.pushBack(HistoryItem{ .toast = p }) catch return;
}

/// Allows the console to fulfill the interface of a writer like stdout.
pub fn print(self: *Self, comptime format: []const u8, args: anytype) !void {
    self.logHelp(format, args);
}

fn inputTextCallback(data: [*c]c.ImGuiInputTextCallbackData) callconv(.C) c_int {
    _ = data;
    return 0;
}

fn submit(cx: *Core) void {
    var self = &cx.console;
    const submission = std.mem.sliceTo(&self.input_buf, 0);

    if (submission.len < 1) {
        logSubmission(cx, "$", .{});
        return;
    }

    if (self.prev_inputs.len() > 256) {
        const s = self.prev_inputs.popFront() orelse unreachable;
        self.allo.free(s);
    }

    if (self.prev_inputs.len() < 1 or !std.mem.eql(u8, self.prev_inputs.back().?.*, submission)) {
        if (std.fmt.allocPrint(self.allo, "{s}", .{submission})) |p| {
            self.prev_inputs.pushBack(p) catch {
                reportConsoleInputSaveFail.call();
            };
        } else |_| {
            reportConsoleHistoryFail.call();
        }
    }

    defer self.input_buf = [_]u8{0} ** @typeInfo(@TypeOf(self.input_buf)).Array.len;
    logSubmission(cx, "$ {s}", .{submission});
    var tokens = std.mem.tokenizeAny(u8, submission, " \t\n\r");

    const cmd_name = tokens.next() orelse {
        logSubmission(cx, "$ {s}", .{submission});
        return;
    };

    if (commands.get(cmd_name)) |*cmd| {
        const arg_str = submission[cmd_name.len..];

        var args = CommandArgs.init(self.allo, arg_str) catch {
            reportConsoleArgParseFail.call();
            return;
        };

        defer args.deinit();
        cmd.func(cx, cmd, &args);
    } else {
        logInfo(cx, "{s}: command not found", .{cmd_name});
        return;
    }
}

fn inputBufSlice(self: *Self) [:0]u8 {
    return self.input_buf[0..(@sizeOf(@TypeOf(self.input_buf)) - 1) :0];
}

var reportConsoleArgParseFail = std.once(doReportConsoleArgParseFail);

fn doReportConsoleArgParseFail() void {
    log.err("Failed to allocate console argument iterator", .{});
}

var reportConsoleHistoryFail = std.once(doReportConsoleHistoryFail);

fn doReportConsoleHistoryFail() void {
    log.err("Failed to add to console history", .{});
}

var reportConsoleInputSaveFail = std.once(doReportConsoleInputSaveFail);

fn doReportConsoleInputSaveFail() void {
    log.err("Failed to add to console input history", .{});
}
