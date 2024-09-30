const std = @import("std");

const Point = tree_sitter.Point;
const tree_sitter = @import("../../../root.zig").tree_sitter;
const wadmerge = @import("../../../doomtools.zig").wadmerge;

test "empty input" {
    var parser = try wadmerge.Parser.init();
    defer parser.deinit();

    const tree = try parser.parseString("");
    defer tree.deinit();

    const root = tree.root();
    try std.testing.expectEqual(0, root.namedChildCount());

    const sexpr = root.sExpr();
    defer std.c.free(sexpr.ptr);
    try std.testing.expectEqualStrings("(source_file)", sexpr);
}

test "clear_command, ident symbol" {
    const sample =
        \\clear negelida
        \\clear clear
    ;

    var parser = try wadmerge.Parser.init();
    defer parser.deinit();

    var reader = struct {
        pub fn read(_: *@This(), byte_index: u32, _: Point) []const u8 {
            return sample[byte_index..];
        }
    }{};

    const tree = try parser.parse(.utf8, &reader);
    defer tree.deinit();

    const root = tree.root();
    const sexpr = root.sExpr();
    defer std.c.free(sexpr.ptr);

    std.testing.expectEqual(2, root.namedChildCount()) catch {
        std.debug.panic("{s}\n", .{sexpr});
    };
}

test "clear_command, string symbol" {
    const sample =
        \\clear "ephemeral"
        \\clear "abstract thoughts"
    ;

    var parser = try wadmerge.Parser.init();
    defer parser.deinit();

    const tree = try parser.parseString(sample);
    defer tree.deinit();

    const root = tree.root();
    const sexpr = root.sExpr();
    defer std.c.free(sexpr.ptr);

    std.testing.expectEqual(2, root.namedChildCount()) catch {
        std.debug.panic("{s}\n", .{sexpr});
    };
}

test "create_command, ident symbol" {
    const sample =
        \\create reprocessing iwad
        \\create stone steel
        \\create 𝜴+ IWAD 𝝋
        \\create iwad iwad iwad
    ;

    var parser = try wadmerge.Parser.init();
    defer parser.deinit();

    const tree = try parser.parseString(sample);
    defer tree.deinit();

    const root = tree.root();
    const sexpr = root.sExpr();
    defer std.c.free(sexpr.ptr);

    std.testing.expectEqual(4, root.namedChildCount()) catch {
        std.debug.panic("{s}\n", .{sexpr});
    };

    const create_0 = root.namedChild(0).?;
    const create_0_sym = create_0.field("symbol").?;
    try std.testing.expectEqualStrings("reprocessing", create_0_sym.slice(sample));
    const create_0_iwad = create_0.field("iwad_qual").?;
    try std.testing.expectEqualStrings("iwad", create_0_iwad.slice(sample));

    const create_1 = root.namedChild(1).?;
    const create_1_sym = create_1.field("symbol").?;
    try std.testing.expectEqualStrings("stone", create_1_sym.slice(sample));

    const create_2 = root.namedChild(2).?;
    const create_2_sym = create_2.field("symbol").?;
    try std.testing.expectEqualStrings("𝜴+", create_2_sym.slice(sample));
    const create_2_iwad = create_2.field("iwad_qual").?;
    try std.testing.expectEqualStrings("IWAD", create_2_iwad.slice(sample));

    const create_3 = root.namedChild(3).?;
    const create_3_sym = create_3.field("symbol").?;
    try std.testing.expectEqualStrings("iwad", create_3_sym.slice(sample));
    const create_3_iwad = create_3.field("iwad_qual").?;
    try std.testing.expectEqualStrings("iwad", create_3_iwad.slice(sample));
}

test "create_command, string symbol" {
    const sample =
        \\create ""
        \\create "immortal industry" iWad
        \\create " Subterraqueous " iwad Deep But Not Profound
    ;

    var parser = try wadmerge.Parser.init();
    defer parser.deinit();

    const tree = try parser.parseString(sample);
    defer tree.deinit();

    const root = tree.root();
    const sexpr = root.sExpr();
    defer std.c.free(sexpr.ptr);

    std.testing.expectEqual(3, root.namedChildCount()) catch {
        std.debug.panic("{s}\n", .{sexpr});
    };

    const create_0 = root.namedChild(0).?;
    const create_0_sym = create_0.field("symbol").?;
    try std.testing.expectEqualStrings("\"\"", create_0_sym.slice(sample));

    const create_1 = root.namedChild(1).?;
    const create_1_sym = create_1.field("symbol").?;
    try std.testing.expectEqualStrings("\"immortal industry\"", create_1_sym.slice(sample));
    const create_1_iwad = create_1.field("iwad_qual").?;
    try std.testing.expectEqualStrings("iWad", create_1_iwad.slice(sample));

    const create_2 = root.namedChild(2).?;
    const create_2_sym = create_2.field("symbol").?;
    try std.testing.expectEqualStrings("\" Subterraqueous \"", create_2_sym.slice(sample));
    const create_2_iwad = create_2.field("iwad_qual").?;
    try std.testing.expectEqualStrings("iwad", create_2_iwad.slice(sample));
}

test "echo_command" {
    const sample =
        \\
        \\ # severed dreams
        \\echo
        \\#corrosion
        \\echo milliphobia
        \\eCho ' vedauwoo '
        \\
    ;

    var parser = try wadmerge.Parser.init();
    defer parser.deinit();

    const tree = try parser.parseString(sample);
    defer tree.deinit();
}

test "end_command" {
    const sample =
        \\end
        \\ end
        \\end # into the dark
        \\end wastewater falls
    ;

    var parser = try wadmerge.Parser.init();
    defer parser.deinit();

    const tree = try parser.parseString(sample);
    defer tree.deinit();

    const root = tree.root();
    const sexpr = root.sExpr();
    defer std.c.free(sexpr.ptr);

    std.testing.expectEqual(4, root.namedChildCount()) catch {
        std.debug.panic("{s}\n", .{sexpr});
    };
}
