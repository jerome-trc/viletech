const segs = @import("segs.zig");
const ssectors = @import("ssectors.zig");
const things = @import("things.zig");
const vertexes = @import("vertexes.zig");

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

test {
    @import("std").testing.refAllDecls(@This());
}
