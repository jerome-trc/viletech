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

    const zig_args = b.dependency("zig-args", .{});
    exe.root_module.addImport("zig-args", zig_args.module("args"));

    root.engine.link(b, exe, null);
    root.subterra.link(b, exe, null);
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
