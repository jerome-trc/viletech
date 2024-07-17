const std = @import("std");

const client = @import("client/build.sub.zig");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Similar to creating the run step earlier, this exposes a `test` step to
    // the `zig build --help` menu, providing a way for the user to request
    // running the unit tests.
    const test_step = b.step("test", "Run unit tests");

    @import("libs/subterra/build.sub.zig").build(b, target, optimize, test_step);

    const engine = client.build(b, target, optimize, test_step);

    @import("plugins/smartloot/build.smartloot.zig").build(b, target, optimize, engine);
}
