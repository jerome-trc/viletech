//! ECS components for actors (traditionally known as "things" or "map objects").

const fxp = @import("../fxp.zig");

pub const Space = struct {
    radius: fxp.I16F16,
    height: fxp.I16F16,
};
