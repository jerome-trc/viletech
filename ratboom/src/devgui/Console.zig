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
    summary: []const u8,
};

const HistoryItem = union(enum) {
    info: []const u8,
    stdlog: []const u8,
    submission: []const u8,
    toast: []const u8,
};

pub const commands = std.StaticStringMap(Command).initComptime(.{
    .{ "clear", Command{
        .name = "clear",
        .func = &ccmds.clear,
        .summary = "Clear the message log",
    } },
    .{ "exit", Command{
        .name = "exit",
        .func = &ccmds.exit,
        .summary = "Close the engine",
    } },
    .{ "help", Command{
        .name = "help",
        .func = &ccmds.help,
        .summary = "Print a list of all commands",
    } },
    .{ "level.exit", Command{
        .name = "level.exit",
        .func = &ccmds.levelExit,
        .summary = "",
    } },
    .{ "pistolstart.hold", Command{
        .name = "pistolstart.hold",
        .func = &ccmds.pistolstartHold,
        .summary = "Disable -pistolstart for one level transition",
    } },
    .{ "plugin", Command{
        .name = "plugin",
        .func = &ccmds.plugin,
        .summary = "Inspect and manipulate plugins",
    } },
    .{ "quit", Command{
        .name = "quit",
        .func = &ccmds.exit,
        .summary = "Close the engine",
    } },
    .{ "version", Command{
        .name = "version",
        .func = &ccmds.version,
        .summary = "Prints various engine version metadata",
    } },
});

pub const cmd_name_max_len: usize = blk: {
    var max_name_len: usize = 0;

    for (commands.keys(), commands.values()) |k, v| {
        std.debug.assert(std.mem.eql(u8, k, v.name));
        max_name_len = @max(max_name_len, k.len);
    }

    break :blk max_name_len;
};

pub var std_log: Deque([]const u8) = undefined;
pub var std_log_mutex: std.Thread.Mutex = .{};
var std_log_init = std.once(initStdLogQueue);

/// This is backed by either a GPA or `std.heap.c_allocator`.
alloc: std.mem.Allocator,
input_buf: [256]u8,
history: Deque(HistoryItem),
prev_inputs: Deque([]const u8),

pub fn init(allocator: std.mem.Allocator) !Self {
    std_log_init.call();

    return Self{
        .alloc = allocator,
        .input_buf = [_]u8{0} ** 256,
        .history = try Deque(HistoryItem).init(allocator),
        .prev_inputs = try Deque([]const u8).init(allocator),
    };
}

pub fn deinit(self: *Self) void {
    while (self.history.popFront()) |h| {
        switch (h) {
            .info => |s| self.alloc.free(s),
            .stdlog => |s| std.heap.c_allocator.free(s),
            .submission => |s| self.alloc.free(s),
            .toast => |s| self.alloc.free(s),
        }
    }

    self.history.deinit();

    while (self.prev_inputs.popFront()) |p| {
        self.alloc.free(p);
    }

    self.prev_inputs.deinit();
}

