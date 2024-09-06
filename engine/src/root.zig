//! A Zig library that fills in the gaps between the other libraries in the
//! VileTech project for consumption by the client.

pub const BoomRng = @import("BoomRng.zig");
pub const fxp = @import("fxp.zig");
pub const gamemode = @import("gamemode.zig");
pub const stdx = @import("stdx.zig");

pub const Fxp = fxp.Fxp;
pub const I16F16 = fxp.I16F16;
pub const I32F32 = fxp.I32F32;
pub const FVec = fxp.FVec;
pub const Fx16Vec2 = fxp.Fx16Vec2;
pub const Fx16Vec3 = fxp.Fx16Vec3;
