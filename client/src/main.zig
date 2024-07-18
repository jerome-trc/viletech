const builtin = @import("builtin");
const std = @import("std");
const log = std.log.scoped(.main);
const meta = @import("meta");

const args = @import("zig-args");
const sdl = @import("sdl2");

const Core = @import("Core.zig");
const devgui = @import("devgui.zig");
const Game = @import("Game.zig");
const Frontend = @import("Frontend.zig");
const imgui = @import("imgui.zig");
const gamemode = @import("gamemode.zig");
const platform = @import("platform.zig");

comptime {
    if (builtin.mode == .ReleaseFast or builtin.mode == .ReleaseSmall) {
        @compileError("ReleaseFast and ReleaseSmall builds are currently unsupported");
    }
}

pub const c = @cImport({
    @cDefine("CIMGUI_USE_SDL2", {});
    @cDefine("CIMGUI_DEFINE_ENUMS_AND_STRUCTS", {});
    @cInclude("cimgui.h");
    @cInclude("cimgui_impl.h");
    @cUndef("CIMGUI_USE_SDL2");
    @cUndef("CIMGUI_DEFINE_ENUMS_AND_STRUCTS");

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
    const start_time = try std.time.Instant.now();
    std.log.debug("***** DEBUG BUILD *****", .{});

    const opts = try args.parseWithVerbForCurrentProcess(Params, Verbs, std.heap.page_allocator, .print);
    defer opts.deinit();

    if (opts.options.help) {
        try args.printHelp(Params, "viletech", std.io.getStdOut().writer());
        return;
    }

    if (opts.options.version) {
        const stdout_file = std.io.getStdOut().writer();
        var stdout_bw = std.io.bufferedWriter(stdout_file);

        try stdout_bw.writer().print("{s} {s}", .{ meta.version, meta.commit });
        try stdout_bw.flush();
        return;
    }

    var gpa: if (builtin.mode == .Debug) Core.DebugAllocator else void = undefined;
    var core_alloc: std.mem.Allocator = undefined;

    if (builtin.mode == .Debug) {
        gpa = Core.DebugAllocator{};
        core_alloc = gpa.allocator();
    } else {
        gpa = {};
        core_alloc = std.heap.c_allocator;
    }

    var cx = try Core.init(if (builtin.mode == .Debug) &gpa else null);
    defer cx.deinit() catch {};

    try cx.eprintln(
        \\VileTech Engine {s} (https://github.com/jerome-trc/viletech)
        \\
        \\VileTech is released under the GNU General Public License v3.0.
        \\You are welcome to redistribute it under certain conditions.
        \\It comes with ABSOLUTELY NO WARRANTY. See the file LICENSE for details.
    , .{meta.version});

    gamemode.start();

    try sdl.init(.{
        .video = true,
        .events = true,
        .audio = true,
    });
    defer sdl.quit();

    try sdl.image.init(.{ .png = true });
    defer sdl.image.quit();

    try cx.displays.append(try platform.Display.init());

    const imgui_io = c.igGetIO();
    imgui_io.*.ConfigFlags |= c.ImGuiConfigFlags_NavEnableKeyboard;

    c.igStyleColorsDark(null);

    outer: while (true) {
        while (sdl.pollNativeEvent()) |native_event| {
            _ = imgui.implSdl2.processEvent(@ptrCast(&native_event));

            switch (sdl.Event.from(native_event)) {
                .quit => break :outer,
                .window => |event| if (platform.onWindowEvent(&cx, event)) break :outer else {},
                .drop_file => |event| {
                    if (cx.scene_tag == .frontend and event.windowID == 1) {
                        try cx.scene.frontend.addToLoadOrder(std.mem.sliceTo(event.file, 0));
                    }
                },
                else => {},
            }
        }

        for (0.., cx.displays.items) |i, *display| {
            display.newFrame();

            if (i == 0) {
                switch (cx.scene_tag) {
                    .frontend => switch (Frontend.draw(&cx)) {
                        .exit => break :outer,
                        .none => {},
                        .start_game => cx.transition = Core.Transition{
                            .game = .{
                                .load_order = try cx.scene.frontend.load_order.toOwnedSlice(),
                            },
                        },
                    },
                    .game => {},
                }
            }

            devgui.draw(&cx, display);
            try display.finishFrame(imgui_io);
        }

        switch (cx.transition) {
            .none => {},
            .exit => break :outer,
            .frontend => {
                try cx.deinitScene();
                cx.scene = Core.Scene{ .frontend = try Frontend.init(cx.alloc) };
                cx.scene_tag = .frontend;
                cx.transition = .none;
            },
            .game => |t| {
                try cx.deinitScene();
                cx.scene = Core.Scene{ .game = try Game.init(&cx, t.load_order) };
                cx.scene_tag = .game;
                cx.transition = .none;
            },
        }
    }

    const end_time = try std.time.Instant.now();
    const duration = HhMmSs.fromNs(end_time.since(start_time));

    // In my experience, runtime duration is a good thing to have in a bug report.
    log.info("Engine uptime: {:0>2}:{:0>2}:{:0>2}", .{
        duration.hours,
        duration.minutes,
        duration.seconds,
    });
}

/// Hours, minutes, and seconds.
const HhMmSs = struct {
    hours: u64,
    minutes: u64,
    seconds: u64,

    fn fromNs(nanos: u64) @This() {
        const microsecs = nanos / std.time.ns_per_us;
        const millisecs = microsecs / std.time.us_per_ms;
        var secs = millisecs / std.time.ms_per_s;
        var mins = secs / std.time.s_per_min;
        const hours = mins / 60;

        secs -= (mins * 60);
        mins -= (hours * 60);

        return .{ .hours = hours, .minutes = mins, .seconds = secs };
    }
};
