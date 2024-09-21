const std = @import("std");
const cfg = @import("cfg");

const install_dir = std.fs.path.basename(cfg.install_prefix);

// Analysis ////////////////////////////////////////////////////////////////////

test "doom2 map 4 nm speed by Vile" {
    try runDemo(.{ .demo = "nm04-036.lmp" });
    try expectAnalysis("nm04-036", "skill", "5");
    try expectAnalysis("nm04-036", "respawn", "0");
    try expectAnalysis("nm04-036", "fast", "0");
}

test "doom2 map 1 uv speed by Thomas Pilger" {
    try runDemo(.{ .demo = "lv01-005.lmp" });
    try expectAnalysis("lv01-005", "skill", "4");
    try expectAnalysis("lv01-005", "nomonsters", "0");
    try expectAnalysis("lv01-005", "100k", "0");
    try expectAnalysis("lv01-005", "100s", "0");
}

test "doom2 map 1 nomonsters by depr4vity" {
    try runDemo(.{ .demo = "lv01o497.lmp" });
    try expectAnalysis("lv01o497", "nomonsters", "1");
}

test "doom2 map 2 uv respawn by Looper" {
    try runDemo(.{ .demo = "re02-107.lmp" });
    try expectAnalysis("re02-107", "respawn", "1");
}

test "doom2 map 4 uv fast by Radek Pecka" {
    try runDemo(.{ .demo = "fa04-109.lmp" });
    try expectAnalysis("fa04-109", "fast", "1");
}

test "doom2 map 1 uv max by Xit Vono" {
    try runDemo(.{ .demo = "lv01-039.lmp" });
    try expectAnalysis("lv01-039", "100k", "1");
    try expectAnalysis("lv01-039", "tyson_weapons", "0");
    try expectAnalysis("lv01-039", "turbo", "0");
}

test "doom2 episode 1 nm100s in 11:56 by JCD" {
    try runDemo(.{ .demo = "1156ns01.lmp" });
    try expectAnalysis("1156ns01", "100s", "1");
}

test "doom2 ep 3 max in 26:54 by Vile" {
    try runDemo(.{ .demo = "lve3-2654.lmp" });
    try expectAnalysis("lve3-2654", "missed_monsters", "0");
    try expectAnalysis("lve3-2654", "missed_secrets", "1");
}

test "pacifist, barrel chain" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "barrel_chain.lmp" });
    try expectAnalysis("barrel_chain", "pacifist", "0");
}

test "pacifist, barrel assist" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "barrel_assist.lmp" });
    try expectAnalysis("barrel_assist", "pacifist", "0");
}

test "pacifist, player shoots Keen" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "keen.lmp" });
    try expectAnalysis("keen", "pacifist", "0");
}

test "pacifist, player shoots Romero" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "romero.lmp" });
    try expectAnalysis("romero", "pacifist", "0");
}

test "pacifist, splash damage" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "splash.lmp" });
    try expectAnalysis("splash", "pacifist", "0");
}

test "pacifist, telefrag" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "telefrag.lmp" });
    try expectAnalysis("telefrag", "pacifist", "1");
}

test "pacifist, player shoots voodoo doll" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "voodoo.lmp" });
    try expectAnalysis("voodoo", "pacifist", "1");
}

test "doom2 map 8 stroller by 4shockblast" {
    try runDemo(.{ .demo = "lv08str037.lmp" });
    try expectAnalysis("lv08str037", "stroller", "1");
}

test "doom2 map 8 pacifist by 4shockblast" {
    try runDemo(.{ .demo = "pa08-020.lmp" });
    try expectAnalysis("pa08-020", "stroller", "0");
}

test "reality, player takes enemy damage" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "damage.lmp" });
    try expectAnalysis("damage", "reality", "0");
    try expectAnalysis("damage", "almost_reality", "0");
}

test "reality, player takes nukage damage" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "nukage.lmp" });
    try expectAnalysis("nukage", "reality", "0");
    try expectAnalysis("nukage", "almost_reality", "1");
}

test "reality, player takes crusher damage" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "crusher.lmp" });
    try expectAnalysis("crusher", "reality", "0");
    try expectAnalysis("crusher", "almost_reality", "0");
}

test "reality, player takes no damage" {
    try runDemo(.{ .pwad = "analysis_test.wad", .demo = "reality.lmp" });
    try expectAnalysis("reality", "reality", "1");
    try expectAnalysis("reality", "almost_reality", "0");
}

test "doom2 map 1 tyson by j4rio" {
    try runDemo(.{ .demo = "lv01t040.lmp" });
    try expectAnalysis("lv01t040", "tyson_weapons", "1");
    try expectAnalysis("lv01t040", "weapon_collector", "0");
}

test "doom2 done turbo quicker by 4shockblast" {
    try runDemo(.{ .demo = "d2dtqr.lmp" });
    try expectAnalysis("d2dtqr", "turbo", "1");
}

