const std = @import("std");

pub fn link(b: *std.Build, compile: *std.Build.Step.Compile) void {
    compile.addCSourceFile(.{
        .file = b.path("depend/flecs/flecs.c"),
        .flags = &[_][]const u8{
            "-std=gnu99",
            "-DFLECS_NO_HTTP",
            "-DFLECS_NO_REST",
        },
    });

    compile.addSystemIncludePath(b.path("depend/flecs"));
}
