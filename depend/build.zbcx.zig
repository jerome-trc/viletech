const std = @import("std");

pub fn link(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) void {
    var symln_buf = [1]u8{0} ** std.c.PATH_MAX;

    const root = if (std.fs.cwd().readLink("depend/zbcx.ln", symln_buf[0..])) |_|
        "depend/zbcx.ln"
    else |_|
        "depend/zbcx";

    var lib = b.addStaticLibrary(.{
        .name = "zbcx",
        .target = target,
        .optimize = optimize,
    });

    lib.linkLibC();

    lib.addCSourceFiles(.{
        .root = b.path(root),
        .flags = &[_][]const u8{
            "--std=c99",
            "-I",
            b.pathJoin(&[_][]const u8{ root, "include" }),
            "-I",
            b.pathJoin(&[_][]const u8{ root, "src" }),
            "-I",
            b.pathJoin(&[_][]const u8{ root, "src/cache" }),
            "-I",
            b.pathJoin(&[_][]const u8{ root, "src/codegen" }),
            "-I",
            b.pathJoin(&[_][]const u8{ root, "src/parse" }),
            "-I",
            b.pathJoin(&[_][]const u8{ root, "src/semantic" }),
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

    compile.addSystemIncludePath(b.path(b.pathJoin(&[_][]const u8{ root, "include" })));

    compile.linkLibrary(lib);
}
