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
}
