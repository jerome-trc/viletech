const std = @import("std");
const builtin = @import("builtin");

pub const Context = struct {
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
    test_step: *std.Build.Step,
    // Libraries
    subterra: ?*std.Build.Module,
};

pub fn build(b: *std.Build) void {
    const posix_like = switch (builtin.os.tag) {
        .linux => true,
        .windows => false,
        else => @compileError("not yet supported"),
    };

    const wad_dir = if (posix_like)
        "/usr/local/share/games/doom"
    else
        ".";

    const simplechecks = b.option(
        bool,
        "simplecheck",
        "Enable checks which only impose significant overhead if a posible error is detected",
    ) orelse true;

    const rangecheck = b.option(
        bool,
        "rangecheck",
        "Enable extra bounds checks in dsda-doom code",
    ) orelse false;

    const cfg_hdr = b.addConfigHeader(.{
        .style = .{ .cmake = b.path("dsda-doom/prboom2/cmake/config.h.cin") },
        .include_path = ".zig-cache/config.h",
    }, .{
        .PACKAGE_NAME = "ratboom",
        .PACKAGE_TARNAME = "ratboom",
        .WAD_DATA = "ratboom.wad",
        // TODO: read from build.zig.zon?
        .PACKAGE_VERSION = "0.0.0",
        .PACKAGE_STRING = "ratboom 0.0.0",

        .VTEC_WAD_DIR = wad_dir,
        .VTEC_ABSOLUTE_PWAD_PATH = wad_dir,

        .WORDS_BIGENDIAN = builtin.cpu.arch.endian() == .big,

        .HAVE_GETOPT = posix_like,
        .HAVE_MMAP = posix_like,
        .HAVE_CREATE_FILE_MAPPING = false,
        .HAVE_STRSIGNAL = posix_like,
        .HAVE_MKSTEMP = posix_like,

        .HAVE_SYS_WAIT_H = posix_like,
        .HAVE_UNISTD_H = posix_like,
        .HAVE_ASM_BYTEORDER_H = posix_like,
        .HAVE_DIRENT_H = posix_like,

        .HAVE_LIBSDL2_IMAGE = true,
        .HAVE_LIBSDL2_MIXER = false,

        .HAVE_LIBDUMB = false,
        .HAVE_LIBFLUIDSYNTH = true,
        .HAVE_LIBMAD = true,
        .HAVE_LIBPORTMIDI = false,
        .HAVE_LIBVORBISFILE = true,

        .SIMPLECHECKS = simplechecks,
        .RANGECHECK = rangecheck,
    });

    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const check = b.step("check", "Semantic check for ZLS");
    const exe = @import("build.client.zig").build(b, target, optimize, check, cfg_hdr);
    const datawad = @import("build.data.zig").data(b, target, cfg_hdr);
    exe.step.dependOn(&datawad.step);

    const demotest_step = b.step("demotest", "Run demo accuracy regression tests");

    const demotest = b.addTest(.{
        .root_source_file = b.path("demotest/main.zig"),
        .target = target,
        // Always use -Doptimize=ReleaseSafe,
        // since we want the demotest to run as quickly as possible.
        .optimize = .ReleaseSafe,
    });
    demotest.step.dependOn(&exe.step);

    const demotest_in = b.addOptions();
    demotest_in.addOption([]const u8, "install_prefix", b.install_prefix);
    demotest.root_module.addOptions("cfg", demotest_in);

    const run_demotest = b.addRunArtifact(demotest);
    demotest_step.dependOn(&run_demotest.step);

    std.fs.cwd().makePath(".zig-cache/fd4rb") catch unreachable;

    // const fd4rb = b.addSharedLibrary(.{
    //     .name = "fd4rb",
    //     .root_source_file = b.path("plugins/fd4rb/src/root.zig"),
    //     .target = target,
    //     .optimize = optimize,
    // });
    // commonDependencies(b, fd4rb, target, optimize);
    // fd4rb.root_module.addImport("ratboom", module);
    // b.installArtifact(fd4rb);

    const fd4rb_decohack_sources = [_][]const u8{
        "plugins/fd4rb/decohack/common.dh",

        "plugins/fd4rb/decohack/borstal-shotgun.dh",
        "plugins/fd4rb/decohack/burst-shotgun.dh",
        "plugins/fd4rb/decohack/plasma-vulcan.dh",
        "plugins/fd4rb/decohack/revolver.dh",
        "plugins/fd4rb/decohack/tornado-battery.dh",
    };

    const fd4rb_decohack = b.addSystemCommand(&[_][]const u8{
        "decohack",
        "--budget",
        "-s",
        ".zig-cache/fd4rb/fd4rb.dh",
        "-o",
        ".zig-cache/fd4rb/fd4rb.deh",
    } ++ fd4rb_decohack_sources);

    for (fd4rb_decohack_sources) |p| {
        fd4rb_decohack.addFileInput(b.path(p));
    }

    exe.step.dependOn(&fd4rb_decohack.step);

    if (std.process.getEnvVarOwned(b.allocator, "DJWAD_DIR")) |path| {
        const dir = std.fs.openDirAbsolute(path, .{}) catch unreachable;
        @import("tunetech").djwad(b.allocator, dir) catch unreachable;
    } else |_| {}

    const vilebuild = b.addExecutable(.{
        .name = "vilebuild",
        .root_source_file = b.path("vilebuild/main.zig"),
        .target = target,
        .optimize = .Debug,
    });
    vilebuild.linkLibC();
    b.installArtifact(vilebuild);

    const dehpp = b.addRunArtifact(vilebuild);
    dehpp.step.dependOn(&fd4rb_decohack.step);
    dehpp.addFileInput(b.path(".zig-cache/fd4rb/fd4rb.deh"));
    exe.step.dependOn(&dehpp.step);

    const test_step = b.step("test", "Run unit test suite");
    subterra.tests(b, target, optimize, test_step);
    wadload.tests(b, target, optimize, test_step);
}

// fn commonDependencies(
//     b: *std.Build,
//     compile: *std.Build.Step.Compile,
//     _: std.Build.ResolvedTarget,
//     _: std.builtin.OptimizeMode,
// ) void {
//     const zig_args = b.dependency("zig-args", .{});

//     compile.linkLibC();
//     compile.linkLibCpp();
//     compile.addIncludePath(b.path("build"));
//     compile.addIncludePath(b.path("dsda-doom/prboom2/src"));

//     if (compile.kind == .lib and compile.linkage != .dynamic) {
//         compile.bundle_compiler_rt = true;
//         compile.pie = true;
//     }

//     @import("depend/build.cimgui.zig").link(b, compile);
//     compile.root_module.addImport("zig-args", zig_args.module("args"));
// }

pub const subterra = struct {
    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, name: ?[]const u8) void {
        compile.root_module.addImport(
            name orelse "subterra",
            b.addModule("subterra", .{
                .root_source_file = b.path("libs/subterra/src/root.zig"),
            }),
        );
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/subterra/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};

pub const wadload = struct {
    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, name: ?[]const u8) void {
        compile.root_module.addImport(
            name orelse "wadload",
            b.addModule("wadload", .{
                .root_source_file = b.path("libs/wadload/src/root.zig"),
            }),
        );
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/wadload/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};
