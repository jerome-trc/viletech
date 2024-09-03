//! Commands for the developer console.

const std = @import("std");

const argparse = @import("zig-args");
const c = @import("../main.zig").c;

const Core = @import("../Core.zig");
const Console = @import("Console.zig");

pub fn clear(cx: *Core, _: *const Console.Command, _: *Console.CommandArgs) void {
    var self = &cx.console;

    while (self.history.popFront()) |h| {
        switch (h) {
            .info => |s| cx.console.alloc.free(s),
            .stdlog => |s| std.heap.c_allocator.free(s),
            .submission => |s| cx.console.alloc.free(s),
            .toast => |s| cx.console.alloc.free(s),
        }
    }
}

pub fn exit(cx: *Core, cmd: *const Console.Command, args: *Console.CommandArgs) void {
    const Args = struct {
        help: bool = false,
        force: bool = false,

        pub const shorthands = .{ .f = "force", .h = "help" };

        pub const meta = .{
            .usage_summary = "[options...]",
            .option_docs = .{
                .force = "Exit as fast as possible",
                .help = "Print this usage information and do nothing else",
            },
        };
    };

    const opts = argparse.parse(Args, args, std.heap.c_allocator, .print) catch |err| {
        Console.logInfo(cx, "Failed to parse arguments: {s}", .{@errorName(err)});
        return;
    };

    defer opts.deinit();

    if (opts.options.help) {
        argparse.printHelp(Args, cmd.name, &cx.console) catch {
            Console.logInfo(cx, "Failed to print command help.", .{});
        };

        return;
    }

    if (opts.options.force) {
        std.process.exit(0);
    }

    c.I_SafeExit(0);
}

pub fn help(cx: *Core, _: *const Console.Command, _: *Console.CommandArgs) void {
    const cmd_name_max_len = std.fmt.comptimePrint("{}", .{Console.cmd_name_max_len});
    const cmd_fmt = comptime "{s: <" ++ cmd_name_max_len ++ "} : {s}";

    Console.logInfo(cx, "All commands:", .{});

    for (Console.commands.values()) |cmd| {
        Console.logInfo(cx, cmd_fmt, .{ cmd.name, cmd.summary });
    }
}

pub fn levelExit(cx: *Core, _: *const Console.Command, args: *Console.CommandArgs) void {
    if (c.hexen != 0) {
        Console.logInfo(cx, "`level.exit` cannot be used with Hexen.", .{});
        return;
    }

    if (c.gamestate != c.GS_LEVEL) {
        Console.logInfo(cx, "`level.exit` only works when in a level.", .{});
        return;
    }

    var position: c_int = 0;

    const Params = struct {};

    const opts = argparse.parse(Params, args, std.heap.c_allocator, .print) catch |err| {
        Console.logInfo(cx, "Failed to parse arguments: {s}", .{@errorName(err)});
        return;
    };

    defer opts.deinit();

    for (opts.positionals) |pos| {
        position = std.fmt.parseInt(c_int, pos, 10) catch continue;
    }

    c.G_ExitLevel(position);
}

pub fn pistolstartHold(cx: *Core, _: *const Console.Command, _: *Console.CommandArgs) void {
    if (c.pistolstart == c.pistolstart_off) {
        Console.logInfo(cx, "Pistol start is disabled; this command does nothing.", .{});
        return;
    }

    c.pistolstart = c.pistolstart_held;
    Console.logInfo(cx, "Pistol start temporarily disabled for the next level transition only.", .{});
}

pub fn plugin(_: *Core, _: *const Console.Command, _: *Console.CommandArgs) void {
    // TODO
}

pub fn version(cx: *Core, _: *const Console.Command, _: *Console.CommandArgs) void {
    const meta = @import("meta");
    Console.logInfo(cx, "{s} UTC {s}", .{ meta.compile_timestamp, meta.commit });
}
