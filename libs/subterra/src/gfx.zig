//! Graphics-related representations.

const std = @import("std");

/// See <https://doomwiki.org/wiki/COLORMAP>.
/// This struct is meant to be cast to via `std.mem.bytesAsValue`.
pub const ColorMap = extern struct {
    pub const Table = [256]u8;

    /// 0 is brightest, 31 is darkest.
    brightness: [32]Table,
    invuln: Table,
    allblack: Table,
};

/// See <https://doomwiki.org/wiki/ENDOOM>.
/// This struct is meant to be cast to via `std.mem.bytesAsValue`.
pub const EnDoom = extern struct {
    pub const Foreground = enum(u4) {
        black,
        blue,
        green,
        cyan,
        red,
        magenta,
        brown,
        light_gray,
        dark_gray,
        light_blue,
        light_green,
        light_cyan,
        light_red,
        light_magenta,
        yellow,
        white,
    };

    pub const Background = enum(u3) {
        black,
        blue,
        green,
        cyan,
        red,
        magenta,
        brown,
        light_gray,
    };

    pub const Character = struct {
        ascii: u8,
        color: packed struct(u8) {
            fg: Foreground,
            bg: Background,
            blink: bool,
        },
    };

    chars: [80 * 25]Character,
};

/// See <https://doomwiki.org/wiki/PLAYPAL> and [`PaletteSet`].
pub const Palette = [256]Rgb8;

/// See <https://doomwiki.org/wiki/PLAYPAL>.
/// This struct is meant to be cast to via `std.mem.bytesAsValue`.
pub const PaletteSet = struct {
    normal: Palette,
    /// Went unused by the original Doom engine.
    red11: Palette,
    reds: [7]Palette,
    /// Went unused by the original Doom engine.
    yellow12_5: Palette,
    yellows: [3]Palette,
    green: Palette,
};

pub const PictureReader = struct {
    pub const ReadContext = fn (usize, usize, Rgb8) void;

    fbs_h: std.io.FixedBufferStream([]const u8),

    /// The position just past the header.
    checkpoint: u64,
    width: u16,
    height: u16,
    /// Offset in pixels to the left of the origin.
    left: i16,
    /// Offset below the origin.
    top: i16,

    const header_size: usize = @sizeOf(u16) * 4;

    pub fn init(bytes: []const u8) !PictureReader {
        var fbs_h = std.io.fixedBufferStream(bytes);

        const width = try fbs_h.reader().readInt(u16, .little);
        const height = try fbs_h.reader().readInt(u16, .little);
        const left = try fbs_h.reader().readInt(i16, .little);
        const top = try fbs_h.reader().readInt(i16, .little);

        // (SLADE) Sanity checks on dimensions and offsets.

        if (width >= 4096 or height >= 4096) {
            // TODO: add details to these errors when doing so is supported.
            return error.InvalidHeader;
        }

        if (left <= -2000 or left >= 2000) {
            return error.InvalidHeader;
        }

        if (top <= -2000 or top >= 2000) {
            return error.InvalidHeader;
        }

        if (bytes.len < (header_size + (width * 4))) {
            return error.InvalidHeader;
        }

        const checkpoint = fbs_h.pos;

        for (0..width) |_| {
            const col_offs = try fbs_h.reader().readInt(u32, .little);

            if (col_offs > bytes.len or col_offs < header_size) {
                return error.InvalidHeader;
            }

            // (SLADE) Check if total size is reasonable; this computation corresponds
            // to the most inefficient possible use of space by the format
            // (horizontal stripes of 1 pixel, 1 pixel apart).

            const num_pixels: usize = ((height + 2 + @rem(height, 2)) / 2);
            const max_col_size = @sizeOf(u32) + (num_pixels * 5) + 1;

            if (bytes.len > header_size + (width * max_col_size)) {
                // TODO: consider giving this a try, since it's unlikely but possible.
                return error.InvalidHeader;
            }
        }

        return PictureReader{
            .fbs_h = fbs_h,
            .checkpoint = checkpoint,
            .width = width,
            .height = height,
            .left = left,
            .top = top,
        };
    }

    /// `ctx` should be a type with a method `callback`, matching the signature of `ReadContext`.
    pub fn readAll(
        self: *PictureReader,
        palette: *const Palette,
        colormap: *const ColorMap.Table,
        ctx: anytype,
    ) void {
        const fbs_pix = std.io.fixedBufferStream(self.fbs_h.buffer);
        self.fbs_h.seekTo(self.fbs_h.getPos() catch unreachable) catch unreachable;

        for (0..self.width) |i| {
            const col_offs = self.fbs_h.reader().readInt(u32, .little) catch unreachable;
            fbs_pix.seekTo(col_offs) catch unreachable;
            var row_start: u8 = 0;

            while (row_start != 255) {
                row_start = fbs_pix.reader().readByte() catch unreachable;

                if (row_start == 255) break;

                const pixel_count = fbs_pix.reader().readByte() catch unreachable;
                _ = fbs_pix.reader().readByte() catch unreachable; // Dummy

                for (0..pixel_count) |ii| {
                    const map_entry = fbs_pix.reader().readByte() catch unreachable;
                    const pal_entry = colormap[map_entry];
                    const pixel = palette[pal_entry];
                    const col = ii + row_start;
                    ctx.callback(i, col, pixel);
                }

                _ = fbs_pix.reader().readByte() catch unreachable; // Dummy
            }
        }
    }
};

