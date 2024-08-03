//! Shim functions exported to dsda-doom's C code.

const std = @import("std");

const c = @import("main.zig").c;

const Core = @import("Core.zig");

export fn registerPref(ccx: *Core.C, pref_vz: [*:0]const u8) void {
    ccx.core.registerPref(std.mem.sliceTo(pref_vz, 0)) catch
        c.I_Error("Failed to register a preference: out of memory");
}
