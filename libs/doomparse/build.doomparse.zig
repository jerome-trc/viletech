const std = @import("std");
const zig_version = @import("builtin").zig_version;

const ProgressNode = if (zig_version.major > 0 or zig_version.minor >= 13)
    std.Progress.Node
else
    *std.Progress.Node;

const src_dirs = std.StaticStringMap([]const u8).initComptime(.{
    .{ "wadmerge", "doomtools/wadmerge" },
});

const Fixup = struct {
    step: std.Build.Step,
    src_dir: []const u8,
};

pub fn createStep(b: *std.Build) void {
    if (b.findProgram(&[_][]const u8{"tree-sitter"}, &[_][]const u8{})) |_| {} else |_| {
        @panic("Failed to find Tree-sitter (is it in your PATH)?");
    }

    const lang = if (b.option(
        []const u8,
        "tslang",
        "Language for which to generate a Tree-sitter parser",
    )) |opt| opt else {
        // `--help` needs to have something to show if the user doesn't pass -Dtslang.
        // TODO: try `std.Build.addFail` when 0.14.0 lands. Might be better for this.

        const fail = b.allocator.create(std.Build.Step) catch @panic("Allocation failure");

        fail.* = std.Build.Step.init(.{
            .id = .custom,
            .name = "tree-sitter-fail",
            .makeFn = struct {
                fn make(_: *std.Build.Step, _: ProgressNode) anyerror!void {
                    @panic("Step `tree-sitter` requires `-Dtslang`");
                }
            }.make,
            .owner = b,
        });

        b.step("tree-sitter", "Generate Tree-sitter for a DoomParse language")
            .dependOn(fail);

        return;
    };

    const src_dir = src_dirs.get(lang) orelse
        std.debug.panic("Unknown language: {s}", .{lang});

    const fixup = b.allocator.create(Fixup) catch @panic("Allocation failure");

    fixup.* = Fixup{
        .step = std.Build.Step.init(.{
            .id = .custom,
            .name = "tree-sitter-fixup",
            .makeFn = makeFn,
            .owner = b,
        }),
        .src_dir = src_dir,
    };

    const gen = b.addSystemCommand(&[_][]const u8{
        "tree-sitter",
        "generate",
        "--no-bindings",
        "./grammar.js",
    });
    gen.setCwd(b.path(b.pathJoin(&[_][]const u8{ "libs/doomparse/src", src_dir })));
    fixup.step.dependOn(&gen.step);

    b.step("tree-sitter", "Generate Tree-sitter for a DoomParse language")
        .dependOn(&fixup.step);
}

fn makeFn(step: *std.Build.Step, prog_node: ProgressNode) anyerror!void {
    const fixup: *Fixup = @fieldParentPtr("step", step);

    const alloc = step.owner.allocator;
    comptime var prog_steps = 7;
    prog_node.setEstimatedTotalItems(prog_steps);

    var work_dir = try std.fs.cwd().openDir(
        step.owner.pathJoin(&[_][]const u8{ "libs/doomparse/src", fixup.src_dir }),
        .{},
    );
    prog_node.completeOne();
    prog_steps -= 1;
    defer work_dir.close();

    const parser_c_in = try work_dir.openFile("src/parser.c", .{});
    prog_node.completeOne();
    prog_steps -= 1;
    defer parser_c_in.close();

    const parser_txt = try parser_c_in.readToEndAlloc(alloc, 1024 * 1024 * 64);
    prog_node.completeOne();
    prog_steps -= 1;
    defer alloc.free(parser_txt);

    const inc_path = try std.mem.concat(alloc, u8, &[_][]const u8{
        "\"",
        fixup.src_dir,
        "/parser.h\"",
    });

    const parser_txt_new = try std.mem.replaceOwned(
        u8,
        alloc,
        parser_txt,
        "<tree_sitter/parser.h>",
        inc_path,
    );
    prog_node.completeOne();
    prog_steps -= 1;

    var parser_c = try work_dir.createFile("parser.c", .{});
    prog_node.completeOne();
    prog_steps -= 1;
    defer parser_c.close();

    try parser_c.writeAll(parser_txt_new);
    prog_node.completeOne();
    prog_steps -= 1;

    try work_dir.rename("src/tree_sitter/parser.h", "parser.h");
    prog_node.completeOne();
    prog_steps -= 1;

    comptime std.debug.assert(prog_steps == 0);
}
