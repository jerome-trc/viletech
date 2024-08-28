//! For generating a [compilation commands database].
//! https://github.com/xxxbrian/zig-compile-commands
//! This work is provided with no license, and thus used as such.
//!
//! [compilation commands database]: https://clang.llvm.org/docs/JSONCompilationDatabase.html

const std = @import("std");

const CSourceFiles = std.Build.Module.CSourceFiles;

const zig_version = @import("builtin").zig_version;

const CompileCommandEntry = struct {
    arguments: []const []const u8,
    directory: []const u8,
    file: []const u8,
    output: []const u8,
};

const CompileCommandOptions = struct {
    target: ?*std.Build.Step.Compile = null,
    targets: []const *std.Build.Step.Compile = &.{},
    include_emitted_include_trees: bool = false,
};

// TODO use @fieldParentPtr to get this from the step instead of relying on a global
var the_options: CompileCommandOptions = undefined;

/// Note: you must call this *after* all other configuration in your build script!
pub fn createStep(b: *std.Build, name: []const u8, options: CompileCommandOptions) void {
    const step = b.allocator.create(std.Build.Step) catch @panic("Allocation failure, probably OOM");

    the_options = .{
        .target = options.target,
        .targets = b.allocator.dupe(*std.Build.Step.Compile, options.targets) catch @panic("OOM"),
        .include_emitted_include_trees = options.include_emitted_include_trees,
    };

    step.* = std.Build.Step.init(.{
        .id = .custom,
        .name = "cc_file",
        .makeFn = makeCdb,
        .owner = b,
    });

    var compile_steps_list = std.ArrayList(*std.Build.Step.Compile).init(b.allocator);
    if (options.target) |target| compile_steps_list.append(target) catch @panic("OOM");
    compile_steps_list.appendSlice(options.targets) catch @panic("OOM");

    var index: u32 = 0;

    // list may be appended to during the loop, so use a while
    while (index < compile_steps_list.items.len) {
        const compile_step = compile_steps_list.items[index];

        for (compile_step.root_module.include_dirs.items) |include_dir| {
            switch (include_dir) {
                .other_step => |compile| {
                    for (compile.installed_headers.items) |install_header| {
                        switch (install_header) {
                            .file => {},
                            .directory => |dir| {
                                dir.source.addStepDependencies(step);
                            },
                        }
                    }
                    if (options.include_emitted_include_trees) {
                        _ = compile.getEmittedIncludeTree();
                        step.dependOn(&compile.installed_headers_include_tree.?.step);
                    }
                },
                .path, .path_system, .path_after, .framework_path, .framework_path_system => |path| {
                    path.addStepDependencies(step);
                },
                .config_header_step => {}, // TODO
            }
        }

        for (compile_step.root_module.link_objects.items) |link_object| {
            switch (link_object) {
                .static_path, .system_lib, .assembly_file, .win32_resource_file => continue,
                .other_step => {
                    compile_steps_list.append(link_object.other_step) catch @panic("OOM");
                },
                .c_source_file => |c_source_file| {
                    // convert C source file into c source fileS
                    c_source_file.file.addStepDependencies(step);
                },
                .c_source_files => |c_source_files| {
                    c_source_files.root.addStepDependencies(step);
                },
            }
        }
        index += 1;
    }

    const cdb_step = b.step(name, "Create compile_commands.json");
    cdb_step.dependOn(step);
}

/// Errors turning the build graph into compile command strings.
const GraphSerializeError = error{InvalidHeader};

