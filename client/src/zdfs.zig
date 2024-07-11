//! Wrapper around ZDFS' C API for more ergonomic use by Zig code.

const std = @import("std");

const c = @import("root").c;

pub const VirtualFs = packed struct {
    const Self = @This();

    inner: *c.zdfs_FileSys,

    pub fn init() !Self {
        return Self{
            .inner = c.zdfs_fs_new() orelse return error.FileSysInitNull,
        };
    }

    pub fn deinit(self: Self) void {
        c.zdfs_fs_free(self.inner);
    }
};

pub const Error = error{
    FileSysInitNull,
};
