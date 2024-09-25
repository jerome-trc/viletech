const std = @import("std");

const root = @import("../root.zig");
const nb = root.nodebuild;
const Node = root.Node;

test "Node builder, vanilla, smoke" {
    const f = try std.fs.cwd().openFile("sample/freedoom2/map01.wad", .{});
    defer f.close();

    const bytes = try f.readToEndAlloc(std.testing.allocator, 1024 * 128);
    defer std.testing.allocator.free(bytes);

    var cursor: usize = 12;
    const things = bytes[cursor..(cursor + 1620)];
    cursor += 1620;
    const linedefs = bytes[cursor..(cursor + 14966)];
    cursor += 14966;
    const sidedefs = bytes[cursor..(cursor + 49980)]; // SIDEDEFS
    cursor += 49980;
    const vertices = bytes[cursor..(cursor + 4032)];
    cursor += 4032;
    _ = bytes[cursor..(cursor + 22056)]; // SEGS
    cursor += 22056;
    _ = bytes[cursor..(cursor + 2212)]; // SSECTORS
    cursor += 2212;
    _ = bytes[cursor..(cursor + 15456)]; // NODES
    cursor += 15456;
    const sectors = bytes[cursor..(cursor + 5148)];

    var nodebuilder = try nb.NodeBuilder.init(
        .vanilla,
        [8:0]u8{ 'M', 'A', 'P', '0', '1', 0, 0, 0 },
        things,
        vertices,
        linedefs,
        sidedefs,
        sectors,
    );
    defer nodebuilder.deinit();

    nodebuilder.run();

    var nbctx = struct {
        nodes: std.ArrayList(Node),

        pub fn foreach(self: *@This(), node: *const Node) void {
            self.nodes.append(node.*) catch unreachable;
        }
    }{
        .nodes = std.ArrayList(Node).init(std.testing.allocator),
    };
    defer nbctx.nodes.deinit();

    nodebuilder.foreachNode(&nbctx);

    // TODO: copy over the unit tests from Rust once the standard library's MD5
    // implementation stabilizes.
}