test "doom2 map 1 collector by hokis" {
    try runDemo(.{ .demo = "cl01-022.lmp" });
    try expectAnalysis("cl01-022", "weapon_collector", "1");
}

// Sync ////////////////////////////////////////////////////////////////////////

test "doom2 30uv in 17:55 by Looper" {
    try runDemo(.{ .demo = "30uv1755.lmp" });
    try expectTotalTime("30uv1755", "17:55");
}

test "doom2 20 uv max in 2:22 by termrork & kOeGy (a)" {
    try runDemo(.{ .demo = "cm20k222.lmp" });
    try expectTotalTime("cm20k222", "2:22");
}

test "doom2 20 uv max in 2:22 by termrork & kOeGy (b)" {
    try runDemo(.{ .demo = "cm20t222.lmp" });
    try expectTotalTime("cm20t222", "2:22");
}

test "rush 12 uv max in 21:14 by Ancalagon" {
    try runDemo(.{ .pwad = "rush.wad", .demo = "ru12-2114.lmp" });
    try expectTotalTime("ru12-2114", "21:14");
}

test "valiant e1 uv speed in 5:13 by Krankdud" {
    try runDemo(.{ .pwad = "Valiant.wad", .demo = "vae1-513.lmp" });
    try expectTotalTime("vae1-513", "5:13");
}

test "e1 sm max in 52:40 by JCD" {
    try runDemo(.{ .iwad = "DOOM.WAD", .pwad = "HERETIC.WAD", .demo = "h1m-5240.lmp" });
    try expectTotalTime("h1m-5240", "52:40");
}

test "e2 sm max in 67:02 by JCD" {
    try runDemo(.{ .iwad = "DOOM.WAD", .pwad = "HERETIC.WAD", .demo = "h2ma6702.lmp" });
    try expectTotalTime("h2ma6702", "67:02");
}

test "e3 sm max in 62:48 by JCD" {
    try runDemo(.{ .iwad = "DOOM.WAD", .pwad = "HERETIC.WAD", .demo = "h3ma6248.lmp" });
    try expectTotalTime("h3ma6248", "62:48");
}

test "e4 sm speed in 10:55 by veovis" {
    try runDemo(.{ .iwad = "DOOM.WAD", .pwad = "HERETIC.WAD", .demo = "h4sp1055.lmp" });
    try expectTotalTime("h4sp1055", "10:55");
}

test "e5 sm speed in 12:57 by veovis" {
    try runDemo(.{ .iwad = "DOOM.WAD", .pwad = "HERETIC.WAD", .demo = "h5sp1257.lmp" });
    try expectTotalTime("h5sp1257", "12:57");
}

test "e1 sk4 max in 45:37 by PVS" {
    try runDemo(.{ .iwad = "HEXEN.WAD", .demo = "me1c4537.lmp" });
    try expectTotalTime("me1c4537", "45:37");
}

// Details /////////////////////////////////////////////////////////////////////

const alloc = std.testing.allocator;

fn runDemo(args: struct {
    demo: []const u8,
    pwad: []const u8 = "",
    iwad: []const u8 = "DOOM2.WAD",
}) !void {
    const pwad_str = if (std.mem.eql(u8, args.pwad, "HERETIC.WAD"))
        try std.fmt.allocPrint(alloc, "../sample/iwads/{s}", .{args.pwad})
    else
        try std.fmt.allocPrint(alloc, "../sample/pwads/{s}", .{args.pwad});
    defer alloc.free(pwad_str);
    const iwad_str = try std.fmt.allocPrint(alloc, "../sample/iwads/{s}", .{args.iwad});
    defer alloc.free(iwad_str);
    const demo_str = try std.fmt.allocPrint(alloc, "../sample/demos/{s}", .{args.demo});
    defer alloc.free(demo_str);

    const argv_no_pwad = [_][]const u8{
        install_dir ++ "/bin/viletech",
        "-nosound",
        "-nodraw",
        "-levelstat",
        "-analysis",
        "-iwad",
        iwad_str,
        "-fastdemo",
        demo_str,
    };
    const argv_pwad = argv_no_pwad ++ [_][]const u8{ "-file", pwad_str };
    const argv_heretic = argv_pwad ++ [_][]const u8{"-heretic"};
    const argv_hexen = argv_pwad ++ [_][]const u8{"-hexen"};

    const argv = if (args.pwad.len == 0)
        argv_no_pwad[0..]
    else if (std.mem.eql(u8, args.pwad, "HERETIC.WAD"))
        argv_heretic[0..]
    else if (std.mem.eql(u8, args.iwad, "HEXEN.WAD"))
        argv_hexen[0..]
    else
        argv_pwad[0..];

    const result = std.process.Child.run(.{
        .allocator = alloc,
        .argv = argv,
        .cwd = "zig-out",
    }) catch |err| {
        std.debug.print("Command failed: {s}\n", .{argv});
        std.debug.print("Details: {}\n", .{err});
        return;
    };

    alloc.free(result.stdout);
    alloc.free(result.stderr);
}

