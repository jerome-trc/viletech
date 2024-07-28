//! The interface used by plugins.

const std = @import("std");

pub const PCore = packed struct { ptr: *anyopaque };

pub const OnLoad = *const fn (PCore) callconv(.C) void;
pub const OnUnload = *const fn (PCore) callconv(.C) void;