/// A single pixel.
pub const Rgb8 = struct {
    r: u8,
    g: u8,
    b: u8,
};

pub const PatchedTex = struct {
    /// Prefer calling `name` to reading this since it properly handles
    /// the intermediate null terminator which may or may not be present.
    name_raw: [8:0]u8,
    size_x: u32,
    size_y: u32,

    pub fn name(self: *const PatchedTex) []const u8 {
        return std.mem.sliceTo(self.name_raw[0..], 0);
    }
};

/// See <https://doomwiki.org/wiki/TEXTURE1_and_TEXTURE2>.
pub const TextureX = struct {
    fbs: std.io.FixedBufferStream([]align(@alignOf(u32)) const u8),
    num_textures: u32,
    offsets: []const u32,
    /// Index into `offsets`.
    pos: usize,

    pub fn init(bytes: []align(@alignOf(u32)) const u8) !TextureX {
        if (bytes.len < @sizeOf(u32)) {
            return error.InvalidHeader;
        }

        var fbs = std.io.fixedBufferStream(bytes);

        const num_textures = try fbs.reader().readInt(u32, .little);

        const offsets_start = @sizeOf(u32);
        const offsets_end = offsets_start + (@sizeOf(u32) * num_textures);

        if (bytes.len < offsets_end) {
            // Not enough bytes left for the expected offsets array.
            return error.InvalidHeader;
        }

        const offsets = std.mem.bytesAsSlice(u32, bytes[offsets_start..offsets_end]);
        const expected_len = offsets[offsets.len - 1] + @sizeOf(MapTexture);

        if (bytes.len < expected_len) {
            return error.InvalidHeader;
        }

        return TextureX{
            .fbs = fbs,
            .num_textures = num_textures,
            .offsets = offsets,
            .pos = 0,
        };
    }

    pub fn reset(self: *TextureX) void {
        self.pos = 0;
        self.fbs.seekTo(@sizeOf(u32) + (@sizeOf(u32) * self.num_textures)) catch unreachable;
    }

    pub fn next(self: *TextureX) !?MapPatches {
        if (self.pos >= self.offsets.len) {
            return null;
        }

        self.fbs.seekTo(self.offsets[self.pos]) catch unreachable;

        const texture = self.fbs.reader().readStructEndian(MapTexture, .little) catch unreachable;
        const expected_bytes = self.fbs.pos + (@sizeOf(TexPatch) * texture.patch_count);

        if (self.fbs.buffer.len < expected_bytes) {
            return error.EndOfStream;
        }

        const patch_bytes: []align(@alignOf(TexPatch)) const u8 = @alignCast(
            self.fbs.buffer[self.fbs.pos..expected_bytes],
        );

        const patches = std.mem.bytesAsSlice(TexPatch, patch_bytes);

        self.pos += 1;
        return MapPatches{ .texture = texture, .patches = patches };
    }
};