fn expectAnalysis(demoname: []const u8, key: []const u8, val: []const u8) !void {
    const path = try std.fmt.allocPrint(alloc, "zig-out/analysis.{s}.txt", .{demoname});
    defer alloc.free(path);

    var line_buf: [1024]u8 = undefined;
    var line_iter = try readLines(path, &line_buf, .{});
    defer line_iter.deinit();

    while (try line_iter.next()) |line| {
        var parts = std.mem.splitScalar(u8, line, ' ');
        const part_0 = parts.next().?;
        if (!std.mem.eql(u8, part_0, key)) continue;
        try std.testing.expectEqualStrings(val, parts.next().?);
    }
}

fn expectTotalTime(demoname: []const u8, val: []const u8) !void {
    const path = try std.fmt.allocPrint(alloc, "zig-out/levelstat.{s}.txt", .{demoname});
    defer alloc.free(path);

    var line_buf: [1024]u8 = undefined;
    var line_iter = try readLines(path, &line_buf, .{});
    defer line_iter.deinit();

    var lines = std.ArrayList([]const u8).init(std.testing.allocator);
    defer lines.deinit();

    while (try line_iter.next()) |line| {
        try lines.append(line);
    }

    const last_line = if (lines.items[lines.items.len - 1].len == 0)
        lines.items[lines.items.len - 2] // Properly handle an E.o.F. newline.
    else
        lines.items[lines.items.len - 1];

    var parts = std.mem.splitScalar(u8, last_line, ' ');
    _ = parts.next().?; // e.g. MAP01 or E1M1
    _ = parts.next().?; // -
    const part2 = parts.next().?; // May be level time, may be empty.

    if (part2.len == 0) _ = parts.next().?;

    const part3 = std.mem.trim(u8, parts.next().?, "()");
    try std.testing.expectEqualStrings(val, part3);
}

// Code below is from ZUL: https://github.com/karlseguin/zul
// See legal/zul.txt

const LineIterator = LineIteratorSize(4096);

// Made into a generic so that we can efficiently test files larger than buffer
fn LineIteratorSize(comptime size: usize) type {
    return struct {
        out: []u8,
        delimiter: u8,
        file: std.fs.File,
        buffered: std.io.BufferedReader(size, std.fs.File.Reader),

        const Self = @This();

        pub const Opts = struct {
            open_flags: std.fs.File.OpenFlags = .{},
            delimiter: u8 = '\n',
        };

        pub fn deinit(self: Self) void {
            self.file.close();
        }

        pub fn next(self: *Self) !?[]u8 {
            const delimiter = self.delimiter;

            var out = self.out;
            var written: usize = 0;

            var buffered = &self.buffered;
            while (true) {
                const start = buffered.start;
                if (std.mem.indexOfScalar(u8, buffered.buf[start..buffered.end], delimiter)) |pos| {
                    const written_end = written + pos;
                    if (written_end > out.len) {
                        return error.StreamTooLong;
                    }

                    const delimiter_pos = start + pos;
                    if (written == 0) {
                        // Optimization. We haven't written anything into `out` and we have
                        // a line. We can return this directly from our buffer, no need to
                        // copy it into `out`.
                        buffered.start = delimiter_pos + 1;
                        return buffered.buf[start..delimiter_pos];
                    } else {
                        @memcpy(out[written..written_end], buffered.buf[start..delimiter_pos]);
                        buffered.start = delimiter_pos + 1;
                        return out[0..written_end];
                    }
                } else {
                    // We didn't find the delimiter. That means we need to write the rest
                    // of our buffered content to out, refill our buffer, and try again.
                    const written_end = (written + buffered.end - start);
                    if (written_end > out.len) {
                        return error.StreamTooLong;
                    }
                    @memcpy(out[written..written_end], buffered.buf[start..buffered.end]);
                    written = written_end;

                    // fill our buffer
                    const n = try buffered.unbuffered_reader.read(buffered.buf[0..]);
                    if (n == 0) {
                        return null;
                    }
                    buffered.start = 0;
                    buffered.end = n;
                }
            }
        }
    };
}

fn readLines(file_path: []const u8, out: []u8, opts: LineIterator.Opts) !LineIterator {
    return readLinesSize(4096, file_path, out, opts);
}

fn readLinesSize(comptime size: usize, file_path: []const u8, out: []u8, opts: LineIterator.Opts) !LineIteratorSize(size) {
    const file = blk: {
        if (std.fs.path.isAbsolute(file_path)) {
            break :blk try std.fs.openFileAbsolute(file_path, opts.open_flags);
        } else {
            break :blk try std.fs.cwd().openFile(file_path, opts.open_flags);
        }
    };

    const buffered = std.io.bufferedReaderSize(size, file.reader());
    return .{
        .out = out,
        .file = file,
        .buffered = buffered,
        .delimiter = opts.delimiter,
    };
}
