const std = @import("std");

pub fn link(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) void {
    var lib = b.addStaticLibrary(.{
        .name = "zbcx",
        .target = target,
        .optimize = optimize,
    });

    lib.linkLibC();

    lib.addCSourceFiles(.{
        .root = b.path("depend/zbcx"),
        .flags = &[_][]const u8{
            "--std=c99",
            "-I",
            "depend/zbcx/include",
            "-I",
            "depend/zbcx/src",
            "-I",
            "depend/zbcx/src/cache",
            "-I",
            "depend/zbcx/src/codegen",
            "-I",
            "depend/zbcx/src/parse",
            "-I",
            "depend/zbcx/src/semantic",
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

    compile.addSystemIncludePath(b.path("depend/zbcx/include"));

    compile.linkLibrary(lib);
}
