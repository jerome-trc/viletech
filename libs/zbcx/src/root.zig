//! # ZBCX

const std = @import("std");

const c = @cImport({
    @cInclude("zbcx.h");
});

// Tests for the wrapper, as well as for the backing C code since unit testing
// in Zig is better than in C by a wide margin.

test "compilation, smoke" {
    const sample = @import("sample");

    const Context = struct {
        const io_vtable = c.zbcx_IoVtable{
            .close = ioClose,
            .@"error" = ioError,
            .read = ioRead,
            .seek = ioSeek,
        };

        const out_vtable = c.zbcx_IoVtable{
            .close = ioClose,
            .@"error" = ioError,
            .write = ioWrite,
        };

        cur_file: []const u8,
        in_stack: std.io.FixedBufferStream([]const u8),
        in_zcommon: std.io.FixedBufferStream([]const u8),
        in_zcommon_h: std.io.FixedBufferStream([]const u8),
        output: std.ArrayList(u8),

        extern "C" fn vsnprintf(
            buffer: [*c]u8,
            bufsz: usize,
            format: [*c]const u8,
            [*c]std.builtin.VaList,
        ) c_int;

        export fn fexists(ctx: ?*anyopaque, path_c: [*c]const u8) bool {
            std.testing.expect(ctx != null) catch unreachable;
            const path = std.mem.sliceTo(path_c, 0);

            for ([_][]const u8{
                "stack.bcs",
                "zcommon.h",
                "zcommon.h.bcs",
                "zcommon.bcs",
            }) |p| {
                if (std.mem.eql(u8, path, p)) return true;
            }

            std.debug.panic("unexpected file existence request: '{s}'", .{path});
        }

        export fn fopen(ctx: ?*anyopaque, path_c: [*c]const u8, modes: [*c]const u8) c.zbcx_Io {
            std.testing.expect(ctx != null) catch unreachable;
            var self: *@This() = @alignCast(@ptrCast(ctx));
            const path = std.mem.sliceTo(path_c, 0);

            blk: {
                for ([_][]const u8{
                    "stack.bcs",
                    "zcommon.h",
                    "zcommon.h.bcs",
                    "zcommon.bcs",
                }) |p| {
                    if (std.mem.eql(u8, path, p)) {
                        self.cur_file = p;
                        break :blk;
                    }
                }

                std.debug.panic("unexpected file open request: '{s}'", .{path});
            }

            std.testing.expectEqualStrings("rb", std.mem.sliceTo(modes, 0)) catch unreachable;

            return c.zbcx_Io{
                .state = ctx,
                .vtable = &io_vtable,
            };
        }

        export fn diag(ctx: ?*anyopaque, flags: c_int, args: [*c]std.builtin.VaList) void {
            std.testing.expect(ctx != null) catch unreachable;
            var pos = c.zbcx_Pos{ .line = 0, .column = 0, .file_id = 0 };

            if ((flags & c.ZBCX_DIAG_FILE) != 0) {
                pos = @cVaArg(args, [*c]c.zbcx_Pos).*;
            }

            const fmt = @cVaArg(args, [*:0]const u8);
            var buf = [_]u8{0} ** 1024;
            const written = vsnprintf(&buf, buf.len, fmt, args);

            if (written < 0) {
                @panic("`diag` failed due to encoding error");
            }

            if ((flags & c.ZBCX_DIAG_ERR) != 0)
                std.log.err("{}:{} {s}", .{ pos.line, pos.column, &buf })
            else if ((flags & c.ZBCX_DIAG_WARN) != 0)
                std.log.warn("{}:{} {s}", .{ pos.line, pos.column, &buf })
            else
                std.log.info("{}:{} {s}", .{ pos.line, pos.column, &buf });
        }

        export fn realpath(ctx: ?*anyopaque, path_c: [*c]const u8) [*c]u8 {
            std.testing.expect(ctx != null) catch unreachable;
            const path = std.mem.sliceTo(path_c, 0);
            const ret = std.heap.c_allocator.dupeZ(u8, path) catch unreachable;
            return ret.ptr; // Leaks...
        }

        export fn ioClose(state: ?*anyopaque) c_int {
            std.testing.expect(state != null) catch unreachable;
            return 0;
        }

        export fn ioError(state: ?*anyopaque) c_int {
            std.testing.expect(state != null) catch unreachable;
            return 0;
        }

        export fn ioSeek(state: ?*anyopaque, offset: c_long, whence: c_int) c_int {
            _ = offset;
            _ = whence;
            std.testing.expect(state != null) catch unreachable;
            unreachable;
        }

        export fn ioRead(dest: ?*anyopaque, size: usize, n: usize, state: ?*anyopaque) c_ulong {
            const self: *@This() = @alignCast(@ptrCast(state));

            var source = if (std.mem.eql(u8, self.cur_file, "stack.bcs"))
                &self.in_stack
            else if (std.mem.eql(u8, self.cur_file, "zcommon.bcs"))
                &self.in_zcommon
            else if (std.mem.eql(u8, self.cur_file, "zcommon.h"))
                &self.in_zcommon_h
            else
                std.debug.panic("unexpected current file: '{s}'", .{self.cur_file});

            std.testing.expect(state != null) catch unreachable;
            const d: [*]u8 = @ptrCast(dest orelse unreachable);
            const to_write = @min(
                size * n,
                (source.getEndPos() catch unreachable) - (source.getPos() catch unreachable),
            );
            return source.read(d[0..to_write]) catch unreachable;
        }

        export fn ioWrite(src: ?*anyopaque, size: usize, n: usize, state: ?*anyopaque) c_ulong {
            std.testing.expect(state != null) catch unreachable;
            var ctx: *@This() = @alignCast(@ptrCast(state));
            const d: [*]u8 = @ptrCast(src orelse unreachable);
            ctx.output.writer().writeAll(d[0..(size * n)]) catch unreachable;
            return size * n;
        }
    };

    var options = c.zbcx_options_init();
    defer c.zbcx_options_deinit(&options);

    var context = Context{
        .cur_file = "",
        .in_stack = std.io.fixedBufferStream(sample.zbcx.stack),
        .in_zcommon = std.io.fixedBufferStream(sample.zbcx.zcommon),
        .in_zcommon_h = std.io.fixedBufferStream(sample.zbcx.zcommon_h),
        .output = std.ArrayList(u8).init(std.testing.allocator),
    };
    defer context.output.deinit();

    options.context = &context;
    options.diag = @ptrCast(&Context.diag);
    options.fexists = Context.fexists;
    options.fopen = Context.fopen;
    options.output = c.zbcx_Io{
        .state = &context,
        .vtable = &Context.out_vtable,
    };
    options.realpath = Context.realpath;
    options.source_file = "stack.bcs";
    options.write_asserts = false;

    const result = c.zbcx_compile(&options);
    try std.testing.expectEqual(std.math.lossyCast(c_uint, c.zbcx_res_ok), result);

    try std.testing.expectEqual(624, context.output.items.len);
    const checksum = std.hash.Crc32.hash(context.output.items);
    try std.testing.expectEqual(0x0c697995, checksum);
}
