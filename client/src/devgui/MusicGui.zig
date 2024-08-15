const std = @import("std");
const log = std.log.scoped(.dj);

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

arena: std.heap.ArenaAllocator,
collections: std.ArrayListUnmanaged(Collection),
filter_buf: [256]u8,
filter_case_sensitive: bool,
prng: std.Random.Pcg,

pub fn init(alloc: std.mem.Allocator) Self {
    const rng_seed = if (std.time.Instant.now()) |now|
        if (@TypeOf(now.timestamp) == std.posix.timespec)
            now.timestamp.tv_nsec
        else
            now.timestamp
    else |_|
        0;

    return Self{
        .arena = std.heap.ArenaAllocator.init(alloc),
        .collections = std.ArrayListUnmanaged(Collection){},
        .prng = std.Random.Pcg.init(@intCast(rng_seed)),
        .filter_buf = [_]u8{0} ** 256,
        .filter_case_sensitive = false,
    };
}

pub fn deinit(self: *Self) void {
    self.arena.deinit();
}

pub fn populate(self: *Self) std.mem.Allocator.Error!void {
    const start_time = std.time.Instant.now();

    var lmp_num: c.LumpNum = 0;
    var map_arena = std.heap.ArenaAllocator.init(self.arena.child_allocator);
    defer map_arena.deinit();
    var map = std.StringHashMap(c.LumpNum).init(map_arena.allocator());
    defer map.deinit();

    while (lmp_num < c.numlumps) {
        defer lmp_num += 1;

        const lmp = &c.lumpinfo[std.math.lossyCast(usize, lmp_num)];
        const lmp_name = lmp.name[0..8];

        if (lmp_name[0] != 'D' or
            lmp_name[1] != 'J' or
            !std.ascii.isDigit(lmp_name[2]) or
            !std.ascii.isDigit(lmp_name[3]) or
            !std.ascii.isDigit(lmp_name[4]) or
            !std.ascii.isDigit(lmp_name[5]))
        {
            continue;
        }

        try map.put(try map_arena.allocator().dupe(u8, lmp_name[0..6]), lmp_num);
    }

    lmp_num = 0;
    var arena = std.heap.ArenaAllocator.init(self.arena.child_allocator);
    defer _ = arena.deinit();

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
            .name = try self.arena.allocator().dupeZ(u8, coll_name),
            .songs = std.ArrayList(Song).init(self.arena.allocator()),
        };

        var ii: usize = 0;

        while (iter.next()) |line| {
            if (line.len == 0) break;

            var parts = std.mem.splitSequence(u8, line, "__");
            defer ii += 1;

            const title = parts.next() orelse {
                c.I_Error(
                    "DSKJOCKY lump \"%.*s\" is missing a song title at line %lu",
                    new_coll.name.len,
                    new_coll.name.ptr,
                    ii,
                );
            };
            const artist = parts.next() orelse {
                c.I_Error(
                    "DSKJOCKY lump \"%.*s\" is missing an artist at line %lu",
                    new_coll.name.len,
                    new_coll.name.ptr,
                    ii,
                );
            };
            const mus_lmp = parts.next() orelse {
                c.I_Error(
                    "DSKJOCKY lump \"%.*s\" is missing a lump name at line %lu",
                    new_coll.name.len,
                    new_coll.name.ptr,
                    ii,
                );
            };
            const mus_lmp_trimmed = std.mem.trim(u8, mus_lmp, " \r\n\t");

            const resolved = map.get(mus_lmp_trimmed) orelse {
                c.I_Error(
                    "DSKJOCKY lump \"%.*s\", line %lu lump name `%.*s` was not found",
                    new_coll.name.len,
                    new_coll.name.ptr,
                    ii,
                    mus_lmp_trimmed.len,
                    mus_lmp_trimmed.ptr,
                );
            };

            try new_coll.songs.append(Song{
                .artist = try self.arena.allocator().dupeZ(u8, std.mem.trim(
                    u8,
                    artist,
                    " \r\n\t",
                )),
                .title = try self.arena.allocator().dupeZ(u8, std.mem.trim(
                    u8,
                    title,
                    " \r\n\t",
                )),
                .lump = resolved,
            });
        }

        try self.collections.append(self.arena.allocator(), new_coll);
    }

    if (start_time) |t| {
        const now = std.time.Instant.now() catch unreachable;
        log.info("Music GUI populated in {}ms", .{now.since(t) / 1000 / 1000});
    } else |_| {}
}

