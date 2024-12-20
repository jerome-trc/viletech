const builtin = @import("builtin");
const std = @import("std");

const root = @import("../build.zig");

const Self = @This();

b: *std.Build,
target: std.Build.ResolvedTarget,
optimize: std.builtin.OptimizeMode,
check: *std.Build.Step,

libdumb: bool,
libfluidsynth: bool,
libsdlimage: bool,
libmad: bool,
libportmidi: bool,
libvorbisfile: bool,

assets: *std.Build.Module,
deque: *std.Build.Module,
sdl: *root.Sdl,
zig_args: *std.Build.Module,

pub fn build(self: *Self) *std.Build.Step.Compile {
    const exe_options = std.Build.ExecutableOptions{
        .name = "viletech",
        .root_source_file = self.b.path("client/src/main.zig"),
        .target = self.target,
        .optimize = self.optimize,
    };

    const exe = self.b.addExecutable(exe_options);
    const exe_check = self.b.addExecutable(exe_options);
    self.exeCommon(exe);
    self.exeCommon(exe_check);

    var metainfo = self.b.addOptions();

    const DateTime = root.datetime.DateTime;
    var compile_timestamp_buf: [64]u8 = undefined;
    const compile_timestamp = std.fmt.bufPrint(
        compile_timestamp_buf[0..],
        "{}",
        .{DateTime.now()},
    ) catch unreachable;
    metainfo.addOption([]const u8, "compile_timestamp", compile_timestamp);

    const commit_hash = self.b.run(&[_][]const u8{ "git", "rev-parse", "HEAD" });
    metainfo.addOption([]const u8, "commit", std.mem.trim(u8, commit_hash, " \n\r\t"));
    metainfo.addOption([]const u8, "version", root.packageVersion());

    exe.root_module.addOptions("meta", metainfo);
    exe_check.root_module.addOptions("meta", metainfo);

    self.b.installArtifact(exe);
    self.check.dependOn(&exe_check.step);

    const run_cmd = self.b.addRunArtifact(exe);
    run_cmd.step.dependOn(self.b.getInstallStep());

    if (self.b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = self.b.step("client", "Install and run the VileTech client");
    run_step.dependOn(&run_cmd.step);

    return exe;
}

fn exeCommon(self: *Self, exe: *std.Build.Step.Compile) void {
    exe.linkLibC();
    exe.linkLibCpp();

    root.engine.link(self.b, exe, null);
    root.subterra.link(self.b, exe, .{ .znbx = true });
    root.wadload.link(self.b, exe, null);
    self.sdl.link(exe, .static, .SDL2);
    root.zmsx.link(self.b, exe, .{
        .target = exe.root_module.resolved_target.?,
        .optimize = exe.root_module.optimize.?,
    });

    exe.root_module.addImport("assets", self.assets);
    exe.root_module.addImport("deque", self.deque);
    exe.root_module.addImport("sdl", self.sdl.getWrapperModule());
    exe.root_module.addImport("zig-args", self.zig_args);
}
