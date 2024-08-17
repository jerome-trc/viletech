const std = @import("std");

const Context = @import("../../build.zig").Context;

pub fn build(b: *std.Build, ctx: *const Context) *std.Build.Module {
    const mod = b.addModule("subterra", .{
        .root_source_file = b.path("libs/subterra/src/root.zig"),
    });

    const unit_tests = b.addTest(.{
        .root_source_file = b.path("libs/subterra/src/root.zig"),
        .target = ctx.target,
        .optimize = ctx.optimize,
    });

    const run_unit_tests = b.addRunArtifact(unit_tests);
    ctx.test_step.dependOn(&run_unit_tests.step);

    return mod;
}