pub fn layout(cx: *Core, left: bool, menu_bar_height: f32) void {
    var self = &cx.console;

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
                imgui.report_err_clipper_ctor.call();
                break :scroll;
            };
            defer clipper.deinit();
            clipper.begin(self.history.len(), -1.0);

            while (clipper.step()) {
                for (clipper.displayStart()..clipper.displayEnd()) |i| {
                    switch ((self.history.get(i) orelse unreachable).*) {
                        HistoryItem.info => |str| {
                            // Light green
                            const color = .{ .x = 0.8, .y = 1.0, .z = 0.7, .w = 1.0 };
                            c.igTextColored(color, "%.*s", str.len, str.ptr);
                        },
                        HistoryItem.stdlog => |str| {
                            const color: c.ImVec4 = if (std.mem.startsWith(u8, str, "info("))
                                .{ .x = 0.7, .y = 1.0, .z = 1.0, .w = 1.0 } // Light blue
                            else if (std.mem.startsWith(u8, str, "warning("))
                                .{ .x = 1.0, .y = 1.0, .z = 0.1, .w = 1.0 } // Yellow
                            else if (std.mem.startsWith(u8, str, "error("))
                                .{ .x = 1.0, .y = 0.1, .z = 0.1, .w = 1.0 } // Red
                            else if (std.mem.startsWith(u8, str, "debug("))
                                .{ .x = 1.0, .y = 0.1, .z = 0.7, .w = 1.0 } // Pink
                            else
                                .{ .x = 0.7, .y = 0.1, .z = 1.0, .w = 1.0 }; // Purple

                            c.igTextColored(color, "%.*s", str.len, str.ptr);
                        },
                        HistoryItem.submission => |str| {
                            // Light grey
                            const color = .{ .x = 0.7, .y = 0.7, .z = 0.7, .w = 1.0 };
                            c.igTextColored(color, "%.*s", str.len, str.ptr);
                        },
                        HistoryItem.toast => |str| {
                            // White
                            const color = .{ .x = 1.0, .y = 1.0, .z = 1.0, .w = 1.0 };
                            c.igTextColored(color, "%.*s", str.len, str.ptr);
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
        submitCommands(cx);
    }

    c.igSameLine(0.0, -1.0);

    if (c.igButton("Submit", .{ .x = 0.0, .y = 0.0 })) {
        submitCommands(cx);
    }

    c.igSetItemDefaultFocus();
}

pub fn logHelp(self: *Self, comptime format: []const u8, args: anytype) void {
    errdefer report_console_history_fail.call();
    const p = std.fmt.allocPrint(self.alloc, format, args) catch return;
    self.history.pushBack(HistoryItem{ .info = p }) catch return;
}

pub fn logInfo(cx: *Core, comptime format: []const u8, args: anytype) void {
    errdefer report_console_history_fail.call();
    cx.eprintln(format, args) catch return;
    const p = std.fmt.allocPrint(cx.console.alloc, format, args) catch return;
    cx.console.history.pushBack(HistoryItem{ .info = p }) catch return;
}

pub fn logSubmission(cx: *Core, comptime format: []const u8, args: anytype) void {
    errdefer report_console_history_fail.call();
    cx.eprintln(format, args) catch return;
    const p = std.fmt.allocPrint(cx.console.alloc, format, args) catch return;
    cx.console.history.pushBack(HistoryItem{ .submission = p }) catch return;
}

pub fn logToast(cx: *Core, comptime format: []const u8, args: anytype) void {
    errdefer report_console_history_fail.call();
    cx.eprintln(format, args) catch return;
    const p = std.fmt.allocPrint(cx.console.alloc, format, args) catch return;
    cx.console.history.pushBack(HistoryItem{ .toast = p }) catch return;
}

pub fn feedFromStdLog(self: *Self) void {
    std_log_mutex.lock();
    defer std_log_mutex.unlock();

    while (std_log.popFront()) |l| {
        self.history.pushBack(HistoryItem{ .stdlog = l }) catch {};
    }
}

fn cToast(ccx: *Core.C, msg: [*:0]const u8) callconv(.C) void {
    logToast(ccx.core(), "{s}", .{msg});
}

/// Allows the console to fulfill the interface of a writer like stdout.
pub fn print(self: *Self, comptime format: []const u8, args: anytype) !void {
    self.logHelp(format, args);
}

fn inputTextCallback(data: [*c]c.ImGuiInputTextCallbackData) callconv(.C) c_int {
    _ = data;
    return 0;
}

fn submitCommands(cx: *Core) void {
    var self = &cx.console;
    const submission = std.mem.sliceTo(&self.input_buf, 0);

    if (submission.len < 1) {
        logSubmission(cx, "$", .{});
        return;
    }

    if (self.prev_inputs.len() > 256) {
        const s = self.prev_inputs.popFront() orelse unreachable;
        self.alloc.free(s);
    }

    if (self.prev_inputs.len() < 1 or !std.mem.eql(u8, self.prev_inputs.back().?.*, submission)) {
        if (std.fmt.allocPrint(self.alloc, "{s}", .{submission})) |p| {
            self.prev_inputs.pushBack(p) catch {
                console_input_save_fail.call();
            };
        } else |_| {
            report_console_history_fail.call();
        }
    }

    defer self.input_buf = [_]u8{0} ** @typeInfo(@TypeOf(self.input_buf)).Array.len;
    logSubmission(cx, "$ {s}", .{submission});
    var parts = std.mem.tokenizeScalar(u8, submission, ';');

    while (true) {
        const part = parts.next() orelse break;
        submitCommand(cx, part);
    }
}

fn submitCommand(cx: *Core, command: []const u8) void {
    const self = &cx.console;
    var tokens = std.mem.tokenizeAny(u8, command, " \t\n\r");

    const cmd_name = tokens.next() orelse {
        logSubmission(cx, "$ {s}", .{command});
        return;
    };

    if (commands.get(cmd_name)) |*cmd| {
        const arg_str = command[cmd_name.len..];

        var args = CommandArgs.init(self.alloc, arg_str) catch {
            report_console_arg_parse_fail.call();
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

fn initStdLogQueue() void {
    std_log = Deque([]const u8).init(std.heap.c_allocator) catch |err|
        c.I_Error("Failed to initialize log-to-console queue: %s", @errorName(err).ptr);
}

var report_console_arg_parse_fail = std.once(reportConsoleArgParseFail);

fn reportConsoleArgParseFail() void {
    log.err("Failed to allocate console argument iterator", .{});
}

var report_console_history_fail = std.once(reportConsoleHistoryFail);

fn reportConsoleHistoryFail() void {
    log.err("Failed to add to console history", .{});
}

var console_input_save_fail = std.once(consoleInputSaveFail);

fn consoleInputSaveFail() void {
    log.err("Failed to add to console input history", .{});
}

comptime {
    @export(cToast, .{ .name = "addConsoleToast" });
}