//! DeHackEd action functions.

const std = @import("std");

const c = @import("main.zig").c;

const Core = @import("Core.zig");

const inventory_burstshotgun: usize = 0;

fn burstShotgunCheckVent(ccx: *Core.C, player: *c.player_t, psp: *c.pspdef_t) callconv(.C) void {
    player.inventory[inventory_burstshotgun].count += 1;

    if (player.inventory[inventory_burstshotgun].count >= 4) {
        const state = std.math.lossyCast(c.statenum_t, psp.state.*.args[0]);
        player.inventory[inventory_burstshotgun].count = 0;
        c.P_SetPspritePtr(@ptrCast(ccx), player, psp, state);
    }
}

fn burstShotgunFire(ccx: *Core.C, player: *c.player_t, _: *c.pspdef_t) callconv(.C) void {
    const hspread: c.fixed_t = 6;
    const vspread: c.fixed_t = 4;
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

// Details /////////////////////////////////////////////////////////////////////

fn bulletSlope(ccx: *Core.C, actor: *c.mobj_t) c.fixed_t {
    var aim: c.aim_t = undefined;
    const tgt_mask = if (c.mbf_features()) c.MF_FRIEND else 0;
    c.dsda_PlayerAimBad(@ptrCast(ccx), actor, actor.angle, &aim, tgt_mask);

    const bulletslope = @extern(*c.fixed_t, .{ .name = "bulletslope" });
    bulletslope.* = aim.slope;
    return bulletslope.*;
}
