//! The [node builder] used by the VileTech Engine.
//!
//! Currently this is [ZNBX], but a long-term goal is to replace it with a pure-Zig port.
//!
//! [node builder]: https://doomwiki.org/wiki/Node_builder
//! [ZNBX]: https://github.com/jerome-trc/znbx

const std = @import("std");

const c = @cImport({
    @cInclude("znbx.h");
});
const root = @import("root.zig");
const Node = root.Node;

pub const NodeBuilder = opaque {
    pub const NodeVisitor = fn (anytype, *const Node) void;

    pub fn init(
        flavor: enum { vanilla, extended },
        level_name: [8:0]u8,
        things: []const u8,
        vertices: []const u8,
        linedefs: []const u8,
        sidedefs: []const u8,
        sectors: []const u8,
    ) Error!*NodeBuilder {
        const level = c.znbx_Level{
            .name = @bitCast(level_name),
            .things = c.znbx_SliceU8{ .ptr = things.ptr, .len = things.len },
            .vertices = c.znbx_SliceU8{ .ptr = vertices.ptr, .len = vertices.len },
            .linedefs = c.znbx_SliceU8{ .ptr = linedefs.ptr, .len = linedefs.len },
            .sidedefs = c.znbx_SliceU8{ .ptr = sidedefs.ptr, .len = sidedefs.len },
            .sectors = c.znbx_SliceU8{ .ptr = sectors.ptr, .len = sectors.len },
        };

        const ret = switch (flavor) {
            .vanilla => c.znbx_processor_new_vanilla(level) orelse return error.InitFail,
            .extended => c.znbx_processor_new_extended(level) orelse return error.InitFail,
        };

        return @as(*@This(), @ptrCast(ret));
    }

    pub fn initUdmf(
        level_name: [8:0]u8,
        textmap: []const u8,
    ) Error!*NodeBuilder {
        const level = c.znbx_Level{
            .name = level_name,
            .textmap = c.znbx_SliceU8{ .ptr = textmap.ptr, .len = textmap.len },
        };

        return @as(*@This(), @ptrCast(c.znbx_processor_new_udmf(level))) orelse error.InitFail;
    }

    pub fn deinit(self: *NodeBuilder) void {
        c.znbx_processor_destroy(@ptrCast(self));
    }

    pub fn run(self: *NodeBuilder) void {
        c.znbx_processor_run(@ptrCast(self), null);
    }

    pub fn runWith(self: *NodeBuilder, config: struct {
        aa_preference: i32 = c.AA_PREFERENCE_DEFAULT,
        max_segs: i32 = c.MAX_SEGS_DEFAULT,
        split_cost: i32 = c.SPLIT_COST_DEFAULT,
    }) void {
        const cfg = c.znbx_NodeConfig{
            .aa_preference = config.aa_preference,
            .max_segs = config.max_segs,
            .split_cost = config.split_cost,
        };

        c.znbx_processor_run(@ptrCast(self), &cfg);
    }

    pub fn magicNumber(self: *NodeBuilder, compress: bool) ?[:0]const u8 {
        return c.znbx_processor_magicnumber(@ptrCast(self), compress);
    }

    pub fn foreachNode(self: *NodeBuilder, context: anytype) void {
        const callback = struct {
            fn callback(v_o: ?*anyopaque, n: [*c]const c.znbx_NodeRaw) callconv(.C) void {
                var ctx: @TypeOf(context) = @alignCast(@ptrCast(v_o.?));
                const node: Node = @bitCast(n.*);
                ctx.foreach(&node);
            }
        };

        c.znbx_processor_nodes_foreach(@ptrCast(self), context, callback.callback);
    }
};

pub const Error = error{
    InitFail,
};

test {
    std.testing.refAllDecls(@import("nodebuild/test.zig"));
}
