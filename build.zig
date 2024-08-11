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
        // Always use -Doptimize=ReleaseSafe,
        // since we want the demotest to run as quickly as possible.
        .optimize = .ReleaseSafe,
    });
    demotest.step.dependOn(&lib.step);

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

    const fd4rb_decohack = b.addSystemCommand(&[_][]const u8{
        "decohack",
        "--budget",
        "-s",
        "zig-out/fd4rb.wad",
        "-o",
        "zig-out/fd4rb.deh",
        "plugins/fd4rb/decohack/burst-shotgun.dh",
        "plugins/fd4rb/decohack/revolver.dh",
    });
    fd4rb_decohack.addFileInput(b.path("plugins/fd4rb/decohack/burst-shotgun.dh"));
    fd4rb_decohack.addFileInput(b.path("plugins/fd4rb/decohack/revolver.dh"));
    fd4rb.step.dependOn(&fd4rb_decohack.step);

    if (std.process.getEnvVarOwned(b.allocator, "DJWAD_DIR")) |path| {
        const dir = std.fs.openDirAbsolute(path, .{}) catch unreachable;
        @import("tunetech").djwad(b.allocator, dir) catch unreachable;
    } else |_| {}

    const vilebuild = b.addExecutable(.{
        .name = "vilebuild",
        .root_source_file = b.path("vilebuild/main.zig"),
        .target = target,
        .optimize = .Debug,
    });
    vilebuild.linkLibC();
    b.installArtifact(vilebuild);

    const dehpp = b.addRunArtifact(vilebuild);
    dehpp.step.dependOn(&fd4rb_decohack.step);
    dehpp.addFileInput(b.path("zig-out/fd4rb.deh"));
    fd4rb.step.dependOn(&dehpp.step);
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
