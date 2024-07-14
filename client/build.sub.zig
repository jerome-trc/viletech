const std = @import("std");

const cimgui = @import("../depend/build.cimgui.zig");
const sdl = @import("../depend/build.sdl.zig");
const zdfs = @import("../depend/build.zdfs.zig");

pub fn build(
    b: *std.Build,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
    test_step: *std.Build.Step,
) void {
    var options = b.addOptions();
    // TODO: retrieve version stored in build.zig.zon.
    options.addOption([]const u8, "version", "0.0.0");
    options.addOption([]const u8, "commit", b.run(&[_][]const u8{
        "git",
        "rev-parse",
        "HEAD",
    }));

    const exe = b.addExecutable(.{
        .name = "viletech",
        .root_source_file = b.path("client/src/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    common(b, exe, options);

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
    common(b, exe_check, options);

    const check = b.step("check", "Semantic check for ZLS");
    check.dependOn(&exe_check.step);

    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run_cmd.step);

    const exe_unit_tests = b.addTest(.{
        .root_source_file = b.path("client/src/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    common(b, exe_unit_tests, options);

    const run_exe_unit_tests = b.addRunArtifact(exe_unit_tests);
    test_step.dependOn(&run_exe_unit_tests.step);
}

fn common(b: *std.Build, compile: *std.Build.Step.Compile, meta: *std.Build.Step.Options) void {
    const sdl_sdk = sdl.init(b, null);
    const zig_args = b.dependency("zig-args", .{});

    compile.linkLibC();
    compile.linkLibCpp();

    cimgui.build(b, compile);
    sdl_sdk.link(compile, .static);
    zdfs.build(b, compile);

    compile.root_module.addOptions("meta", meta);
    compile.root_module.addImport("sdl2", sdl_sdk.getWrapperModule());
    compile.linkSystemLibrary("sdl2_image");
    compile.root_module.addImport("zig-args", zig_args.module("args"));
}
