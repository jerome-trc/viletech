const std = @import("std");

const client = @import("client/build.sub.zig");
const mus2mid = @import("libs/mus2mid/build.sub.zig");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Similar to creating the run step earlier, this exposes a `test` step to
    // the `zig build --help` menu, providing a way for the user to request
    // running the unit tests.
    const test_step = b.step("test", "Run unit tests");

    client.build(b, target, optimize, test_step);
    mus2mid.build(b, target, optimize, test_step);
}
