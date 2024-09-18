const std = @import("std");
const builtin = @import("builtin");

pub const zon = @embedFile("build.zig.zon");

pub const ccdb = @import("depend/ccdb.zig");
pub const cimgui = @import("depend/build.cimgui.zig");
pub const datetime = @import("depend/datetime.zig");
pub const zmsx = @import("depend/build.zmsx.zig");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const assets = b.createModule(.{
        .root_source_file = b.path("assets/assets.zig"),
        .target = target,
        .optimize = optimize,
    });
    const deque = b.createModule(.{
        .root_source_file = b.path("depend/deque.zig"),
        .target = target,
        .optimize = optimize,
    });
    const zig_args = b.dependency("zig-args", .{});

    const check = b.step("check", "Semantic check for ZLS");
    const ratboom = @import("ratboom/build.ratboom.zig").build(b, target, optimize, check);
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
    zbcx.tests(b, target, optimize, test_step);

    const doc_step = b.step("doc", "Generate documentation");
    engine.doc(b, target, optimize, doc_step);
    doomparse.doc(b, target, optimize, doc_step);
    subterra.doc(b, target, optimize, doc_step);
    wadload.doc(b, target, optimize, doc_step);
    zbcx.doc(b, target, optimize, doc_step);

    const re2_step = b.step("re2", "Run all re2zig lexer generators");
    subterra.generateUdmfLexer(b, re2_step);

    if (std.process.getEnvVarOwned(b.allocator, "DJWAD_DIR")) |path| {
        const dir = std.fs.openDirAbsolute(path, .{}) catch unreachable;
        @import("tunetech").djwad(b.allocator, dir) catch unreachable;
    } else |_| {}

    var client_builder = @import("client/Builder.zig"){
        .b = b,
        .target = target,
        .optimize = optimize,
        .check = check,
        .assets = assets,
        .deque = deque,
        .zig_args = zig_args.module("args"),
    };
    client_builder.build();
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

    fn generateUdmfLexer(b: *std.Build, re2_step: *std.Build.Step) void {
        const run = b.addSystemCommand(&[_][]const u8{
            "re2zig",
            "--lang",
            "zig",
            "--api",
            "default",
            "-i",
            "--loop-switch",
            "--case-ranges",
            "-W",
            "libs/subterra/src/UdmfLexer.zig.re",
            "-o",
            "libs/subterra/src/UdmfLexer.zig",
        });

        run.addFileInput(b.path("libs/subterra/src/UdmfLexer.zig.re"));
        re2_step.dependOn(&run.step);
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

pub const zbcx = struct {
    const c = @import("depend/build.zbcx.zig");

    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, config: struct {
        name: []const u8 = "zbcx",
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
    }) void {
        c.link(b, compile, config.target, config.optimize);

        const module = b.addModule("zbcx", .{
            .root_source_file = b.path("libs/zbcx/src/root.zig"),
        });

        compile.root_module.addImport(config.name, module);
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "zbcx",
            .root_source_file = b.path("libs/zbcx/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        c.link(b, dummy, target, optimize);

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "zbcx",
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
            .root_source_file = b.path("libs/zbcx/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const sample = b.createModule(.{
            .root_source_file = b.path("depend/sample.zig"),
            .target = target,
            .optimize = optimize,
        });
        unit_tests.root_module.addImport("depsample", sample);

        c.link(b, unit_tests, target, optimize);

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};

pub fn packageVersion() []const u8 {
    const zon_vers_start = std.mem.indexOf(u8, zon, ".version = ").?;
    const zon_vers_end = std.mem.indexOfPos(u8, zon, zon_vers_start, ",").?;
    const zon_vers_kvp = zon[zon_vers_start..zon_vers_end];
    return std.mem.trim(u8, zon_vers_kvp, ".version =\"");
}
