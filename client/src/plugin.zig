//! The interface used by plugins.

const std = @import("std");

pub const PCore = extern struct {
    prefs: *const std.StringHashMap(Pref),
};

/// Preferences are a generalized system for passing configuration to plugins
/// through command line arguments.
pub const Pref = union(enum) {
    boolean: bool,
    float: f64,
    int: i64,
    string: []const u8,
};

pub const OnLoad = *const fn (PCore) callconv(.C) void;
pub const OnUnload = *const fn (PCore) callconv(.C) void;
