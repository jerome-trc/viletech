const std = @import("std");

pub fn link(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) void {
    const dep = b.dependency("zbcx", .{});

    var lib = b.addStaticLibrary(.{
        .name = "zbcx",
        .target = target,
        .optimize = optimize,
    });

    lib.linkLibC();

    lib.addCSourceFiles(.{
        .root = dep.path("."),
        .flags = &[_][]const u8{
            "--std=c99",
            // TODO: some load-bearing UB here still. Slowly move files off this flag.
            "-fno-sanitize=undefined",
        },
        .files = &[_][]const u8{
            "src/builtin.c",
            "src/cache/archive.c",
            "src/cache/cache.c",
            "src/cache/field.c",
            "src/cache/library.c",
            "src/codegen/asm.c",
            "src/codegen/chunk.c",
            "src/codegen/dec.c",
            "src/codegen/expr.c",
            "src/codegen/linear.c",
            "src/codegen/obj.c",
            "src/codegen/pcode.c",
            "src/codegen/phase.c",
            "src/codegen/stmt.c",
            "src/common.c",
            "src/gbuf.c",
            "src/parse/asm.c",
            "src/parse/dec.c",
            "src/parse/expr.c",
            "src/parse/library.c",
            "src/parse/phase.c",
            "src/parse/stmt.c",
            "src/parse/token/dirc.c",
            "src/parse/token/expr.c",
            "src/parse/token/info.c",
            "src/parse/token/output.c",
            "src/parse/token/queue.c",
            "src/parse/token/source.c",
            "src/parse/token/stream.c",
            "src/parse/token/user.c",
            "src/semantic/asm.c",
            "src/semantic/dec.c",
            "src/semantic/expr.c",
            "src/semantic/phase.c",
            "src/semantic/stmt.c",
            "src/semantic/type.c",
            "src/task.c",
            "src/version.c",
            "src/zbcx.c",
        },
    });

    lib.addIncludePath(dep.path("include"));
    lib.addIncludePath(dep.path("src"));
    lib.addIncludePath(dep.path("src/cache"));
    lib.addIncludePath(dep.path("src/codegen"));
    lib.addIncludePath(dep.path("src/parse"));
    lib.addIncludePath(dep.path("src/semantic"));

    compile.addSystemIncludePath(dep.path("include"));

    compile.linkLibrary(lib);
}
