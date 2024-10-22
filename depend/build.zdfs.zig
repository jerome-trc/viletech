const std = @import("std");

pub fn link(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) void {
    const dep = b.dependency("zdfs", .{});

    var lib = b.addStaticLibrary(.{
        .name = "zdfs",
        .target = target,
        .optimize = optimize,
    });
    lib.linkLibC();
    lib.linkLibCpp();

    const compile_flags = [_][]const u8{
        "-Dstricmp=strcasecmp",
        "-Dstrnicmp=strncasecmp",
        "-DBZ_NO_STDIO",
        "-DMINIZ_NO_STDIO",
        "-DZ7_PPMD_SUPPORT",
        "-fPIC",
    };

    const c_flags = compile_flags ++ [_][]const u8{
        "--std=c17",
        // TODO: some load-bearing UB here still. Slowly move files off this flag.
        "-fno-sanitize=undefined",
    };
    const cxx_flags = compile_flags ++ [_][]const u8{
        "--std=c++20",
        "-fno-sanitize=undefined",
    };

    lib.addCSourceFiles(.{
        .root = dep.path("bzip2"),
        .flags = &c_flags,
        .files = &[_][]const u8{
            "blocksort.c",
            "bzlib.c",
            "compress.c",
            "crctable.c",
            "decompress.c",
            "huffman.c",
            "randtable.c",
        },
    });

    lib.addCSourceFiles(.{
        .root = dep.path("lzma/C"),
        .flags = &c_flags,
        .files = &[_][]const u8{
            "7zAlloc.c",
            "7zArcIn.c",
            "7zBuf2.c",
            "7zBuf.c",
            "7zCrc.c",
            "7zCrcOpt.c",
            "7zDec.c",
            "7zFile.c",
            "7zStream.c",
            "Alloc.c",
            "Bcj2.c",
            "Bcj2Enc.c",
            "Bra86.c",
            "Bra.c",
            "CpuArch.c",
            "Delta.c",
            "DllSecur.c",
            "LzFind.c",
            "LzFindMt.c",
            "LzFindOpt.c",
            "Lzma2Dec.c",
            "Lzma2DecMt.c",
            "Lzma2Enc.c",
            "LzmaDec.c",
            "LzmaEnc.c",
            "LzmaLib.c",
            "MtCoder.c",
            "MtDec.c",
            "Ppmd7.c",
            "Ppmd7Dec.c",
            "Ppmd7Enc.c",
            "Sha256.c",
            "Sha256Opt.c",
            "Sort.c",
            "SwapBytes.c",
            "Threads.c",
            "Xz.c",
            "XzCrc64.c",
            "XzCrc64Opt.c",
            "XzDec.c",
            "XzEnc.c",
            "XzIn.c",
        },
    });

    lib.addCSourceFiles(.{
        .root = dep.path("miniz"),
        .flags = &c_flags,
        .files = &[_][]const u8{"miniz.c"},
    });

    lib.addCSourceFiles(.{
        .root = dep.path("utf8proc"),
        .flags = &c_flags,
        .files = &[_][]const u8{"utf8proc.c"},
    });

    lib.addCSourceFiles(.{
        .root = dep.path("src"),
        .flags = &cxx_flags,
        .files = &[_][]const u8{
            "7z.cpp",
            "ancientzip.cpp",
            "critsec.cpp",
            "decompress.cpp",
            "directory.cpp",
            "files.cpp",
            "filesystem.cpp",
            "findfile.cpp",
            "grp.cpp",
            "hog.cpp",
            "lump.cpp",
            "mvl.cpp",
            "pak.cpp",
            "resourcefile.cpp",
            "rff.cpp",
            "ssi.cpp",
            "stringpool.cpp",
            "unicode.cpp",
            "wad.cpp",
            "whres.cpp",
            "zdfs.h.cpp",
            "zip.cpp",
        },
    });

    lib.addIncludePath(dep.path("include"));
    lib.addIncludePath(dep.path("src"));
    lib.addSystemIncludePath(dep.path("bzip2"));
    lib.addSystemIncludePath(dep.path("lzma/C"));
    lib.addSystemIncludePath(dep.path("miniz"));
    lib.addSystemIncludePath(dep.path("utf8proc"));

    compile.linkLibrary(lib);
    compile.addSystemIncludePath(dep.path("include"));
}