pub fn layout(cx: *Core, left: bool, menu_bar_height: f32) void {
    const self = &cx.musicgui;

    const vp_size = if (c.igGetMainViewport()) |vp| vp.*.Size else {
        imgui.report_err_get_main_viewport.call();
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
        c.ImGuiWindowFlags_NoTitleBar |
            c.ImGuiWindowFlags_NoResize |
            c.ImGuiWindowFlags_MenuBar,
    )) return;

    defer c.igEnd();

    if (c.igBeginMenuBar()) {
        defer c.igEndMenuBar();

        if (c.igButton("Stop", .{ .x = 0.0, .y = 0.0 })) {
            c.S_StopMusic(@ptrCast(&cx.c));
        }

        if (c.igButton("Restart", .{ .x = 0.0, .y = 0.0 })) {
            c.S_StopMusic(@ptrCast(&cx.c));
            c.S_RestartMusic(@ptrCast(&cx.c));
        }

        if (c.igButton("Play Random", .{ .x = 0.0, .y = 0.0 })) {
            const coll_i = self.prng.random().intRangeAtMost(
                usize,
                0,
                self.collections.items.len - 1,
            );
            const coll = &self.collections.items[coll_i];
            const song_i = self.prng.random().intRangeAtMost(
                usize,
                0,
                coll.songs.items.len - 1,
            );
            const song = &coll.songs.items[song_i];
            c.S_ChangeMusInfoMusic(@ptrCast(&cx.c), song.lump, @intFromBool(true));
        }

        if (imgui.inputText("Filter##vfsgui.filter", self.filterBufSlice(), .{}, null, null)) {}
        c.igSameLine(0.0, -1.0);
        _ = c.igCheckbox("aA##vfsgui.filter_case_sensitive", &self.filter_case_sensitive);

        if (self.filter_case_sensitive) {
            c.igSetItemTooltip("Filtering: Case Sensitively");
        } else {
            c.igSetItemTooltip("Filtering: Case Insensitively");
        }
    }

    const filter = std.mem.sliceTo(&self.filter_buf, 0);

    for (self.collections.items, 0..) |coll, i| {
        if (!c.igTreeNode_Str(coll.name)) continue;
        defer c.igTreePop();

        if (c.igButton("Play Random", .{ .x = 0.0, .y = 0.0 })) {
            const song_i = self.prng.random().intRangeAtMost(
                usize,
                0,
                coll.songs.items.len - 1,
            );
            const song = &coll.songs.items[song_i];
            c.S_ChangeMusInfoMusic(@ptrCast(&cx.c), song.lump, @intFromBool(true));
        }

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
            if (filter.len > 0) {
                const filt_title = if (self.filter_case_sensitive)
                    std.mem.indexOf(u8, song.title, filter)
                else
                    std.ascii.indexOfIgnoreCase(song.title, filter);

                const filt_artist = if (self.filter_case_sensitive)
                    std.mem.indexOf(u8, song.artist, filter)
                else
                    std.ascii.indexOfIgnoreCase(song.artist, filter);

                if (filt_title) |_| {} else if (filt_artist) |_| {} else continue;
            }

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

fn filterBufSlice(self: *Self) [:0]u8 {
    return self.filter_buf[0..(@sizeOf(@TypeOf(self.filter_buf)) - 1) :0];
}

export fn populateMusicPlayer(ccx: *Core.C) void {
    ccx.core().musicgui.populate() catch
        c.I_Error("Music player population failed: out of memory");
}
