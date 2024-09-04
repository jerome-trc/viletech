const std = @import("std");

const segs = @import("segs.zig");
const sidedefs = @import("sidedefs.zig");
const ssectors = @import("ssectors.zig");
const things = @import("things.zig");
const vertexes = @import("vertexes.zig");

pub const Blockmap = @import("Blockmap.zig");
pub const ednums = things.ednums;
pub const gfx = @import("gfx.zig");
pub const LineDef = @import("linedefs.zig").LineDef;
pub const mus = @import("mus.zig");
pub const Node = @import("nodes.zig").Node;
pub const nodebuild = if (@import("cfg").znbx) @import("nodebuild.zig") else void;
pub const Sector = @import("sectors.zig").Sector;
pub const Seg = segs.Seg;
pub const SegGl1 = segs.SegGl(.v1);
pub const SegGl3 = segs.SegGl(.v3);
pub const SegGl5 = segs.SegGl(.v5);
pub const SideDef = sidedefs.SideDef(.doom);
pub const SideDef64 = sidedefs.SideDef(.doom64);
pub const Subsector = ssectors.Subsector(u16);
pub const SubsectorGl = ssectors.Subsector(u32);
pub const Thing = things.Thing;
pub const Vertex = vertexes.Vertex;

pub const Game = enum {
    boom, // Not a game per se...
    doom,
    doom64,
    doom64ex,
    doom64_2020,
    heretic,
    hexen,
    mbf, // Also not a game...
    strife,
};

/// See <https://zdoom.org/wiki/Editor_number>.
pub const EditorNum = u16;

/// A two-dimensional position with signed 16-bit precision, used for deserializing levels.
pub const Pos16 = struct { x: i16, y: i16 };

/// See <https://glbsp.sourceforge.net/specs.php#Marker>.
pub const GlLevelMarker = struct {
    pub const Error = error{
        // TODO: payloads with details, if/when they are supported.
        BlankLine,
        /// e.g. line `LEVEL=`.
        EmptyValue,
        /// An `X=` line did not follow any other valid keyword line.
        InvalidExtension,
        /// A line was over 250 characters long.
        OverlongLine,
    } || std.mem.Allocator.Error;

    builder: ?[]const u8,
    checksum: ?[]const u8,
    level: ?[]const u8,
    time: ?[]const u8,

    pub fn read(alloc: std.mem.Allocator, bytes: []const u8) GlLevelMarker.Error!GlLevelMarker {
        const delimiter = if (std.mem.containsAtLeast(u8, bytes, 1, "\r\n"))
            "\r\n"
        else
            "\n";

        var lines = std.mem.splitSequence(u8, bytes, delimiter);

        var builder: ?[]const u8 = null;
        var checksum: ?[]const u8 = null;
        var level: ?[]const u8 = null;
        var time: ?[]const u8 = null;
        var last_field: ?*?[]const u8 = null;

        while (lines.next()) |line| {
            if (line.len > 250) {
                return error.OverlongLine;
            }

            var parts = std.mem.splitScalar(u8, line, '=');
            const key = parts.next() orelse return error.BlankLine;
            const val = parts.next() orelse return error.EmptyValue;

            if (std.mem.eql(u8, key, "BUILDER")) {
                last_field = &builder;
                builder = try alloc.dupe(u8, val);
            } else if (std.mem.eql(u8, key, "CHECKSUM")) {
                last_field = &checksum;
                checksum = try alloc.dupe(u8, val);
            } else if (std.mem.eql(u8, key, "LEVEL")) {
                last_field = &level;
                level = try alloc.dupe(u8, val);
            } else if (std.mem.eql(u8, key, "TIME")) {
                last_field = &time;
                time = try alloc.dupe(u8, val);
            } else if (std.mem.eql(u8, key, "X")) {
                const opt = last_field orelse return error.InvalidExtension;
                const prev = opt.*.?;
                opt.* = try std.mem.concat(alloc, u8, &[2][]const u8{ opt.*.?, val });
                alloc.free(prev);
            } else continue;
        }

        return GlLevelMarker{
            .level = level,
            .builder = builder,
            .time = time,
            .checksum = checksum,
        };
    }

    /// `alloc` must be the same allocator passed when calling `read`.
    pub fn deinit(self: *const GlLevelMarker, alloc: std.mem.Allocator) void {
        if (self.builder) |builder| alloc.free(builder);
        if (self.checksum) |checksum| alloc.free(checksum);
        if (self.level) |level| alloc.free(level);
        if (self.time) |time| alloc.free(time);
    }

    test "smoke" {
        const text =
            \\LEVEL=E1
            \\X=M1
            \\BUILDER=glBSP 2.14
            \\TIME=2005-03-26 13:50:03
            \\X=.2500
            \\CHECKSUM=0xABCDEF01
        ;

        const marker = try GlLevelMarker.read(std.testing.allocator, text);
        defer marker.deinit(std.testing.allocator);
        try std.testing.expectEqualStrings("E1M1", marker.level.?);
        try std.testing.expectEqualStrings("glBSP 2.14", marker.builder.?);
        try std.testing.expectEqualStrings("2005-03-26 13:50:03.2500", marker.time.?);
        try std.testing.expectEqualStrings("0xABCDEF01", marker.checksum.?);
    }
};

test {
    @import("std").testing.refAllDecls(@This());
}
