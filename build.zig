const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const lib = b.addStaticLibrary(.{
        .name = "ratboomzig",
        .target = target,
        .optimize = optimize,
        .root_source_file = b.path("client/src/main.zig"),
    });
    commonDependencies(b, lib, target, optimize);
    b.installArtifact(lib);

    const lib_check = b.addStaticLibrary(.{
        .name = "ratboomzig",
        .target = target,
        .optimize = optimize,
        .root_source_file = b.path("client/src/main.zig"),
    });
    commonDependencies(b, lib_check, target, optimize);

    const check = b.step("check", "Semantic check for ZLS");
    check.dependOn(&lib_check.step);

    const demotest_step = b.step("demotest", "Run demo accuracy regression tests");

    const demotest = b.addTest(.{
        .root_source_file = b.path("demotest/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    const run_demotest = b.addRunArtifact(demotest);
    demotest_step.dependOn(&run_demotest.step);
}

fn commonDependencies(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    _: std.Build.ResolvedTarget,
    _: std.builtin.OptimizeMode,
) void {
    compile.linkLibC();
    compile.linkLibCpp();
    compile.addIncludePath(b.path("dsda-doom/prboom2/src"));
    compile.bundle_compiler_rt = true;
    compile.pie = true;
    @import("depend/build.cimgui.zig").link(b, compile);
}
