const std = @import("std");

pub fn link(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) void {
    const dep = b.dependency("znbx", .{});

    var lib = b.addStaticLibrary(.{
        .name = "znbx",
        .target = target,
        .optimize = optimize,
    });
    lib.linkLibC();
    lib.linkLibCpp();

    lib.addCSourceFiles(.{
        .root = dep.path("src"),
        .flags = &[_][]const u8{
            "--std=c++17",
            "-fno-sanitize=undefined",
        },
        .files = &[_][]const u8{
            "blockmapbuilder.cpp",
            "classify.cpp",
            "events.cpp",
            "extract.cpp",
            "gl.cpp",
            "nodebuild.cpp",
            "processor_udmf.cpp",
            "processor.cpp",
            "sc_man.cpp",
            "utility.cpp",
            "wad.cpp",
            "znbx.cpp",
        },
    });

    lib.addIncludePath(dep.path("include"));
    lib.addIncludePath(dep.path("src"));

    compile.linkSystemLibrary2("z", .{
        .preferred_link_mode = .static,
    });

    compile.addSystemIncludePath(dep.path("include"));
    compile.linkLibrary(lib);
}
