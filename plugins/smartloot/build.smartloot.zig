const std = @import("std");

pub fn build(
    b: *std.Build,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
    engine: *std.Build.Module,
) void {
    const lib = b.addSharedLibrary(.{
        .name = "smartloot",
        .root_source_file = b.path("plugins/smartloot/src/root.zig"),
        .target = target,
        .optimize = optimize,
    });

    lib.linkLibC();
    lib.linkLibCpp();

    lib.root_module.addImport("viletech", engine);

    b.installArtifact(lib);
}
