const std = @import("std");

const wadmerge = @import("../../root.zig").tree_sitter.Language("wadmerge");

test "smoke" {
    const sample =
        \\
        \\# severed dreams
        \\echo reprocessing
        \\
        \\create stone steel
        \\create "immortal industry"
        \\
        \\#milliphobia
        \\eCho ' vedauwoo '
        \\# corrosion
        \\ ECHO "wastewater falls"
        \\# into the dark
        \\end
        \\
        \\
    ;

    var parser = try wadmerge.Parser.init();
    defer parser.deinit();

    const tree = try parser.parseString(sample);
    defer tree.deinit();

    const root = tree.root();

    try std.testing.expectEqual(10, root.namedChildCount());

    const echo_0 = root.namedChild(1).?;
    const echo_0_txt = echo_0.namedChild(0).?;
    try std.testing.expectEqualStrings("reprocessing", echo_0_txt.slice(sample));

    const echo_1 = root.namedChild(5).?;
    const echo_1_txt = echo_1.namedChild(0).?;
    try std.testing.expectEqualStrings("' vedauwoo '", echo_1_txt.slice(sample));

    const create_0 = root.namedChild(2).?;
    const create_0_name = create_0.namedChild(0).?;
    try std.testing.expectEqualStrings("stone", create_0_name.slice(sample));

    const create_1 = root.namedChild(3).?;
    const create_1_name = create_1.namedChild(0).?;
    try std.testing.expectEqualStrings("\"immortal industry\"", create_1_name.slice(sample));
}
