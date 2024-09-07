const std = @import("std");

const root = @import("../build.zig");

pub fn build(
    b: *std.Build,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) void {
    const exe = b.addExecutable(.{
        .name = "viletech",
        .root_source_file = b.path("client/src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    var metainfo = b.addOptions();

    const DateTime = root.datetime.DateTime;
    var compile_timestamp_buf: [64]u8 = undefined;
    const compile_timestamp = std.fmt.bufPrint(
        compile_timestamp_buf[0..],
        "{}",
        .{DateTime.now()},
    ) catch unreachable;
    metainfo.addOption([]const u8, "compile_timestamp", compile_timestamp);

    const commit_hash = b.run(&[_][]const u8{ "git", "rev-parse", "HEAD" });
    metainfo.addOption([]const u8, "commit", commit_hash);

    exe.root_module.addOptions("meta", metainfo);

    const zig_args = b.dependency("zig-args", .{});
    exe.root_module.addImport("zig-args", zig_args.module("args"));

    root.engine.link(b, exe, null);
    root.subterra.link(b, exe, .{
        .znbx = .source,
    });
    root.wadload.link(b, exe, null);

    b.installArtifact(exe);

    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("client", "Build and run the VileTech client");
    run_step.dependOn(&run_cmd.step);
}
