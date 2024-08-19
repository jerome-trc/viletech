//! I/O abstractions for Doom's [WAD] archive file format.
//!
//! [WAD]: https://doomwiki.org/wiki/WAD

const std = @import("std");

/// Whether this WAD is the basis of a game, or a "mod".
pub const Kind = enum {
    /// "Internal WAD". See <https://doomwiki.org/wiki/IWAD>.
    iwad,
    /// "Patch WAD". See <https://doomwiki.org/wiki/PWAD>.
    pwad,
};

/// Most likely you'll pass one of the following as a `Reader`:
/// - `std.fs.File`
/// - `std.io.FixedBufferStream`
pub fn DirIterator(Reader: type) type {
    return struct {
        const Self = @This();

        pub const Entry = struct {
            /// Prefer calling `name` to reading this since it properly handles
            /// the intermediate null terminator which may or may not be present.
            name_raw: [8:0]u8,
            /// The byte position relative to byte 0 of the file that the entry's
            /// data begins at.
            start: usize,
            /// The byte position relative to byte 0 of the file that the entry's
            /// data ends at.
            end: usize,

            pub fn name(self: *const Entry) []const u8 {
                return std.mem.sliceTo(self.name_raw[0..], 0);
            }

            pub fn len(self: Entry) usize {
                return self.end - self.start;
            }
        };

        reader: Reader,
        /// Never mutate this, but feel free to read it.
        kind: Kind,
        /// The number of entries as reported by the directory.
        /// Never mutate this, but feel free to read it.
        len: usize,
        /// Never mutate this, but feel free to read it.
        current: usize,
        /// Never mutate this, but feel free to read it.
        stream_pos: u64,

        pub fn init(reader: Reader) !Self {
            const header = try Header.read(reader);

            return Self{
                .reader = reader,
                .kind = header.kind,
                .len = @intCast(header.lump_count),
                .current = 0,
                .stream_pos = header.dir_offs,
            };
        }

        pub fn next(self: *Self) anyerror!?Entry {
            if (self.current >= self.len) {
                return null;
            }

            const ebuf = try self.reader.reader().readBoundedBytes(16);
            if (ebuf.len != 16) return error.DirectoryTrunc;

            const offs_i32 = std.mem.readInt(i32, ebuf.buffer[0..4], .little);
            const size_i32 = std.mem.readInt(i32, ebuf.buffer[4..8], .little);

            if (offs_i32 < 0 or size_i32 < 0) {
                return error.InvalidDirEntry;
            }

            const offs: usize = @intCast(offs_i32);
            const size: usize = @intCast(size_i32);

            self.current += 1;
            self.stream_pos += dir_entry_size;

            return Entry{
                .name_raw = [8:0]u8{
                    ebuf.buffer[8],  ebuf.buffer[9],  ebuf.buffer[10], ebuf.buffer[11],
                    ebuf.buffer[12], ebuf.buffer[13], ebuf.buffer[14], ebuf.buffer[15],
                },
                .start = offs,
                .end = offs + size,
            };
        }
    };
}

/// A thin wrapper over a [`DirIterator`] that maps its output, returning both
/// directory entries and the byte content of their associated "lump".
///
/// Requires that `Reader` support seeking somehow, so that it can jump from
/// the directory to the lump and back again.
pub fn LumpIterator(Reader: type) type {
    return struct {
        const Self = @This();

        pub const Entry = struct {
            /// Prefer calling `name` to reading this since it properly handles
            /// the intermediate null terminator which may or may not be present.
            name_raw: [8:0]u8,
            /// The byte position relative to byte 0 of the file that the entry's
            /// data begins at.
            start: usize,
            /// The byte position relative to byte 0 of the file that the entry's
            /// data ends at.
            end: usize,
            data: []const u8,

            pub fn name(self: *const Entry) []const u8 {
                return std.mem.sliceTo(self.name_raw[0..], 0);
            }

            pub fn len(self: Entry) usize {
                return self.data.len;
            }
        };

        pub const Parent = DirIterator(Reader);

        inner: DirIterator(Reader),

        /// Note that `reader`'s cursor can be at any position.
        pub fn init(reader: Reader) !Self {
            return Self{ .inner = try DirIterator(Reader).init(reader) };
        }

        pub fn next(self: *Self, alloc: std.mem.Allocator) anyerror!?Entry {
            const entry = try self.inner.next() orelse return null;
            try self.inner.reader.seekTo(entry.start);
            const buf = try alloc.alloc(u8, entry.len());
            errdefer alloc.free(buf);
            const read_count = try self.inner.reader.reader().read(buf);
            if (read_count != entry.len()) return error.LumpTrunc;
            try self.inner.reader.seekTo(self.inner.stream_pos);

            return Entry{
                .name_raw = entry.name_raw,
                .start = entry.start,
                .end = entry.end,
                .data = buf,
            };
        }

        pub fn nextNoAlloc(self: *Self) anyerror!?Parent.Entry {
            return self.inner.next();
        }
    };
}

