//! Shim functions exported to dsda-doom's C code.

const std = @import("std");

const c = @import("main.zig").c;

const Core = @import("Core.zig");
const deh = @import("deh.zig");

comptime {
    @export(deh.burstShotgunCheckVent, .{ .name = "A_BurstShotgunCheckVent" });
    @export(deh.burstShotgunFire, .{ .name = "A_BurstShotgunFire" });
    @export(deh.revolverCheckReload, .{ .name = "A_RevolverCheckReload" });

    @export(deh.weaponSoundRandom, .{ .name = "A_WeaponSoundRandom" });
}

export fn pathStem(path: [*:0]const u8, out_len: *usize) [*]const u8 {
    const slice = std.mem.sliceTo(path, 0);
    const ret = std.fs.path.stem(slice);
    out_len.* = ret.len;
    return ret.ptr;
}

export fn registerPref(ccx: *Core.C, pref_vz: [*:0]const u8) void {
    ccx.core().registerPref(std.mem.sliceTo(pref_vz, 0)) catch
        c.I_Error("Failed to register a preference: out of memory");
}