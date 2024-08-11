//! The DeHackEd post-processor feature of the CLI.

const std = @import("std");
const log = std.log.scoped(.dehpp);
const BufWriter = std.io.BufferedWriter(4096, std.fs.File.Writer);

const Self = @This();

pub const State = packed struct(u8) {
    inc_arg1: bool = false,
    inc_arg2: bool = false,
    inc_arg3: bool = false,
    inc_arg4: bool = false,
    inc_arg5: bool = false,
    inc_arg6: bool = false,
    inc_arg7: bool = false,
    inc_arg8: bool = false,
};

const actions_with_state_args = std.StaticStringMap(State).initComptime(.{
    .{ "RandomJump", State{ .inc_arg1 = true } },
    .{ "HealChase", State{ .inc_arg1 = true } },
    .{ "JumpIfHealthBelow", State{ .inc_arg1 = true } },
    .{ "JumpIfTargetInSight", State{ .inc_arg1 = true } },
    .{ "JumpIfTargetCloser", State{ .inc_arg1 = true } },
    .{ "JumpIfTracerInSight", State{ .inc_arg1 = true } },
    .{ "JumpIfTracerCloser", State{ .inc_arg1 = true } },
    .{ "JumpIfFlagsSet", State{ .inc_arg1 = true } },
    .{ "WeaponJump", State{ .inc_arg1 = true } },
    .{ "CheckAmmo", State{ .inc_arg1 = true } },
    .{ "RefireTo", State{ .inc_arg1 = true } },
    .{ "GunFlashTo", State{ .inc_arg1 = true } },
    // RatBoom-specific ////////////////////////////////////////////////////////
    .{ "BurstShotgunCheckVent", State{ .inc_arg1 = true } },
    .{ "RevolverCheckReload", State{ .inc_arg1 = true } },
});

arena: std.heap.ArenaAllocator,

pub fn run() !void {
    var cwd = std.fs.cwd();

    var in_file = try cwd.openFile("zig-out/fd4rb.deh", .{});
    defer in_file.close();
    var out_file = try cwd.createFile("zig-out/fd4rb.out.deh", .{});
    defer out_file.close();

    var self = Self{
        .arena = std.heap.ArenaAllocator.init(std.heap.c_allocator),
    };

    defer _ = self.arena.reset(.free_all);

    const in_text = try in_file.readToEndAlloc(self.arena.allocator(), 1024 * 1024 * 64);
    var lines = std.mem.splitAny(u8, in_text, "\r\n");
    var states = std.AutoHashMap(u32, State).init(self.arena.allocator());

    while (lines.next()) |line| {
        if (std.mem.startsWith(u8, line, "[CODEPTR]")) break;
    }

    _ = lines.next();

    while (lines.next()) |line| {
        if (line.len == 0) continue;

        var parts = std.mem.splitScalar(u8, line, ' ');
        const part0 = parts.next() orelse break;
        if (!std.mem.eql(u8, part0, "FRAME")) break;
        const num_str = parts.next().?;
        const num = try std.fmt.parseInt(u32, num_str, 10);
        _ = parts.next().?; // `=`
        const action = parts.next().?;

        if (actions_with_state_args.get(action)) |state| {
            try states.put(num, state);
        }
    }

    lines.reset();
    var out_text = try std.ArrayListUnmanaged(u8).initCapacity(self.arena.allocator(), in_text.len);
    var out_writer = out_text.writer(self.arena.allocator());
    var prev_frame: u32 = 0;

    while (lines.next()) |line| {
        if (line.len == 0) {
            _ = try out_writer.writeByte('\n');
            continue;
        }

        var parts = std.mem.splitScalar(u8, line, ' ');

        const part0 = parts.next() orelse continue;

        if (std.mem.eql(u8, part0, "Frame")) {
            const num_str = parts.next().?;
            prev_frame = try std.fmt.parseInt(u32, num_str, 10);
            try std.fmt.format(out_writer, "Frame {}", .{prev_frame + 10_000});
            continue;
        }

        if (std.mem.eql(u8, part0, "Next")) {
            _ = parts.next().?; // `frame`
            _ = parts.next().?; // `=`
            const num_str = parts.next().?;
            const num = try std.fmt.parseInt(u32, num_str, 10);
            try std.fmt.format(out_writer, "Next frame = {}", .{num + 10_000});
            continue;
        }

        if (std.mem.eql(u8, part0, "Deselect")) {
            _ = parts.next().?; // `frame`
            _ = parts.next().?; // `=`
            const num_str = parts.next().?;
            const num = try std.fmt.parseInt(u32, num_str, 10);
            try std.fmt.format(out_writer, "Deselect frame = {}", .{num + 10_000});
            continue;
        }

        if (std.mem.eql(u8, part0, "Select")) {
            _ = parts.next().?; // `frame`
            _ = parts.next().?; // `=`
            const num_str = parts.next().?;
            const num = try std.fmt.parseInt(u32, num_str, 10);
            try std.fmt.format(out_writer, "Select frame = {}", .{num + 10_000});
            continue;
        }

        if (std.mem.eql(u8, part0, "Bobbing")) {
            _ = parts.next().?; // `frame`
            _ = parts.next().?; // `=`
            const num_str = parts.next().?;
            const num = try std.fmt.parseInt(u32, num_str, 10);
            try std.fmt.format(out_writer, "Bobbing frame = {}", .{num + 10_000});
            continue;
        }

        if (std.mem.eql(u8, part0, "Shooting")) {
            _ = parts.next().?; // `frame`
            _ = parts.next().?; // `=`
            const num_str = parts.next().?;
            const num = try std.fmt.parseInt(u32, num_str, 10);
            try std.fmt.format(out_writer, "Shooting frame = {}", .{num + 10_000});
            continue;
        }

        if (std.mem.eql(u8, part0, "Firing")) {
            _ = parts.next().?; // `frame`
            _ = parts.next().?; // `=`
            const num_str = parts.next().?;
            const num = try std.fmt.parseInt(u32, num_str, 10);
            try std.fmt.format(out_writer, "Firing frame = {}", .{num + 10_000});
            continue;
        }

        if (std.mem.eql(u8, part0, "FRAME")) {
            const num_str = parts.next().?;
            const num = try std.fmt.parseInt(u32, num_str, 10);
            _ = parts.next().?; // `=`
            const action = parts.next().?;
            try std.fmt.format(out_writer, "FRAME {} = {s}", .{ num + 10_000, action });
            continue;
        }

        if (std.mem.eql(u8, part0, "Args1")) {
            _ = parts.next().?; // `=`
            const num_str = parts.next().?;
            const num = try std.fmt.parseInt(u32, num_str, 10);

            if (states.get(prev_frame)) |state| {
                if (state.inc_arg1) {
                    try std.fmt.format(out_writer, "Args1 = {}", .{num + 10_000});
                    continue;
                }
            }

            _ = try out_writer.write(line);
            continue;
        }

        _ = try out_writer.write(line);
    }

    try out_file.writeAll(out_text.items);
    log.info("FD4RB DeHackEd post-processing complete.", .{});
}

fn incFrameHeader() !void {}

fn incNextFrame() !void {}
