const std = @import("std");

pub fn link(
    b: *std.Build,
    compile: *std.Build.Step.Compile,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) void {
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
        "-I",
        "depend/zdfs/include",
        "-I",
        "depend/zdfs/src",
        "-isystem",
        "depend/zdfs/bzip2",
        "-isystem",
        "depend/zdfs/lzma/C",
        "-isystem",
        "depend/zdfs/miniz",
        "-isystem",
        "depend/zdfs/utf8proc",
    };

    const c_flags = compile_flags ++ [_][]const u8{"--std=c17"};
    const cxx_flags = compile_flags ++ [_][]const u8{"--std=c++20"};

    lib.addCSourceFiles(.{
        .root = b.path("depend/zdfs/bzip2"),
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
        .root = b.path("depend/zdfs/lzma/C"),
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
        .root = b.path("depend/zdfs/miniz"),
        .flags = &c_flags,
        .files = &[_][]const u8{"miniz.c"},
    });

    lib.addCSourceFiles(.{
        .root = b.path("depend/zdfs/utf8proc"),
        .flags = &c_flags,
        .files = &[_][]const u8{"utf8proc.c"},
    });

    lib.addCSourceFiles(.{
        .root = b.path("depend/zdfs/src"),
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

    compile.linkLibrary(lib);
    compile.addSystemIncludePath(b.path("depend/zdfs/include"));
}
