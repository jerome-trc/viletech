const std = @import("std");

const cimgui = @import("../depend/build.cimgui.zig");
const sdl = @import("../depend/build.sdl.zig");
const zdfs = @import("../depend/build.zdfs.zig");

pub fn build(
    b: *std.Build,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
    test_step: *std.Build.Step,
) *std.Build.Module {
    var options = b.addOptions();
    // TODO: retrieve version stored in build.zig.zon.
    options.addOption([]const u8, "version", "0.0.0");
    options.addOption([]const u8, "commit", b.run(&[_][]const u8{
        "git",
        "rev-parse",
        "HEAD",
    }));

    const engine = b.addModule("viletech-engine", .{
        .root_source_file = b.path("client/src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    commonDependencies(b, options, target, optimize, .{ .module = engine });

    const exe = b.addExecutable(.{
        .name = "viletech",
        .root_source_file = b.path("client/src/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    commonDependencies(b, options, target, optimize, .{ .compile = exe });

    b.installArtifact(exe);

    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const exe_check = b.addExecutable(.{
        .name = "viletech",
        .root_source_file = b.path("client/src/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    commonDependencies(b, options, target, optimize, .{ .compile = exe_check });

    const check = b.step("check", "Semantic check for ZLS");
    check.dependOn(&exe_check.step);

    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run_cmd.step);

    const exe_unit_tests = b.addTest(.{
        .root_source_file = b.path("client/src/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    commonDependencies(b, options, target, optimize, .{ .compile = exe_unit_tests });

    const run_exe_unit_tests = b.addRunArtifact(exe_unit_tests);
    test_step.dependOn(&run_exe_unit_tests.step);

    return engine;
}

fn commonDependencies(
    b: *std.Build,
    meta: *std.Build.Step.Options,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
    artifact: union(enum) {
        compile: *std.Build.Step.Compile,
        module: *std.Build.Module,
    },
) void {
    const sdl_sdk = @import("../depend/build.sdl.zig").init(b, null);
    const zig_args = b.dependency("zig-args", .{});

    switch (artifact) {
        .compile => |c| {
            c.root_module.addOptions("meta", meta);

            c.linkLibC();
            c.linkLibCpp();

            c.linkSystemLibrary("sdl2_image");

            cimgui.build(b, c);
            sdl_sdk.link(c, .static);
            zdfs.build(b, c, target, optimize);

            c.root_module.addImport("sdl2", sdl_sdk.getWrapperModule());
            c.root_module.addImport("zig-args", zig_args.module("args"));
        },
        .module => |m| {
            m.addOptions("meta", meta);

            m.linkSystemLibrary("sdl2_image", .{
                .needed = true,
                .preferred_link_mode = .static,
                .use_pkg_config = .yes,
            });

            m.addIncludePath(b.path("depend/imgui"));
            m.addIncludePath(b.path("depend/zdfs/include"));

            m.addImport("sdl2", sdl_sdk.getWrapperModule());
            m.addImport("zig-args", zig_args.module("args"));
        },
    }
}
