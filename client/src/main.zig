const std = @import("std");
const log = std.log.scoped(.main);
const meta = @import("meta");

const args = @import("zig-args");
const HhMmSs = @import("viletech").stdx.HhMmSs;

const Converter = @import("Converter.zig");

pub fn main() !void {
    const start_time = try std.time.Instant.now();

    const opts = try args.parseWithVerbForCurrentProcess(
        Params,
        Verbs,
        std.heap.page_allocator,
        .print,
    );
    defer opts.deinit();

    if (opts.options.help) {
        if (opts.verb) |verb| {
            switch (verb) {
                .conv => {
                    try args.printHelp(Converter.Args, "viletech", std.io.getStdOut().writer());
                    return;
                },
                .diff => @panic("not yet implemented"),
                .find => @panic("not yet implemented"),
                .prune => @panic("not yet implemented"),
            }
        } else {
            try args.printHelp(Params, "viletech", std.io.getStdOut().writer());
            return;
        }
    }

    if (opts.options.version) {
        const stdout_file = std.io.getStdOut().writer();
        var stdout_bw = std.io.bufferedWriter(stdout_file);

        try stdout_bw.writer().print("{s}\n{s}\n{s}\n", .{
            meta.version,
            meta.commit,
            meta.compile_timestamp,
        });
        try stdout_bw.flush();
        return;
    }

    log.debug("***** DEBUG BUILD *****", .{});

    if (opts.verb) |verb| {
        switch (verb) {
            .conv => |o| return Converter.run(o),
            .diff => @panic("not yet implemented"),
            .find => @panic("not yet implemented"),
            .prune => @panic("not yet implemented"),
        }
    }

    const end_time = try std.time.Instant.now();
    const duration = HhMmSs.fromNs(end_time.since(start_time));

    // In my experience, runtime duration is a good thing to have in a bug report,
    // and thus a good thing to include in logs.
    log.info("Engine uptime: {:0>2}:{:0>2}:{:0>2}", .{
        duration.hours,
        duration.minutes,
        duration.seconds,
    });
}

const Params = struct {
    help: bool = false,
    version: bool = false,

    pub const shorthands = .{
        .h = "help",
        .V = "version",
    };

    pub const meta = .{
        .usage_summary = "[options...] [conv|diff|find|prune] [options...]",
        .option_docs = .{
            .help = "Print this usage information and then exit",
            .version = "Print version/compile information and then exit",
        },
    };
};

const Verbs = union(enum) {
    conv: Converter.Args,
    diff: void,
    find: void,
    prune: void,
};
