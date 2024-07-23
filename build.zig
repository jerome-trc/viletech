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

    lib.linkLibC();
    lib.linkLibCpp();
    lib.addIncludePath(b.path("dsda-doom/prboom2/src"));
    lib.bundle_compiler_rt = true;
    lib.pie = true;

    b.installArtifact(lib);

    const lib_check = b.addStaticLibrary(.{
        .name = "ratboomzig",
        .target = target,
        .optimize = optimize,
        .root_source_file = b.path("client/src/main.zig"),
    });

    lib_check.linkLibC();
    lib_check.linkLibCpp();
    lib_check.addIncludePath(b.path("dsda-doom/prboom2/src"));
    lib_check.bundle_compiler_rt = true;
    lib_check.pie = true;

    const check = b.step("check", "Semantic check for ZLS");
    check.dependOn(&lib_check.step);
}