/// A compilation step has an "include_dirs" array list, which contains paths as
/// well as other compile steps. This loops until all the include directories
/// necessary for good intellisense on the files compile by this step are found.
pub fn extractIncludeDirsFromCompileStep(b: *std.Build, step: *std.Build.Step.Compile, include_emitted_include_trees: bool) []const []const u8 {
    var dirs = std.ArrayList([]const u8).init(b.allocator);

    for (step.root_module.include_dirs.items) |include_dir| {
        switch (include_dir) {
            .other_step => |compile| {
                for (compile.installed_headers.items) |install_header| {
                    switch (install_header) {
                        .file => {},
                        .directory => |dir| {
                            dirs.append(dir.source.getPath(compile.step.owner)) catch @panic("OOM");
                        },
                    }
                }

                if (include_emitted_include_trees) {
                    dirs.append(compile.getEmittedIncludeTree().getPath(compile.step.owner)) catch @panic("OOM");
                }
            },
            .path, .path_system, .path_after, .framework_path, .framework_path_system => |path| {
                dirs.append(path.getPath(b)) catch @panic("OOM");
            },
            // TODO: support this
            .config_header_step => {},
        }
    }

    return dirs.toOwnedSlice() catch @panic("OOM");
}

const CSources = struct {
    c_source_files: *CSourceFiles,
    compile: *std.Build.Step.Compile,
};

// NOTE: some of the CSourceFiles pointed at by the elements of the returned
// array are allocated with the allocator, some are not.
fn getCSources(b: *std.Build, options: CompileCommandOptions) []CSources {
    var allocator = b.allocator;
    var res = std.ArrayList(CSources).init(allocator);

    // move the compile steps into a mutable dynamic array, so we can add
    // any child steps
    var compile_steps_list = std.ArrayList(*std.Build.Step.Compile).init(b.allocator);
    if (options.target) |target| compile_steps_list.append(target) catch @panic("OOM");
    compile_steps_list.appendSlice(options.targets) catch @panic("OOM");

    var index: u32 = 0;

    // list may be appended to during the loop, so use a while
    while (index < compile_steps_list.items.len) {
        const compile = compile_steps_list.items[index];

        var shared_flags = std.ArrayList([]const u8).init(allocator);

        // catch all the system libraries being linked, make flags out of them
        for (compile.root_module.link_objects.items) |link_object| {
            switch (link_object) {
                .system_lib => |lib| shared_flags.append(linkFlag(allocator, lib.name)) catch @panic("OOM"),
                else => {},
            }
        }

        if (compile.is_linking_libc) {
            shared_flags.append(linkFlag(allocator, "c")) catch @panic("OOM");
        }
        if (compile.is_linking_libcpp) {
            shared_flags.append(linkFlag(allocator, "c++")) catch @panic("OOM");
        }

        // make flags out of all include directories
        for (extractIncludeDirsFromCompileStep(b, compile, options.include_emitted_include_trees)) |include_dir| {
            shared_flags.append(includeFlag(b.allocator, include_dir)) catch @panic("OOM");
        }

        for (compile.root_module.link_objects.items) |link_object| {
            switch (link_object) {
                .static_path, .system_lib, .assembly_file, .win32_resource_file => continue,
                .other_step => {
                    compile_steps_list.append(link_object.other_step) catch @panic("OOM");
                },
                .c_source_file => {
                    // convert C source file into c source fileS
                    const path = link_object.c_source_file.file.getPath(b);
                    var files_mem = allocator.alloc([]const u8, 1) catch @panic("Allocation failure, probably OOM");
                    files_mem[0] = path;

                    const source_file = allocator.create(CSourceFiles) catch @panic("Allocation failure, probably OOM");

                    var flags = std.ArrayList([]const u8).init(allocator);
                    flags.appendSlice(link_object.c_source_file.flags) catch @panic("OOM");
                    flags.appendSlice(shared_flags.items) catch @panic("OOM");

                    source_file.* = CSourceFiles{
                        .root = b.path("."),
                        .files = files_mem,
                        .flags = flags.toOwnedSlice() catch @panic("OOM"),
                    };

                    res.append(.{
                        .c_source_files = source_file,
                        .compile = compile,
                    }) catch @panic("OOM");
                },
                .c_source_files => {
                    var source_files = link_object.c_source_files;
                    var flags = std.ArrayList([]const u8).init(allocator);
                    flags.appendSlice(source_files.flags) catch @panic("OOM");
                    flags.appendSlice(shared_flags.items) catch @panic("OOM");
                    source_files.flags = flags.toOwnedSlice() catch @panic("OOM");

                    res.append(.{
                        .c_source_files = source_files,
                        .compile = compile,
                    }) catch @panic("OOM");
                },
            }
        }
        index += 1;
    }

    return res.toOwnedSlice() catch @panic("OOM");
}

