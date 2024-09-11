const std = @import("std");
const builtin = @import("builtin");

pub const ccdb = @import("depend/ccdb.zig");
pub const cimgui = @import("depend/build.cimgui.zig");
pub const datetime = @import("depend/datetime.zig");
pub const zmsx = @import("depend/build.zmsx.zig");

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
    const ratboom = @import("ratboom/build.ratboom.zig").build(b, target, optimize, check, cfg_hdr);
    const datawad = @import("ratboom/build.data.zig").data(b, target, cfg_hdr);
    ratboom.step.dependOn(&datawad.step);

    const demotest_step = b.step("demotest", "Run demo accuracy regression tests");

    const demotest = b.addTest(.{
        .root_source_file = b.path("demotest/main.zig"),
        .target = target,
        // Always use -Doptimize=ReleaseSafe,
        // since we want the demotest to run as quickly as possible.
        .optimize = .ReleaseSafe,
    });
    demotest.step.dependOn(&ratboom.step);

    const demotest_in = b.addOptions();
    demotest_in.addOption([]const u8, "install_prefix", b.install_prefix);
    demotest.root_module.addOptions("cfg", demotest_in);

    const run_demotest = b.addRunArtifact(demotest);
    demotest_step.dependOn(&run_demotest.step);

    const testx = b.option(
        []const []const u8,
        "testx",
        "Extra features to test",
    );

    const test_step = b.step("test", "Run unit test suite");
    engine.tests(b, target, optimize, test_step);
    doomparse.tests(b, target, optimize, test_step);
    subterra.tests(b, target, optimize, test_step, testx);
    wadload.tests(b, target, optimize, test_step);

    const doc_step = b.step("doc", "Generate documentation");
    engine.doc(b, target, optimize, doc_step);
    doomparse.doc(b, target, optimize, doc_step);
    subterra.doc(b, target, optimize, doc_step);
    wadload.doc(b, target, optimize, doc_step);

    if (std.process.getEnvVarOwned(b.allocator, "DJWAD_DIR")) |path| {
        const dir = std.fs.openDirAbsolute(path, .{}) catch unreachable;
        @import("tunetech").djwad(b.allocator, dir) catch unreachable;
    } else |_| {}

    @import("client/build.client.zig").build(b, target, optimize);
}

pub const engine = struct {
    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, name: ?[]const u8) void {
        compile.root_module.addImport(
            name orelse "viletech",
            b.addModule("viletech", .{
                .root_source_file = b.path("engine/src/root.zig"),
            }),
        );
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "viletech",
            .root_source_file = b.path("engine/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "viletech",
        });

        doc_step.dependOn(&install_docs.step);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("engine/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};

// Libraries ///////////////////////////////////////////////////////////////////

pub const doomparse = struct {
    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, name: ?[]const u8) void {
        const module = b.addModule("doomparse", .{
            .root_source_file = b.path("libs/doomparse/src/root.zig"),
        });

        module.addImport("deque", b.addModule("deque", .{
            .root_source_file = b.path("depend/deque.zig"),
        }));

        compile.root_module.addImport(name orelse "doomparse", module);
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "doomparse",
            .root_source_file = b.path("libs/doomparse/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "doomparse",
        });

        doc_step.dependOn(&install_docs.step);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/doomparse/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};

pub const subterra = struct {
    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, config: struct {
        name: []const u8 = "subterra",
        znbx: union(enum) {
            off: void,
            staticlib: struct {
                target: std.Build.ResolvedTarget,
                optimize: std.builtin.OptimizeMode,
            },
            source: void,
        },
    }) void {
        const module = b.addModule("subterra", .{
            .root_source_file = b.path("libs/subterra/src/root.zig"),
        });

        const opts = b.addOptions();
        opts.addOption(bool, "znbx", config.znbx != .off);
        compile.root_module.addOptions("cfg", opts);

        compile.root_module.addImport(config.name, module);

        switch (config.znbx) {
            .off => {},
            .staticlib => |tgt_and_opt| {
                const znbx = b.addStaticLibrary(.{
                    .name = "znbx",
                    .target = tgt_and_opt.target,
                    .optimize = tgt_and_opt.optimize,
                });

                linkZnbx(b, znbx);
                compile.linkLibrary(znbx);
            },
            .source => {
                linkZnbx(b, compile);
            },
        }
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "subterra",
            .root_source_file = b.path("libs/subterra/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "subterra",
        });

        doc_step.dependOn(&install_docs.step);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
        o_testx: ?[]const []const u8,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/subterra/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        var dmxgus: []const u8 = "";
        var genmidi: []const u8 = "";
        var znbx = false;

        if (o_testx) |testx| {
            for (testx) |s| {
                if (std.mem.eql(u8, std.fs.path.stem(s), "GENMIDI")) {
                    genmidi = s;
                } else if (std.mem.eql(u8, std.fs.path.stem(s), "DMXGUS")) {
                    dmxgus = s;
                } else if (std.mem.eql(u8, s, "znbx")) {
                    znbx = true;
                }
            }
        }

        const opts = b.addOptions();
        opts.addOption([]const u8, "dmxgus_sample", dmxgus);
        opts.addOption([]const u8, "genmidi_sample", genmidi);
        opts.addOption(bool, "znbx", znbx);
        unit_tests.root_module.addOptions("cfg", opts);

        if (znbx) {
            linkZnbx(b, unit_tests);
        }

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }

    fn linkZnbx(b: *std.Build, compile: *std.Build.Step.Compile) void {
        if (!compile.is_linking_libc) {
            compile.linkLibC();
        }

        if (!compile.is_linking_libcpp) {
            compile.linkLibCpp();
        }

        compile.linkSystemLibrary2("z", .{
            .preferred_link_mode = .static,
        });

        compile.addSystemIncludePath(b.path("depend/znbx/include"));

        compile.addCSourceFiles(.{
            .root = b.path("depend/znbx"),
            .flags = &[_][]const u8{
                "--std=c++17",
                "-Idepend/znbx/include",
                "-Idepend/znbx/src",
                "-fno-sanitize=undefined",
            },
            .files = &[_][]const u8{
                "src/blockmapbuilder.cpp",
                "src/classify.cpp",
                "src/events.cpp",
                "src/extract.cpp",
                "src/gl.cpp",
                "src/nodebuild.cpp",
                "src/processor_udmf.cpp",
                "src/processor.cpp",
                "src/sc_man.cpp",
                "src/utility.cpp",
                "src/wad.cpp",
                "src/znbx.cpp",
            },
        });
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

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "wadload",
            .root_source_file = b.path("libs/wadload/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "wadload",
        });

        doc_step.dependOn(&install_docs.step);
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
