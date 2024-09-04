//! See <https://doomwiki.org/wiki/Blockmap>.

const std = @import("std");

const Pos16 = @import("root.zig").Pos16;

const Self = @This();

pub const Error = error{
    /// BLOCKMAP "lump" is not even large enough to fit a header.
    UndersizeFile,
    /// Number of columns times number of rows equals zero.
    ZeroBlocks,
};

pub const Header = extern struct {
    _origin_x: i16,
    _origin_y: i16,
    _columns: u16,
    _rows: u16,
};

bytes: []align(2) const u8,
header: Header,
offsets: []const u16,

/// The lifetime of the returned instance is tied to the lifetime of `bytes`; any
/// calls to associated methods become undefined behaviour as soon as `bytes` is invalidated.
pub fn read(bytes: []align(2) const u8) Error!Self {
    if (bytes.len < @sizeOf(Header)) return error.UndersizeFile;

    const header = std.mem.bytesAsValue(Header, bytes[0..@sizeOf(Header)]);
    const block_count =
        std.mem.littleToNative(u16, header._columns) *
        std.mem.littleToNative(u16, header._rows);

    if (block_count == 0) return error.ZeroBlocks;

    const l = @sizeOf(Header) + ((block_count - 1) * @sizeOf(u16));
    const offset_bytes = bytes[@sizeOf(Header)..l];

    return Self{
        .bytes = bytes,
        .header = header.*,
        .offsets = std.mem.bytesAsSlice(u16, offset_bytes),
    };
}

pub fn origin(self: *const Self) Pos16 {
    return Pos16{
        .x = std.mem.littleToNative(i16, self.header._origin_x),
        .y = std.mem.littleToNative(i16, self.header._origin_y),
    };
}

pub fn columns(self: *const Self) u16 {
    return std.mem.littleToNative(u16, self.header._columns);
}

pub fn rows(self: *const Self) u16 {
    return std.mem.littleToNative(u16, self.header._rows);
}

pub fn blocklist(self: *const Self, which: u16) [:0xFFFF]const u16 {
    const offset = self.offsets[which] * 2;
    const b = std.mem.sliceTo(self.bytes[offset..], 0xFFFF);
    return std.mem.bytesAsSlice(u16, b);
}

test "smoke" {
    const f = try std.fs.cwd().openFile("sample/freedoom2/map01.wad", .{});
    defer f.close();

    const bytes = try f.readToEndAllocOptions(
        std.testing.allocator,
        1024 * 128,
        null,
        @alignOf(Header),
        null,
    );
    defer std.testing.allocator.free(bytes);

    var cursor: usize = 12;
    _ = bytes[cursor..(cursor + 1620)]; // THINGS
    cursor += 1620;
    _ = bytes[cursor..(cursor + 14966)]; // LINEDEFS
    cursor += 14966;
    _ = bytes[cursor..(cursor + 49980)]; // SIDEDEFS
    cursor += 49980;
    _ = bytes[cursor..(cursor + 4032)]; // VERTEXES
    cursor += 4032;
    _ = bytes[cursor..(cursor + 22056)]; // SEGS
    cursor += 22056;
    _ = bytes[cursor..(cursor + 2212)]; // SSECTORS
    cursor += 2212;
    _ = bytes[cursor..(cursor + 15456)]; // NODES
    cursor += 15456;
    _ = bytes[cursor..(cursor + 5148)]; // SECTORS
    cursor += 5148;
    _ = bytes[cursor..(cursor + 4901)]; // REJECT
    cursor += 4901;
    const blockmap_align1 = bytes[cursor..(cursor + 5482)];

    const blockmap_bytes = try std.testing.allocator.alignedAlloc(
        u8,
        @alignOf(Header),
        blockmap_align1.len,
    );
    defer std.testing.allocator.free(blockmap_bytes);
    @memcpy(blockmap_bytes, blockmap_align1);

    _ = try read(@alignCast(blockmap_bytes));
    // TODO: test more deeply when I find a proven tool for reading blockmaps.
}
