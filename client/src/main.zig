const builtin = @import("builtin");
const std = @import("std");
const log = std.log.scoped(.main);
const meta = @import("meta");

const args = @import("zig-args");
const HhMmSs = viletech.stdx.HhMmSs;
const sdl = @import("sdl");
const viletech = @import("viletech");

const Converter = @import("Converter.zig");
const Core = @import("Core.zig");
const Frontend = @import("Frontend.zig");

pub const MainAllocator = if (builtin.mode == .Debug)
    std.heap.GeneralPurposeAllocator(.{})
else
    void;

fn onArgError(err: args.Error) anyerror!void {
    try std.io.getStdErr().writer().print("{s}\nSee `viletech --help`.\n", .{err});
}

pub fn main() !void {
    const start_time = try std.time.Instant.now();
    log.debug("***** DEBUG BUILD *****", .{});

    var gpa: MainAllocator = undefined;
    var main_alloc: std.mem.Allocator = undefined;

    if (builtin.mode == .Debug) {
        gpa = MainAllocator{};
        main_alloc = gpa.allocator(); // For leak checking.
    } else if (builtin.link_libc) {
        gpa = {};
        main_alloc = std.heap.c_allocator;
    } else {
        gpa = {};
        main_alloc = std.heap.page_allocator;
    }

    defer _ = if (builtin.mode == .Debug) gpa.deinit();

    const opts = args.parseWithVerbForCurrentProcess(
        Params,
        Verbs,
        main_alloc,
        .{ .forward = onArgError },
    ) catch std.process.exit(1); // Don't spew an error trace at the end user.
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

    if (opts.verb) |verb| {
        switch (verb) {
            .conv => |o| return Converter.run(o),
            .diff => @panic("not yet implemented"),
            .find => @panic("not yet implemented"),
            .prune => @panic("not yet implemented"),
        }
    }

    if (opts.options.gamemode.isOn() and !std.process.hasEnvVarConstant("VTEC_GAMEMODE_OFF")) {
        viletech.gamemode.start();
    }

    var cx = try Core.init(main_alloc);
    defer cx.deinit() catch |err| {
        log.err("Core de-initialization failed: {}", .{err});
        std.process.exit(255);
    };

    try sdl.init(.{ .video = true });
    defer sdl.quit();

    outer: while (true) {
        switch (cx.transition) {
            .entry_to_frontend => {
                cx.scene = Core.Scene{ .frontend = Frontend.init(&cx) };
            },
            .frontend_to_doom => {
                const front = cx.scene.frontend;
                cx.scene = Core.Scene{ .doom = try front.doomArgs() };
            },
            .frontend_to_exit => break :outer,
            .none, .frontend_to_editor, .frontend_to_hellblood => unreachable,
        }

        cx.transition = .none;

        switch (cx.scene) {
            .doom => |*argv| {
                sdl.quit();
                try argv.append(null);
                // Soon!
                unreachable;
            },
            .frontend => |*front| {
                while (cx.transition == .none) {
                    front.ui();
                }
            },
            .editor, .hellblood => unreachable, // Soon!
            .entry => unreachable,
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
    const Boolean = enum {
        false,
        no,
        off,

        true,
        yes,
        on,

        fn isOn(self: Boolean) bool {
            return switch (self) {
                .true, .yes, .on => true,
                else => false,
            };
        }
    };

    help: bool = false,
    version: bool = false,

    gamemode: Boolean = .true,

    pub const shorthands = .{
        .h = "help",
        .V = "version",
        .G = "gamemode",
    };

    pub const meta = .{
        .usage_summary = "[options...] [conv|diff|find|prune] [options...]",
        .option_docs = .{
            .help = "Print this usage information and then exit",
            .version = "Print version/compile information and then exit",

            .gamemode = if (builtin.os.tag == .linux)
                "`on` by default. `off` disables libgamemode"
            else
                "Linux-only; this does nothing on your operating system",
        },
    };
};

const Verbs = union(enum) {
    conv: Converter.Args,
    diff: void,
    find: void,
    prune: void,
};
