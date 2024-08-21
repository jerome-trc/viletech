const builtin = @import("builtin");
const std = @import("std");
const log = std.log.scoped(.ratboom);

const c = @import("main.zig").c;

const Console = @import("devgui/Console.zig");
const devgui = @import("devgui.zig");
const MusicGui = @import("devgui/MusicGui.zig");
const VfsGui = @import("devgui/VfsGui.zig");
const Path = @import("stdx.zig").Path;
const plugin = @import("plugin.zig");

const Self = @This();
pub const DebugAllocator = std.heap.GeneralPurposeAllocator(.{});
const StreamWriter = std.io.BufferedWriter(4096, std.fs.File.Writer);

pub const C = extern struct {
    devgui_open: bool,
    imgui_ctx: *c.ImGuiContext,
    saved_gametick: i32,

    pub fn core(ccx: *C) *Self {
        return @fieldParentPtr("c", ccx);
    }
};

pub const DevGui = struct {
    left: devgui.State,
    right: devgui.State,

    demo_window: bool = false,
    metrics_window: bool = false,
    debug_log: bool = false,
    id_stack_tool_window: bool = false,
    about_window: bool = false,
    user_guide: bool = false,
};

c: C,
alloc: std.mem.Allocator,
console: Console,
dgui: DevGui,
gpa: ?*DebugAllocator,
musicgui: MusicGui,
plugin: struct {
    dynlibs: std.ArrayList(std.DynLib),
    paths: std.ArrayList(Path),
},
prefs: std.StringHashMap(plugin.Pref),
start_time: ?std.time.Instant,
stderr_file: std.fs.File.Writer,
stderr_bw: StreamWriter,
stdout_file: std.fs.File.Writer,
stdout_bw: StreamWriter,
vfsgui: VfsGui,

pub fn init(gpa: ?*DebugAllocator, start_time: ?std.time.Instant) !Self {
    const alloc = if (gpa) |g| g.allocator() else std.heap.c_allocator;

    const stderr_file = std.io.getStdErr().writer();
    const stdout_file = std.io.getStdOut().writer();

    return Self{
        .c = Self.C{
            .devgui_open = if (builtin.mode == .Debug) true else false,
            .imgui_ctx = undefined,
            .saved_gametick = -1,
        },
        .alloc = alloc,
        .console = try Console.init(alloc),
        .dgui = Self.DevGui{
            .left = devgui.State.console,
            .right = devgui.State.music,
        },
        .gpa = gpa,
        .musicgui = MusicGui.init(alloc),
        .plugin = .{
            .dynlibs = std.ArrayList(std.DynLib).init(alloc),
            .paths = std.ArrayList(Path).init(alloc),
        },
        .prefs = std.StringHashMap(plugin.Pref).init(alloc),
        .start_time = start_time,
        .stderr_file = stderr_file,
        .stderr_bw = std.io.bufferedWriter(stderr_file),
        .stdout_file = stdout_file,
        .stdout_bw = std.io.bufferedWriter(stdout_file),
        .vfsgui = VfsGui.init(),
    };
}

pub fn deinit(self: *Self) void {
    self.stdout_bw.flush() catch {};
    self.stderr_bw.flush() catch {};

    self.unloadPlugins();
    self.plugin.dynlibs.deinit();

    for (self.plugin.paths.items) |p| {
        self.alloc.free(p);
    }

    self.plugin.paths.deinit();
    self.prefs.deinit();

    self.console.deinit();
    self.musicgui.deinit();
    // ImGui context has already been destroyed by a call from C.

    if (self.gpa) |gpa| {
        _ = gpa.detectLeaks();
        _ = gpa.deinit();
    }
}

pub fn eprintln(self: *Self, comptime format: []const u8, args: anytype) !void {
    try self.stderr_bw.writer().print(format ++ "\n", args);
    try self.stderr_bw.flush();
}

pub fn println(self: *Self, comptime format: []const u8, args: anytype) !void {
    try self.stdout_bw.writer().print(format ++ "\n", args);
    try self.stdout_bw.flush();
}

fn loadPlugins(self: *Self) std.DynLib.Error!void {
    for (self.plugin.paths.items) |path| {
        var dynlib = try std.DynLib.open(path);
        log.info("Loaded plugin: {s}", .{path});
        try self.plugin.dynlibs.append(dynlib);

        if (dynlib.lookup(plugin.OnLoad, "onLoad")) |onLoad| {
            onLoad(plugin.PCore{
                .prefs = &self.prefs,
                .raven = c.raven != 0,
            });
        }
    }
}

