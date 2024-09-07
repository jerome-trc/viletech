//! See <https://doomwiki.org/wiki/DMXGUS> and <https://zdoom.org/wiki/DMXGUS>.

const std = @import("std");

const Self = @This();

pub const Error = error{
    EmptyLine,
    MissingPatchName,
    MissingPatchNum,
    MissingPatchRemap,
} || std.fmt.ParseIntError;

pub const Iterator = struct {
    inner: std.mem.SplitIterator(u8, .scalar),

    pub fn next(self: *Iterator) Error!?Self {
        const pre_comment = while (self.inner.next()) |line| {
            var c = std.mem.splitScalar(u8, line, '#');
            const p = c.next() orelse return error.EmptyLine;

            if (p.len == 0) continue;
            break p;
        } else return null;

        var parts = std.mem.splitScalar(u8, pre_comment, ',');

        const str_patch_num = parts.next() orelse return error.MissingPatchNum;
        const str_patch_remap_0 = parts.next() orelse return error.MissingPatchRemap;
        const str_patch_remap_1 = parts.next() orelse return error.MissingPatchRemap;
        const str_patch_remap_2 = parts.next() orelse return error.MissingPatchRemap;
        const str_patch_remap_3 = parts.next() orelse return error.MissingPatchRemap;
        const patch_name = parts.next() orelse return error.MissingPatchName;

        return Self{
            .patch_num = try std.fmt.parseInt(
                u8,
                std.mem.trim(u8, str_patch_num, " \t\r"),
                10,
            ),
            .patch_remap = [4]u8{
                try std.fmt.parseInt(
                    u8,
                    std.mem.trim(u8, str_patch_remap_0, " \t\r"),
                    10,
                ),
                try std.fmt.parseInt(
                    u8,
                    std.mem.trim(u8, str_patch_remap_1, " \t\r"),
                    10,
                ),
                try std.fmt.parseInt(
                    u8,
                    std.mem.trim(u8, str_patch_remap_2, " \t\r"),
                    10,
                ),
                try std.fmt.parseInt(
                    u8,
                    std.mem.trim(u8, str_patch_remap_3, " \t\r"),
                    10,
                ),
            },
            .patch_name = std.mem.trim(u8, patch_name, " \t\r"),
        };
    }

    pub fn reset(self: *Iterator) void {
        self.inner.reset();
    }
};

patch_num: u8,
patch_remap: [4]u8,
patch_name: []const u8,

pub fn read(text: []const u8) Iterator {
    return Iterator{ .inner = std.mem.splitScalar(u8, text, '\n') };
}

test "DMXGUS, smoke" {
    const cfg = @import("cfg");

    if (cfg.dmxgus_sample.len == 0) return error.SkipZigTest;

    const f = try std.fs.cwd().openFile(cfg.dmxgus_sample, .{});
    defer f.close();

    const bytes = try f.readToEndAlloc(
        std.testing.allocator,
        1024 * 8,
    );
    defer std.testing.allocator.free(bytes);

    var iter = read(bytes);
    const line19 = (try iter.next()).?;

    try std.testing.expectEqual(0, line19.patch_num);
    try std.testing.expectEqual([4]u8{ 2, 1, 1, 0 }, line19.patch_remap);
    try std.testing.expectEqualStrings("acpiano", line19.patch_name);

    var line208: Self = undefined;

    while (try iter.next()) |d| line208 = d;

    try std.testing.expectEqual(215, line208.patch_num);
    try std.testing.expectEqual([4]u8{ 128, 128, 128, 128 }, line208.patch_remap);
    try std.testing.expectEqualStrings("surdo2", line208.patch_name);
}
