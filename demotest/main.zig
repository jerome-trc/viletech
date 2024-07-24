const std = @import("std");

const alloc = std.testing.allocator;

fn runDemo(args: struct {
    demo: []const u8,
    pwad: []const u8 = "",
    iwad: []const u8 = "DOOM2.WAD",
}) !void {
    const pwad_str = try std.fmt.allocPrint(alloc, "../.temp/pwads/{s}", .{args.pwad});
    defer alloc.free(pwad_str);
    const iwad_str = try std.fmt.allocPrint(alloc, "../.temp/iwads/{s}", .{args.iwad});
    defer alloc.free(iwad_str);
    const demo_str = try std.fmt.allocPrint(alloc, "../sample/demos/{s}", .{args.demo});
    defer alloc.free(demo_str);

    const argv_pwad = [_][]const u8{
        "Release/ratboom",
        "-nosound",
        "-nodraw",
        "-levelstat",
        "-analysis",
        "-iwad",
        iwad_str,
        "-file",
        pwad_str,
        "-fastdemo",
        demo_str,
    };
    const argv_no_pwad = [_][]const u8{
        "Release/ratboom",
        "-nosound",
        "-nodraw",
        "-levelstat",
        "-analysis",
        "-iwad",
        iwad_str,
        "-fastdemo",
        demo_str,
    };

    const argv = if (args.pwad.len == 0) argv_no_pwad[0..] else argv_pwad[0..];

    const result = std.process.Child.run(.{
        .allocator = alloc,
        .argv = argv,
        .cwd = "build",
    }) catch |err| {
        std.debug.print("Command failed: {s}\n", .{argv});
        std.debug.print("Details: {}\n", .{err});
        return;
    };

    alloc.free(result.stdout);
    alloc.free(result.stderr);
}

test "nm04-036.lmp" {
    try runDemo(.{ .demo = "nm04-036.lmp" });
    try expectAnalysis("skill", "5");
}

test "vae1-513.lmp" {
    try runDemo(.{ .pwad = "Valiant.wad", .demo = "vae1-513.lmp" });
    try expectTotalTime("5:13");
}

fn expectAnalysis(key: []const u8, val: []const u8) !void {
    var line_buf: [1024]u8 = undefined;
    var line_iter = try readLines("build/analysis.txt", &line_buf, .{});
    defer line_iter.deinit();

    while (try line_iter.next()) |line| {
        var parts = std.mem.splitScalar(u8, line, ' ');
        const part_0 = parts.next().?;
        if (!std.mem.eql(u8, part_0, key)) continue;
        try std.testing.expectEqualStrings(val, parts.next().?);
    }
}

fn expectTotalTime(val: []const u8) !void {
    var line_buf: [1024]u8 = undefined;
    var line_iter = try readLines("build/levelstat.txt", &line_buf, .{});
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
    _ = parts.next().?;
    _ = parts.next().?;
    _ = parts.next().?;
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
