//! Commands for the developer console.

const std = @import("std");

const argparse = @import("zig-args");

const Core = @import("../Core.zig");
const Console = @import("Console.zig");

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

    Console.logInfo(cx, "Will exit after this engine cycle finishes.", .{});
    cx.exit = true;
}
