const std = @import("std");
const log = std.log.scoped(.main);

const args = @import("zig-args");
const HhMmSs = @import("viletech").stdx.HhMmSs;

pub fn main() !void {
    const start_time = try std.time.Instant.now();
    log.debug("***** DEBUG BUILD *****", .{});

    const opts = try args.parseWithVerbForCurrentProcess(
        Params,
        Verbs,
        std.heap.page_allocator,
        .print,
    );
    defer opts.deinit();

    if (opts.options.help) {
        if (opts.verb) |_| {
            // Soon!
        } else {
            try args.printHelp(Params, "viletech", std.io.getStdOut().writer());
            return;
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
        .usage_summary = "[options...]",
        .option_docs = .{
            .help = "Print this usage information and then exit",
            .version = "Print version/compile information and then exit",
        },
    };
};

const Verbs = union(enum) {};