const Progress_Node = if (zig_version.major > 0 or zig_version.minor >= 13) std.Progress.Node else *std.Progress.Node;
fn makeCdb(step: *std.Build.Step, prog_node: Progress_Node) anyerror!void {
    _ = prog_node;
    const allocator = step.owner.allocator;

    var compile_commands = std.ArrayList(CompileCommandEntry).init(allocator);
    defer compile_commands.deinit();

    // initialize file and struct containing its future contents
    const cwd: std.fs.Dir = std.fs.cwd();
    var file = try cwd.createFile("compile_commands.json", .{});
    defer file.close();

    const cwd_string = try dirToString(cwd, allocator);
    const c_sources = getCSources(step.owner, the_options);

    // fill compile command entries, one for each file
    for (c_sources) |sources| {
        const flags = sources.c_source_files.flags;
        for (sources.c_source_files.files) |c_file| {
            const root = sources.c_source_files.root.getPath(sources.compile.step.owner);
            const file_str = try std.fs.path.resolve(allocator, &.{ root, c_file });
            const output_str = std.fmt.allocPrint(allocator, "{s}.o", .{file_str}) catch @panic("OOM");

            var arguments = std.ArrayList([]const u8).init(allocator);
            // pretend this is clang compiling
            arguments.append("clang") catch @panic("OOM");
            arguments.append(c_file) catch @panic("OOM");
            arguments.appendSlice(&.{ "-o", std.fmt.allocPrint(allocator, "{s}.o", .{c_file}) catch @panic("OOM") }) catch @panic("OOM");
            arguments.appendSlice(flags) catch @panic("OOM");
            arguments.appendSlice(sources.compile.root_module.c_macros.items) catch @panic("OOM");

            // add host native include dirs and libs
            // (doesn't really help unless your include dirs change after generating this)
            // {
            //     var native_paths = try std.zig.system.NativePaths.detect(allocator, step.owner.host);
            //     defer native_paths.deinit();
            //     // native_paths also has lib_dirs. probably not relevant to clangd and compile_commands.json
            //     for (native_paths.include_dirs.items) |include_dir| {
            //         try arguments.append(try common.includeFlag(allocator, include_dir));
            //     }
            // }

            const entry = CompileCommandEntry{
                .arguments = arguments.toOwnedSlice() catch @panic("OOM"),
                .output = output_str,
                .file = file_str,
                .directory = cwd_string,
            };
            compile_commands.append(entry) catch @panic("OOM");
        }
    }

    try writeCompileCommands(&file, compile_commands.items);
}

fn writeCompileCommands(file: *std.fs.File, compile_commands: []CompileCommandEntry) !void {
    const options = std.json.StringifyOptions{
        .whitespace = .indent_tab,
        .emit_null_optional_fields = false,
    };

    try std.json.stringify(compile_commands, options, file.*.writer());
}

fn dirToString(dir: std.fs.Dir, allocator: std.mem.Allocator) ![]const u8 {
    var real_dir = try dir.openDir(".", .{});
    defer real_dir.close();
    return std.fs.realpathAlloc(allocator, ".") catch |err| {
        std.debug.print("error encountered in converting directory to string.\n", .{});
        return err;
    };
}

fn linkFlag(ally: std.mem.Allocator, lib: []const u8) []const u8 {
    return std.fmt.allocPrint(ally, "-l{s}", .{lib}) catch @panic("OOM");
}

fn includeFlag(ally: std.mem.Allocator, path: []const u8) []const u8 {
    return std.fmt.allocPrint(ally, "-I{s}", .{path}) catch @panic("OOM");
}
