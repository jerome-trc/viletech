//! Wrapper around ZDFS' C API for more ergonomic use by Zig code.

const std = @import("std");
const log = std.log.scoped(.zdfs);

const c = @import("main.zig").c;

pub const VirtualFs = packed struct {
    const Self = @This();

    inner: *c.zdfs_FileSys,

    pub fn init() !Self {
        return Self{
            .inner = c.zdfs_fs_new(messageCallback) orelse return error.FileSysInitNull,
        };
    }

    pub fn deinit(self: Self) void {
        c.zdfs_fs_free(self.inner);
    }
};

pub const Error = error{
    FileSysInitNull,
};

extern "C" fn snprintf(buffer: [*c]u8, bufsz: usize, format: [*c]const u8, ...) c_int;

threadlocal var msg_cb_buf = [_]u8{0} ** 1024;

fn messageCallback(level: c.zdfs_MessageLevel, fmt: [*c]const u8, ...) callconv(.C) c_int {
    var args = @cVaStart();
    defer @cVaEnd(&args);

    const written = snprintf(&msg_cb_buf, msg_cb_buf.len, fmt, args);

    if (written < 0) {
        log.err("ZDFS message callback failed due to encoding error.", .{});
        return 0;
    }

    switch (level) {
        c.zdfs_msglevel_error => {
            log.err("{s}", .{&msg_cb_buf});
        },
        c.zdfs_msglevel_warning => {
            log.warn("{s}", .{&msg_cb_buf});
        },
        c.zdfs_msglevel_attention, c.zdfs_msglevel_message => {
            log.info("{s}", .{&msg_cb_buf});
        },
        c.zdfs_msglevel_debugwarn, c.zdfs_msglevel_debugnotify => {
            log.debug("{s}", .{&msg_cb_buf});
        },
        else => {
            return 0;
        },
    }

    return written;
}
