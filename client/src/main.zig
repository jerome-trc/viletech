const std = @import("std");
const log = std.log.scoped(.viletech);
const meta = @import("meta");

const args = @import("zig-args");

const Core = @import("Core.zig");
const gamemode = @import("gamemode.zig");

pub const c = @cImport({
    @cInclude("zdfs/zdfs.h");
});

const Params = struct {
    help: bool = false,
    version: bool = false,

    pub const shorthands = .{
        .h = "help",
        .V = "version",
    };

    pub const meta = .{
        .usage_summary = "[options...]",
        .option_docs = .{
            .help = "Print this usage information and then exit",
            .version = "Print version/compile information and then exit",
        },
    };
};

const Verbs = union(enum) {};

pub fn main() !void {
    var cx = try Core.init();
    defer cx.deinit();

    const opts = try args.parseWithVerbForCurrentProcess(Params, Verbs, std.heap.page_allocator, .print);
    defer opts.deinit();

    if (opts.options.help) {
        try args.printHelp(Params, "viletech", cx.stdout_file);
        return;
    }

    if (opts.options.version) {
        try cx.println("{s} {s}", .{ meta.version, meta.commit });
        return;
    }

    gamemode.start();
}