fn unloadPlugins(self: *Self) void {
    for (self.plugin.dynlibs.items) |*dynlib| {
        dynlib.close();
    }

    self.plugin.dynlibs.clearRetainingCapacity();
}

fn addPlugin(self: *Self, path: [:0]const u8) std.mem.Allocator.Error!void {
    try self.plugin.paths.append(path);
}

pub fn registerPref(self: *Self, pref_v: []const u8) std.mem.Allocator.Error!void {
    var split = std.mem.splitScalar(u8, pref_v, ':');

    const part0 = split.next().?;

    const part1 = split.next() orelse {
        try self.prefs.put(part0, plugin.Pref{ .boolean = true });
        return;
    };

    if (split.next()) |part2| {
        if (std.ascii.eqlIgnoreCase(part1, "bool")) {
            const val = if (std.ascii.eqlIgnoreCase(part2, "true"))
                true
            else if (std.ascii.eqlIgnoreCase(part2, "false"))
                false
            else
                c.I_Error(
                    "Failed to parse `%.*s` value `%.*s` as a boolean",
                    part0.len,
                    part0.ptr,
                    part1.len,
                    part1.ptr,
                );

            try self.prefs.put(part0, plugin.Pref{ .boolean = val });
        } else if (std.ascii.eqlIgnoreCase(part1, "float")) {
            const val = std.fmt.parseFloat(f64, part2) catch |err| {
                c.I_Error(
                    "Failed to parse `%.*s` value `%.*s` as an int (%s)",
                    part0.len,
                    part0.ptr,
                    part2.len,
                    part2.ptr,
                    @errorName(err).ptr,
                );
            };

            try self.prefs.put(part0, plugin.Pref{ .float = val });
        } else if (std.ascii.eqlIgnoreCase(part1, "int")) {
            const val = std.fmt.parseInt(i64, part2, 10) catch |err|
                c.I_Error(
                "Failed to parse `%.*s` value `%.*s` as an int (%s)",
                part0.len,
                part0.ptr,
                part2.len,
                part2.ptr,
                @errorName(err).ptr,
            );

            try self.prefs.put(part0, plugin.Pref{ .int = val });
        } else if (std.ascii.eqlIgnoreCase(part1, "string")) {
            try self.prefs.put(part0, plugin.Pref{ .string = part2 });
        } else {
            c.I_Error("Unknown pref. type: `%.*s`", part1.len, part1.ptr);
        }
    } else {
        if (std.fmt.parseFloat(f64, part1)) |val| {
            try self.prefs.put(part0, plugin.Pref{ .float = val });
            return;
        } else |_| {}

        if (std.fmt.parseInt(i64, part1, 10)) |val| {
            try self.prefs.put(part0, plugin.Pref{ .int = val });
            return;
        } else |_| {}

        if (std.ascii.eqlIgnoreCase(part1, "true")) {
            try self.prefs.put(part0, plugin.Pref{ .boolean = true });
            return;
        }

        if (std.ascii.eqlIgnoreCase(part1, "false")) {
            try self.prefs.put(part0, plugin.Pref{ .boolean = false });
            return;
        }

        try self.prefs.put(part0, plugin.Pref{ .string = part1 });
    }
}

fn deinitC(ccx: *C) callconv(.C) void {
    ccx.core().deinit();
}

fn addPluginC(ccx: *C, path: [*:0]const u8) callconv(.C) void {
    const p = ccx.core().alloc.dupeZ(u8, std.mem.sliceTo(path, 0)) catch
        c.I_Error("Plugin path registration failed: out of memory");
    ccx.core().addPlugin(p) catch
        c.I_Error("Plugin path registration failed: out of memory");
}

fn loadPluginsC(ccx: *C) callconv(.C) void {
    ccx.core().loadPlugins() catch |err|
        c.I_Error("Plugin load failed: %s", @errorName(err).ptr);
}

comptime {
    @export(deinitC, .{ .name = "coreDeinit" });
    @export(addPluginC, .{ .name = "addPlugin" });
    @export(loadPluginsC, .{ .name = "loadPlugins" });
}