const dir_entry_size: usize = 16;

const Header = struct {
    kind: Kind,
    dir_offs: u64,
    lump_count: i32,

    /// Expects `reader` to support:
    /// - `getPos`
    /// - `reader().readBoundedBytes`
    /// - `seekFromEnd`
    fn read(reader: anytype) !Header {
        var r = reader.reader();
        const hbuf = try r.readBoundedBytes(12);
        if (hbuf.len != 12) return error.HeaderTrunc;

        const kind = if (std.mem.eql(u8, hbuf.buffer[0..4], "PWAD"))
            Kind.pwad
        else if (std.mem.eql(u8, hbuf.buffer[0..4], "IWAD"))
            Kind.iwad
        else
            return error.InvalidMagic;

        const lump_count = std.mem.readInt(i32, hbuf.buffer[4..8], .little);

        if (lump_count < 0) {
            return error.InvalidEntryCount;
        }

        const dir_offs = std.mem.readInt(i32, hbuf.buffer[8..12], .little);

        if (dir_offs < 0) {
            return error.InvalidDirOffset;
        }

        const expected_dir_len = std.math.mul(
            usize,
            @as(usize, @intCast(lump_count)),
            dir_entry_size,
        ) catch {
            return error.Oversize;
        };
        const expected_data_len = std.math.add(
            usize,
            @as(usize, @intCast(dir_offs)),
            expected_dir_len,
        ) catch {
            return error.Oversize;
        };

        if (std.meta.hasMethod(@TypeOf(reader), "seekFromEnd")) {
            try reader.seekFromEnd(0);
        } else {
            try reader.seekTo(try reader.getEndPos());
        }

        const pos = try reader.getPos();

        if (pos != (expected_data_len)) {
            return error.DataMalformed;
        }

        try reader.seekTo(@intCast(dir_offs));

        return Header{
            .dir_offs = @intCast(dir_offs),
            .lump_count = lump_count,
            .kind = kind,
        };
    }
};

pub const IterError = error{
    // TODO: payloads, like in the Rust version.

    /// The header prescribed `n` number of lumps and the directory is at a byte
    /// offset of `o`, but `(16 * n) + o` is past the length of readable data.
    DataMalformed,

    DirectoryTrunc,
    HeaderTrunc,
    /// Can be raised when trying to read a header.
    InvalidEntryCount,
    /// Can be raised when trying to read a header.
    InvalidDirOffset,
    /// Can be raised when trying to read a header.
    InvalidMagic,
    LumpTrunc,
    /// Can be raised when trying to read a header.
    Oversize,
};

test "Readers, compilation" {
    const no_bytes = [_]u8{};
    const empty_slice = no_bytes[0..];
    var fbs = std.io.fixedBufferStream(empty_slice);
    _ = LumpIterator(*std.io.FixedBufferStream([]const u8)).init(&fbs) catch {};
}

test "Readers, smoke" {
    const file = std.fs.cwd().openFile("sample/freedoom2/map01.wad", .{}) catch unreachable;
    defer file.close();

    var iter = LumpIterator(std.fs.File).init(file) catch unreachable;

    const map01 = (try iter.next(std.testing.allocator)).?;
    defer std.testing.allocator.free(map01.data);

    try std.testing.expectEqual(0, map01.data.len);
    try std.testing.expectEqualStrings("MAP01", map01.name());

    _ = (try iter.nextNoAlloc()).?; // THINGS
    _ = (try iter.nextNoAlloc()).?; // LINEDEFS
    _ = (try iter.nextNoAlloc()).?; // SIDEDEFS
    _ = (try iter.nextNoAlloc()).?; // VERTEXES
    _ = (try iter.nextNoAlloc()).?; // SEGS
    _ = (try iter.nextNoAlloc()).?; // SSECTORS
    _ = (try iter.nextNoAlloc()).?; // NODES
    _ = (try iter.nextNoAlloc()).?; // SECTORS
    _ = (try iter.nextNoAlloc()).?; // REJECT

    const blockmap = (try iter.next(std.testing.allocator)).?;
    defer std.testing.allocator.free(blockmap.data);

    try std.testing.expectEqualStrings("BLOCKMAP", blockmap.name());
    try std.testing.expectEqual(5482, blockmap.data.len);
    try std.testing.expectEqualSlices(
        u8,
        &[_]u8{ 0xB8, 0xFE, 0xFC, 0xF8, 0x14, 0x00, 0x1C, 0x00 },
        blockmap.data[0..8],
    );
    try std.testing.expectEqualSlices(
        u8,
        &[_]u8{ 0x3A, 0x03, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF },
        blockmap.data[(5482 - 8)..5482],
    );

    try std.testing.expectEqual(null, try iter.nextNoAlloc());
}
