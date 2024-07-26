const std = @import("std");

const c = @import("../main.zig").c;

const Console = @import("Console.zig");
const Core = @import("../Core.zig");
const imgui = @import("../imgui.zig");

const Self = @This();

const Collection = struct {
    name: [:0]const u8,
    songs: std.ArrayList(Song),
};

const Song = struct {
    artist: [:0]const u8,
    title: [:0]const u8,
    lump: c.LumpNum,
};

alloc: std.heap.ArenaAllocator,
collections: std.ArrayList(Collection),

pub fn init(alloc: std.mem.Allocator) Self {
    var arena = std.heap.ArenaAllocator.init(alloc);

    return Self{
        .alloc = arena,
        .collections = std.ArrayList(Collection).init(arena.allocator()),
    };
}

pub fn deinit(self: *Self) void {
    _ = self.alloc.reset(.free_all);
}

pub fn populate(self: *Self) std.mem.Allocator.Error!void {
    var arena = std.heap.ArenaAllocator.init(self.alloc.child_allocator);
    defer _ = arena.deinit();

    var lmp_num: c.LumpNum = 0;

    while (lmp_num < c.numlumps) {
        defer lmp_num += 1;

        const lmp = &c.lumpinfo[std.math.lossyCast(usize, lmp_num)];
        const lmp_name = lmp.name[0..8];

        if (!std.mem.eql(u8, lmp_name, "DSKJOCKY")) {
            continue;
        }

        const buf = try arena.allocator().alloc(u8, std.math.lossyCast(usize, lmp.size));
        defer _ = arena.reset(.retain_capacity);

        c.W_ReadLump(lmp_num, buf.ptr);
        var iter = std.mem.splitAny(u8, buf, "\r\n");

        const coll_name = iter.next() orelse {
            c.I_Error("DSKJOCKY lump %i is missing a collection name", lmp_num);
        };

        var new_coll = Collection{
            .name = try self.alloc.allocator().dupeZ(u8, coll_name),
            .songs = std.ArrayList(Song).init(self.alloc.allocator()),
        };

        var ii: usize = 0;

        while (iter.next()) |line| {
            var parts = std.mem.splitSequence(u8, line, "__");
            lmp_num += 1;
            defer ii += 1;

            const title = parts.next() orelse {
                c.I_Error("DSKJOCKY lump %i is missing a song title at line %lu", lmp_num, ii);
            };
            const artist = parts.next() orelse {
                c.I_Error("DSKJOCKY lump %i is missing an artist at line %lu", lmp_num, ii);
            };

            try new_coll.songs.append(Song{
                .artist = try self.alloc.allocator().dupeZ(u8, std.mem.trim(
                    u8,
                    artist,
                    " \r\n\t",
                )),
                .title = try self.alloc.allocator().dupeZ(u8, std.mem.trim(
                    u8,
                    title,
                    " \r\n\t",
                )),
                .lump = lmp_num,
            });
        }

        try self.collections.append(new_coll);
    }
}

pub fn layout(cx: *Core, left: bool, menu_bar_height: f32) void {
    const self = &cx.musicgui;

    const vp_size = if (c.igGetMainViewport()) |vp| vp.*.Size else {
        imgui.reportErrGetMainViewport.call();
        return;
    };

    if (left) {
        c.igSetNextWindowPos(.{ .x = 0.0, .y = menu_bar_height }, c.ImGuiCond_None, .{});
    } else {
        c.igSetNextWindowPos(
            .{ .x = vp_size.x * 0.5, .y = menu_bar_height },
            c.ImGuiCond_None,
            .{},
        );
    }

    c.igSetNextWindowSize(
        .{ .x = vp_size.x * 0.5, .y = vp_size.y * 0.33 },
        c.ImGuiCond_None,
    );

    if (!c.igBegin(
        "Music",
        null,
        c.ImGuiWindowFlags_NoTitleBar | c.ImGuiWindowFlags_NoResize,
    )) return;

    defer c.igEnd();

    for (self.collections.items, 0..) |coll, i| {
        if (!c.igTreeNode_Str(coll.name)) continue;
        defer c.igTreePop();

        var buf: [64]u8 = undefined;
        var fba = std.heap.FixedBufferAllocator.init(&buf);
        const str_id = std.fmt.allocPrintZ(fba.allocator(), "dj.coll.{}", .{i}) catch break;

        if (!c.igBeginTable(
            str_id.ptr,
            2,
            c.ImGuiTableFlags_RowBg | c.ImGuiTableFlags_Borders | c.ImGuiTableColumnFlags_WidthFixed,
            .{ .x = -1.0, .y = 0.0 },
            0.0,
        )) {
            continue;
        }

        defer c.igEndTable();

        c.igTableSetupColumn(
            "##title",
            c.ImGuiTableColumnFlags_WidthFixed,
            imgui.contentRegionAvail().x * 0.5,
            0,
        );
        c.igTableSetupColumn(
            "##artist",
            c.ImGuiTableColumnFlags_WidthFixed,
            0.0,
            0,
        );

        for (coll.songs.items) |song| {
            c.igTableNextRow(c.ImGuiTableRowFlags_None, 0.0);
            _ = c.igTableSetColumnIndex(0);

            if (c.igSmallButton(song.title)) {
                c.S_ChangeMusInfoMusic(@ptrCast(&cx.c), song.lump, @intFromBool(true));
                Console.logInfo(cx, "Now playing: '{s}' by {s}", .{ song.title, song.artist });
            }

            _ = c.igTableSetColumnIndex(1);
            imgui.textUnformatted(song.artist);
        }
    }
}

fn numSongs(self: *const Self) usize {
    var ret = 0;

    for (self.collections.items) |coll| {
        ret += coll.songs.items.len;
    }

    return ret;
}

export fn populateMusicPlayer(ccx: *Core.C) void {
    ccx.core.musicgui.populate() catch {
        c.I_Error("Music player population failed: out of memory");
    };
}
