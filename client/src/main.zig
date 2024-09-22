const builtin = @import("builtin");
const std = @import("std");
const log = std.log.scoped(.main);
const meta = @import("meta");

const args = @import("zig-args");
const HhMmSs = @import("viletech").stdx.HhMmSs;

const Converter = @import("Converter.zig");

const MainAllocator = if (builtin.mode == .Debug)
    std.heap.GeneralPurposeAllocator(.{})
else
    void;

fn onArgError(err: args.Error) anyerror!void {
    try std.io.getStdErr().writer().print("{s}.\nSee `viletech --help`.\n", .{err});
}

extern "C" fn dsdaMain(argc: c_int, argv: [*][*:0]u8) c_int;

pub fn main() !void {
    const start_time = try std.time.Instant.now();
    log.debug("***** DEBUG BUILD *****", .{});

    for (std.os.argv) |arg| {
        // Temporary hack until C-side argument parsing is revised or excised.
        if (std.mem.eql(u8, std.mem.sliceTo(arg, 0), "-iwad")) {
            _ = dsdaMain(@intCast(std.os.argv.len), std.os.argv.ptr);
            unreachable;
        }
    }

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

fn refAllDeclsRecursive(comptime T: type) void {
    for (@import("std").meta.declarations(T)) |decl| {
        if (@TypeOf(@field(T, decl.name)) == type) {
            switch (@typeInfo(@field(T, decl.name))) {
                .Struct, .Enum, .Union, .Opaque => refAllDeclsRecursive(@field(T, decl.name)),
                else => {},
            }
        }
        _ = &@field(T, decl.name);
    }
}

comptime {
    // Crappy hack without which symbols don't end up in binary.
    // Alternatively could use `@export` but that would make this
    // already-very-tedious process even more tedious.
    refAllDeclsRecursive(@import("c.zig"));
}

// Functions C needs for now ///////////////////////////////////////////////////

const SliceU8 = extern struct {
    ptr: [*]const u8,
    len: usize,
};

export fn pathStem(path: [*:0]const u8) SliceU8 {
    const ret = std.fs.path.stem(std.mem.sliceTo(path, 0));
    return SliceU8{ .ptr = ret.ptr, .len = ret.len };
}

export fn windowIcon() SliceU8 {
    const bytes = @import("assets").viletech_png;
    return SliceU8{ .ptr = bytes.ptr, .len = bytes.len };
}
