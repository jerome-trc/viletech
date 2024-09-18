//! Shim functions exported to dsda-doom's C code.

const std = @import("std");

const c = @import("main.zig").c;

const Core = @import("Core.zig");
const deh = @import("deh.zig");
const devgui = @import("devgui.zig");

comptime {
    @export(deh.borstalShotgunCheckOverload, .{ .name = "A_BorstalShotgunCheckOverloaded" });
    @export(deh.borstalShotgunCheckReload, .{ .name = "A_BorstalShotgunCheckReload" });
    @export(deh.borstalShotgunClearOverload, .{ .name = "A_BorstalShotgunClearOverload" });
    @export(deh.borstalShotgunDischarge, .{ .name = "A_BorstalShotgunDischarge" });
    @export(deh.borstalShotgunOverload, .{ .name = "A_BorstalShotgunOverload" });
    @export(deh.borstalShotgunReload, .{ .name = "A_BorstalShotgunReload" });
    @export(deh.burstShotgunCheckVent, .{ .name = "A_BurstShotgunCheckVent" });
    @export(deh.burstShotgunFire, .{ .name = "A_BurstShotgunFire" });
    @export(deh.revolverCheckReload, .{ .name = "A_RevolverCheckReload" });

    @export(deh.clearRefire, .{ .name = "A_ClearRefire" });
    @export(deh.light, .{ .name = "A_Light" });
    @export(deh.lightRandomRange, .{ .name = "A_LightRandomRange" });
    @export(deh.weaponSoundLoop, .{ .name = "A_WeaponSoundLoop" });
    @export(deh.weaponSoundRandom, .{ .name = "A_WeaponSoundRandom" });

    @export(devgui.frameBegin, .{ .name = "dguiFrameBegin" });
    @export(devgui.frameDraw, .{ .name = "dguiFrameDraw" });
    @export(devgui.frameFinish, .{ .name = "dguiFrameFinish" });
    @export(devgui.layout, .{ .name = "dguiLayout" });
    @export(devgui.processEvent, .{ .name = "dguiProcessEvent" });
    @export(devgui.setup, .{ .name = "dguiSetup" });
    @export(devgui.shutdown, .{ .name = "dguiShutdown" });
    @export(devgui.wantsKeyboard, .{ .name = "dguiWantsKeyboard" });
    @export(devgui.wantsMouse, .{ .name = "dguiWantsMouse" });
}

export fn loadLevel(ccx: *Core.C) void {
    const log = std.log.scoped(.game);
    const start_time = std.time.Instant.now();

    c.G_DoLoadLevel(@ptrCast(ccx));

    if (start_time) |t| {
        const now = std.time.Instant.now() catch unreachable;
        log.info("Level loaded in {}ms.", .{now.since(t) / std.time.ns_per_ms});
    } else |_| {}
}

export fn pathStem(path: [*:0]const u8, out_len: *usize) [*]const u8 {
    const slice = std.mem.sliceTo(path, 0);
    const ret = std.fs.path.stem(slice);
    out_len.* = ret.len;
    return ret.ptr;
}

export fn populateMusicPlayer(ccx: *Core.C) void {
    ccx.core().musicgui.populate() catch
        c.I_Error("Music player population failed: out of memory");
}

export fn registerPref(ccx: *Core.C, pref_vz: [*:0]const u8) void {
    ccx.core().registerPref(std.mem.sliceTo(pref_vz, 0)) catch
        c.I_Error("Failed to register a preference: out of memory");
}

export fn windowIcon(size: *i32) [*]const u8 {
    const bytes = @import("assets").viletech_png;
    size.* = bytes.len;
    return bytes;
}
