const std = @import("std");

pub const Context = struct {
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
    test_step: *std.Build.Step,
    // Libraries
    subterra: ?*std.Build.Module,
};

pub fn build(b: *std.Build) void {
    var ctx = Context{
        .target = b.standardTargetOptions(.{}),
        .optimize = b.standardOptimizeOption(.{}),
        .test_step = b.step("test", "Run unit tests"),

        .subterra = null,
    };

    ctx.subterra = @import("libs/subterra/build.subterra.zig").build(b, &ctx);

    const engine = @import("client/build.client.zig").build(b, &ctx);

    @import("plugins/smartloot/build.smartloot.zig").build(
        b,
        ctx.target,
        ctx.optimize,
        engine,
    );
}
