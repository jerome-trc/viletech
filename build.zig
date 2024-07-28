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

    const module = b.addModule("ratboom", .{
        .root_source_file = b.path("client/src/plugin.zig"),
        .target = target,
        .optimize = optimize,
    });
    module.addIncludePath(b.path("build"));
    module.addIncludePath(b.path("dsda-doom/prboom2/src"));
    @import("depend/build.cimgui.zig").moduleLink(b, module);
    module.addImport("zig-args", b.dependency("zig-args", .{}).module("args"));

    const fd4rb = b.addSharedLibrary(.{
        .name = "fd4rb",
        .root_source_file = b.path("plugins/fd4rb/src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    commonDependencies(b, fd4rb, target, optimize);
    fd4rb.root_module.addImport("ratboom", module);
    b.installArtifact(fd4rb);
}

fn commonDependencies(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    _: std.Build.ResolvedTarget,
    _: std.builtin.OptimizeMode,
) void {
    const zig_args = b.dependency("zig-args", .{});

    compile.linkLibC();
    compile.linkLibCpp();
    compile.addIncludePath(b.path("build"));
    compile.addIncludePath(b.path("dsda-doom/prboom2/src"));

    if (compile.kind == .lib and compile.linkage != .dynamic) {
        compile.bundle_compiler_rt = true;
        compile.pie = true;
    }

    @import("depend/build.cimgui.zig").link(b, compile);
    compile.root_module.addImport("zig-args", zig_args.module("args"));
}
