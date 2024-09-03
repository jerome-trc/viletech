//! Action functions for use by DeHackEd.

const std = @import("std");

const c = @import("main.zig").c;

const Core = @import("Core.zig");
const I16F16 = @import("fxp.zig").I16F16;

const ang90: c.angle_t = 0x40000000;
const angle_to_fine_shift: c_uint = 19;

const invslot_hellbound_shotgun_overload: usize = 0;
const invslot_hellbound_shotgun_shots: usize = invslot_hellbound_shotgun_overload + 1;
const invslot_plut_pistol: usize = invslot_hellbound_shotgun_shots + 1;
const invslot_tnt_ssg: usize = invslot_plut_pistol + 1;

// FD4RB ///////////////////////////////////////////////////////////////////////

fn borstalShotgunCheckOverload(ccx: *Core.C, player: *c.player_t, psp: *c.pspdef_t) callconv(.C) void {
    if (player.inventory[invslot_hellbound_shotgun_overload].count >= 1) {
        const state = std.math.lossyCast(c.statenum_t, psp.state.*.args[0]);
        c.P_SetPspritePtr(@ptrCast(ccx), player, psp, state);
    }
}

fn borstalShotgunCheckReload(ccx: *Core.C, player: *c.player_t, psp: *c.pspdef_t) callconv(.C) void {
    const arg1 = psp.state.*.args[1];
    const thresh: c_longlong = if (arg1 == 0) 3 else arg1;

    if (player.inventory[invslot_hellbound_shotgun_shots].count >= thresh) {
        const state = std.math.lossyCast(c.statenum_t, psp.state.*.args[0]);
        c.P_SetPspritePtr(@ptrCast(ccx), player, psp, state);
    }
}

fn borstalShotgunDischarge(_: *Core.C, player: *c.player_t, _: *c.pspdef_t) callconv(.C) void {
    if (player.inventory[invslot_hellbound_shotgun_shots].count < 3) {
        player.inventory[invslot_hellbound_shotgun_shots].count += 1;
    }
}

fn borstalShotgunClearOverload(_: *Core.C, player: *c.player_t, _: *c.pspdef_t) callconv(.C) void {
    const i = @max(player.inventory[invslot_hellbound_shotgun_overload].count - 1, 0);
    player.inventory[invslot_hellbound_shotgun_overload].count = i;
}

fn borstalShotgunOverload(_: *Core.C, player: *c.player_t, _: *c.pspdef_t) callconv(.C) void {
    if (player.inventory[invslot_hellbound_shotgun_overload].count < 1) {
        player.inventory[invslot_hellbound_shotgun_overload].count += 1;
    }
}

fn borstalShotgunReload(_: *Core.C, player: *c.player_t, _: *c.pspdef_t) callconv(.C) void {
    const i = @max(player.inventory[invslot_hellbound_shotgun_shots].count - 1, 0);
    player.inventory[invslot_hellbound_shotgun_shots].count = i;
}

fn burstShotgunCheckVent(ccx: *Core.C, player: *c.player_t, psp: *c.pspdef_t) callconv(.C) void {
    player.inventory[invslot_tnt_ssg].count += 1;

    if (player.inventory[invslot_tnt_ssg].count >= 4) {
        const state = std.math.lossyCast(c.statenum_t, psp.state.*.args[0]);
        player.inventory[invslot_tnt_ssg].count = 0;
        c.P_SetPspritePtr(@ptrCast(ccx), player, psp, state);
    }
}

fn burstShotgunFire(ccx: *Core.C, player: *c.player_t, _: *c.pspdef_t) callconv(.C) void {
    const hspread: c.fixed_t = 6 << 16;
    const vspread: c.fixed_t = 4 << 16;
    const damagebase: c_int = 3;
    const damagemod: c_int = 3;
    const bulletslope = bulletSlope(ccx, player.mo);

    for (0..2) |_| {
        const damage = (@rem(c.P_Random(c.pr_mbf21), damagemod) + 1) * damagebase;
        const rhsa: c.angle_t = @bitCast(c.P_RandomHitscanAngle(c.pr_mbf21, hspread));
        const angle = @as(c.angle_t, @intCast(player.mo.*.angle)) +% rhsa;
        const slope = bulletslope + c.P_RandomHitscanSlope(c.pr_mbf21, vspread);

        c.P_LineAttack2(@ptrCast(ccx), .{
            .t1 = player.mo,
            .angle = angle,
            .distance = c.MISSILERANGE,
            .slope = slope,
            .damage = damage,
            .flags = c.laf_none,
        });
    }

    for (0..10) |_| {
        const damage = (@rem(c.P_Random(c.pr_mbf21), damagemod) + 1) * damagebase;
        const rhsa: c.angle_t = @bitCast(c.P_RandomHitscanAngle(c.pr_mbf21, hspread));
        const angle = @as(c.angle_t, @intCast(player.mo.*.angle)) +% rhsa;
        const slope = bulletslope + c.P_RandomHitscanSlope(c.pr_mbf21, vspread);

        c.P_LineAttack2(@ptrCast(ccx), .{
            .t1 = player.mo,
            .angle = angle,
            .distance = c.MISSILERANGE,
            .slope = slope,
            .damage = damage,
            .flags = c.laf_painless,
        });
    }
}

