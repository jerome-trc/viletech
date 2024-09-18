const std = @import("std");

const root = @import("../build.zig");

const Self = @This();

b: *std.Build,
target: std.Build.ResolvedTarget,
optimize: std.builtin.OptimizeMode,
check: *std.Build.Step,

assets: *std.Build.Module,
deque: *std.Build.Module,
zig_args: *std.Build.Module,

pub fn build(self: *Self) void {
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

    const run_cmd = self.b.addRunArtifact(exe);
    run_cmd.step.dependOn(self.b.getInstallStep());

    if (self.b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = self.b.step("client", "Build and run the VileTech client");
    run_step.dependOn(&run_cmd.step);

    self.check.dependOn(&exe_check.step);
}

fn exeCommon(self: *Self, exe: *std.Build.Step.Compile) void {
    exe.root_module.addImport("assets", self.assets);
    exe.root_module.addImport("deque", self.deque);
    exe.root_module.addImport("zig-args", self.zig_args);

    root.engine.link(self.b, exe, null);
    root.subterra.link(self.b, exe, .{ .znbx = .source });
    root.wadload.link(self.b, exe, null);
    root.zmsx.link(self.b, exe, exe.root_module.resolved_target.?, exe.root_module.optimize.?);
}