pub const MapPatches = struct {
    texture: MapTexture,
    patches: []align(@alignOf(TexPatch)) const TexPatch,
};

/// See <https://doomwiki.org/wiki/TEXTURE1_and_TEXTURE2> and [`TextureX`].
pub const MapTexture = extern struct {
    /// Prefer calling `name` to reading this since it properly handles
    /// the intermediate null terminator which may or may not be present.
    name_raw: [8]u8 align(1),
    /// Acts like a boolean (0 = false, 1 = true).
    masked: u32 align(2),
    width: u16,
    height: u16,
    /// Unused by the original Doom engine.
    column_directory: u32 align(2),
    patch_count: u16,

    pub fn name(self: *const MapTexture) []const u8 {
        return std.mem.sliceTo(self.name_raw[0..], 0);
    }
};

/// See [`PatchedTex`].
/// This struct is meant to be cast to via `std.mem.bytesAsValue`.
pub const TexPatch = extern struct {
    /// X-offset of this patch relative to the upper-left of the whole texture.
    origin_x: i16,
    /// Y-offset of this patch relative to the upper-left of the whole texture.
    origin_y: i16,
    /// Index into [`PatchTable`].
    index: u16,
    /// Unused by the original Doom engine.
    step_dir: i16,
    /// Unused by the original Doom engine.
    colormap: i16,
};

// TODO: would be nice if sample data could be embedded in the test binary
// without having to put the files in this directory.

test "PictureReader smoke" {
    const f = try std.fs.cwd().openFile("sample/freedoom/STFST01.lmp", .{});
    defer f.close();

    const bytes = try f.readToEndAlloc(std.testing.allocator, 1024);
    defer std.testing.allocator.free(bytes);

    const pic = try PictureReader.init(bytes);
    try std.testing.expectEqual(pic.width, 24);
    try std.testing.expectEqual(pic.height, 29);
}

test "TEXTUREx smoke" {
    const f = try std.fs.cwd().openFile("sample/freedoom/TEXTURE2.lmp", .{});
    defer f.close();

    const bytes = try f.readToEndAllocOptions(
        std.testing.allocator,
        1024 * 8,
        null,
        @alignOf(u32),
        null,
    );
    defer std.testing.allocator.free(bytes);

    var texx = try TextureX.init(bytes);

    _ = try texx.next() orelse unreachable;
    _ = try texx.next() orelse unreachable;

    const bigdoor5 = try texx.next() orelse unreachable;

    try std.testing.expectEqualStrings("BIGDOOR5", bigdoor5.texture.name());
    try std.testing.expectEqual(bigdoor5.texture.patch_count, 4);
    try std.testing.expectEqual(128, bigdoor5.texture.height);
    try std.testing.expectEqual(128, bigdoor5.texture.width);
    try std.testing.expectEqual(4, bigdoor5.patches.len);

    try std.testing.expectEqual(
        [2]i16{ 0, 0 },
        [2]i16{ bigdoor5.patches[0].origin_x, bigdoor5.patches[0].origin_y },
    );
    try std.testing.expectEqual(
        [2]i16{ 104, 0 },
        [2]i16{ bigdoor5.patches[3].origin_x, bigdoor5.patches[3].origin_y },
    );

    while (try texx.next()) |_| {}
    texx.reset();
}