fn revolverCheckReload(ccx: *Core.C, player: *c.player_t, psp: *c.pspdef_t) callconv(.C) void {
    player.inventory[invslot_plut_pistol].count += 1;

    if (player.inventory[invslot_plut_pistol].count >= 6) {
        const state = std.math.lossyCast(c.statenum_t, psp.state.*.args[0]);
        player.inventory[invslot_plut_pistol].count = 0;
        c.P_SetPspritePtr(@ptrCast(ccx), player, psp, state);
    }
}

// Generic /////////////////////////////////////////////////////////////////////

fn clearRefire(_: *Core.C, player: *c.player_t, _: *c.pspdef_t) callconv(.C) void {
    player.refire = 0;
}

fn light(_: *Core.C, player: *c.player_t, psp: *c.pspdef_t) callconv(.C) void {
    player.extralight = std.math.lossyCast(c_int, psp.state.*.args[0]);
}

fn lightRandomRange(_: *Core.C, player: *c.player_t, psp: *c.pspdef_t) callconv(.C) void {
    const min_incl = std.math.lossyCast(c_int, psp.state.*.args[0]);
    const max_incl = std.math.lossyCast(c_int, psp.state.*.args[1]);
    player.extralight = boomrngRange(c.pr_mbf21, min_incl, max_incl);
}

fn weaponSoundLoop(_: *Core.C, player: *c.player_t, psp: *c.pspdef_t) callconv(.C) void {
    const sfx_id = std.math.lossyCast(c_int, psp.state.*.args[0]);
    const play_globally = psp.state.*.args[1] != 0;
    const timeout = std.math.lossyCast(c_int, psp.state.*.args[2]);
    c.S_LoopMobjSound(if (play_globally) null else player.mo, sfx_id, timeout);
}

fn weaponSoundRandom(_: *Core.C, player: *c.player_t, psp: *c.pspdef_t) callconv(.C) void {
    const play_globally = psp.state.*.args[4] != 0;
    const which = std.math.lossyCast(usize, boomrngRange(c.pr_mbf21, 0, 3));
    const sfx_id = std.math.lossyCast(c_int, psp.state.*.args[which]);
    c.S_StartMobjSound(if (play_globally) null else player.mo, sfx_id);
}

// Details /////////////////////////////////////////////////////////////////////

fn angleToSlope(a: c_int) I16F16 {
    const ang90s: c_int = @intCast(ang90);
    const angle_to_fine_shift_s: c_int = @intCast(angle_to_fine_shift);

    const ret = if (a > ang90s)
        c.finetangent[0]
    else if (-a > ang90s)
        c.finetangent[c.FINEANGLES / 2 - 1]
    else
        c.finetangent[@intCast((ang90s - a) >> angle_to_fine_shift_s)];

    return I16F16.fromBits(ret);
}

fn boomrngRange(rng_class: c.pr_class_t, min_inclusive: c_int, max_inclusive: c_int) c_int {
    return @rem(c.P_Random(rng_class), max_inclusive) + min_inclusive;
}

fn bulletSlope(ccx: *Core.C, actor: *c.mobj_t) c.fixed_t {
    var aim: c.aim_t = undefined;
    const tgt_mask = if (c.mbf_features()) c.MF_FRIEND else 0;
    c.dsda_PlayerAimBad(@ptrCast(ccx), actor, actor.angle, &aim, tgt_mask);

    const bulletslope = @extern(*c.fixed_t, .{ .name = "bulletslope" });
    bulletslope.* = aim.slope;
    return bulletslope.*;
}

fn degToSlope(a: I16F16) I16F16 {
    if (a.inner >= 0)
        return angleToSlope(@intCast(fixedToAngle(a)))
    else
        return angleToSlope(-@as(c_int, @intCast(fixedToAngle(a.neg()))));
}

fn fixedToAngle(a: I16F16) c.angle_t {
    const a64: u64 = @intCast(a.inner);
    return @truncate(a64 * (0x20000000 / 45) >> I16F16.frac_bits);
}
