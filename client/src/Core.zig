const Self = @This();

pub const C = struct {
    core: *Self,
    saved_gametick: i32,
};

c: C,
