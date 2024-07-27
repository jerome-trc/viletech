//! Strong fixed-point decimal number and vector types.

const std = @import("std");

/// Returns the type of a signed fixed-point number.
/// The given number determines how many integral and fractional bits are present.
pub fn Fxp(fracbits: comptime_int) type {
    const Backing = switch (fracbits) {
        16 => i32,
        32 => i64,
        else => @compileError("`fracbits` must be either 16 or 32"),
    };

    return packed struct {
        const Self = @This();

        const frac_bits = fracbits;
        const frac_unit = 1 << frac_bits;

        inner: Backing,

        pub fn fromBits(bits: Backing) Self {
            return .{ .inner = bits };
        }

        pub fn scale(a: Self, b: Self, c: Self) Self {
            const a_64 = @as(i64, a.inner);
            const b_64 = @as(i64, b.inner);
            const c_64 = @as(i64, c.inner);
            return .{ .inner = @truncate(@divTrunc(a_64 * b_64, c_64)) };
        }

        pub fn add(lhs: Self, rhs: Self) Self {
            return .{ .inner = lhs.inner +% rhs.inner };
        }

        pub fn div(lhs: Self, rhs: Self) Self {
            const l = @abs(lhs.inner) >> 14;

            if (l >= @abs(rhs.inner)) {
                const x = (lhs.inner ^ rhs.inner) >> 31;
                return .{ .inner = x ^ std.math.maxInt(Backing) };
            } else {
                const x = @as(i64, lhs.inner) << frac_bits;
                return .{ .inner = @truncate(@divTrunc(x, rhs.inner)) };
            }
        }

        pub fn rem(lhs: Self, rhs: Self) Self {
            // dsda-doom calls this `FixedMod`.
            if ((rhs.inner & (rhs.inner - 1)) != 0) {
                const r = @rem(lhs.inner, rhs.inner);
                return if (r < 0) .{ .inner = r + rhs.inner } else .{ .inner = r };
            } else {
                return .{ .inner = lhs.inner & (rhs.inner - 1) };
            }
        }

        pub fn mul(lhs: Self, rhs: Self) Self {
            const l64 = @as(i64, lhs.inner);
            const r64 = @as(i64, rhs.inner);
            return .{ .inner = @truncate(l64 * r64 >> frac_bits) };
        }

        pub fn sub(lhs: Self, rhs: Self) Self {
            return .{ .inner = lhs.inner -% rhs.inner };
        }
    };
}

pub const I16F16 = Fxp(16);
pub const I32F32 = Fxp(32);

pub fn FVec(fracbits: comptime_int, len: comptime_int) type {
    // TODO: assess if usage of `@Vector` here is faster or slower than machine scalars.

    const Backing = switch (fracbits) {
        16 => i32,
        32 => i64,
        else => @compileError("`fracbits` must be either 16 or 32"),
    };

    switch (len) {
        2, 3, 4 => {},
        else => @compileError("`len` must be 2, 3, or 4"),
    }

    return packed struct {
        const Self = @This();
        const Scalar = Fxp(fracbits);
        const Inner = @Vector(len, Backing);

        const frac_bits = fracbits;
        const frac_unit = 1 << frac_bits;

        inner: Inner,

        pub fn init(scalars: [len]Scalar) Self {
            return Self{ .inner = @bitCast(scalars) };
        }

        pub fn splat(f: Scalar) Self {
            return Self{ .inner = @splat(f.inner) };
        }

        pub fn splatBits(b: Backing) Self {
            return Self{ .inner = @splat(b) };
        }

        pub fn array(self: Self) [len]Scalar {
            return @bitCast(@as([len]Backing, self.inner));
        }

        pub fn x(self: Self) Scalar {
            return Scalar.fromBits(self.inner[0]);
        }

        pub fn xBits(self: Self) Backing {
            return self.inner[0];
        }

        pub fn y(self: Self) Scalar {
            return Scalar.fromBits(self.inner[1]);
        }

        pub fn yBits(self: Self) Backing {
            return self.inner[1];
        }

        pub fn z(self: Self) Scalar {
            return if (len < 3)
                @compileError("vector has no Z component")
            else
                Scalar.fromBits(self.inner[2]);
        }

        pub fn zBits(self: Self) Backing {
            return if (len < 3)
                @compileError("vector has no Z component")
            else
                self.inner[2];
        }

        pub fn w(self: Self) Scalar {
            return if (len < 4)
                @compileError("vector has no W component")
            else
                Scalar.fromBits(self.inner[3]);
        }

        pub fn wBits(self: Self) Backing {
            return if (len < 4)
                @compileError("vector has no W component")
            else
                self.inner[3];
        }

        pub fn add(self: Self, other: Self) Self {
            return Self{ .inner = self.inner + other.inner };
        }

        pub fn div(self: Self, other: Self) Self {
            const s_arr = self.array();
            const o_arr = other.array();
            var q: Inner = undefined;

            inline for (0.., s_arr, o_arr) |i, e_s, e_o| {
                q[i] = e_s.div(e_o).inner;
            }

            return Self{ .inner = q };
        }

        pub fn mul(self: Self, other: Self) Self {
            var s_arr64: [len]i64 = undefined;
            var o_arr64: [len]i64 = undefined;

            inline for (0.., @as([len]Backing, self.inner)) |i, e| {
                s_arr64[i] = e;
            }
            inline for (0.., @as([len]Backing, other.inner)) |i, e| {
                o_arr64[i] = e;
            }

            const I64Vec = @Vector(len, i64);
            const s_64: I64Vec = s_arr64;
            const o_64: I64Vec = o_arr64;

            const p_64 = (s_64 * o_64) >> @splat(frac_bits);
            var p: Inner = undefined;

            inline for (0.., @as([len]i64, p_64)) |i, e| {
                p[i] = @truncate(e);
            }

            return Self{ .inner = p };
        }

        pub fn rem(self: Self, other: Self) Self {
            const s_arr = self.array();
            const o_arr = other.array();
            var q: Inner = undefined;

            inline for (0.., s_arr, o_arr) |i, e_s, e_o| {
                q[i] = e_s.div(e_o).inner;
            }

            return Self{ .inner = q };
        }

        pub fn scale(a: Self, b: Self, c: Self) Self {
            var a_arr64: [len]i64 = undefined;
            var b_arr64: [len]i64 = undefined;
            var c_arr64: [len]i64 = undefined;

            inline for (0.., @as([len]Backing, a.inner)) |i, e| {
                a_arr64[i] = e;
            }
            inline for (0.., @as([len]Backing, b.inner)) |i, e| {
                b_arr64[i] = e;
            }
            inline for (0.., @as([len]Backing, c.inner)) |i, e| {
                c_arr64[i] = e;
            }

            const I64Vec = @Vector(len, i64);
            const a64: I64Vec = a_arr64;
            const b64: I64Vec = b_arr64;
            const c64: I64Vec = c_arr64;

            const d64 = (a64 * b64) / c64;
            var d: Inner = undefined;

            inline for (0.., @as([len]i64, d64)) |i, e| {
                d[i] = @truncate(e);
            }

            return Self{ .inner = d };
        }

        pub fn sub(self: Self, other: Self) Self {
            return Self{ .inner = self.inner - other.inner };
        }
    };
}

const Fx16Vec2 = FVec(16, 2);
const Fx16Vec3 = FVec(16, 3);

test "fixed-point division" {
    const lhs = I16F16.fromBits(40239104);
    const rhs = I16F16.fromBits(25158740);
    try std.testing.expectEqual(104818, lhs.div(rhs).inner);
}

test "fixed-point remainder" {
    const lhs = I16F16.fromBits(40239104);
    const rhs = I16F16.fromBits(25158740);
    try std.testing.expectEqual(15080364, lhs.rem(rhs).inner);
}

test "fixed-point multiplication" {
    const lhs = I16F16.fromBits(40960);
    const rhs = I16F16.fromBits(-2085);
    try std.testing.expectEqual(-1304, lhs.mul(rhs).inner);
}

test "fixed-point scaling" {
    const a = I16F16.fromBits(40239104);
    const b = I16F16.fromBits(25158740);
    const c = I16F16.fromBits(-1304);
    try std.testing.expectEqual(1035433821, I16F16.scale(a, b, c).inner);
}

test "fixed-point vector division" {
    const lhs = Fx16Vec3.splatBits(40239104);
    const rhs = Fx16Vec3.splatBits(25158740);
    try std.testing.expectEqual(
        [_]I16F16{I16F16.fromBits(104818)} ** 3,
        lhs.div(rhs).array(),
    );
}

test "fixed-point vector multiplication" {
    const lhs = Fx16Vec3.splatBits(40960);
    const rhs = Fx16Vec3.splatBits(-2085);
    try std.testing.expectEqual(
        [_]I16F16{I16F16.fromBits(-1304)} ** 3,
        lhs.mul(rhs).array(),
    );
}

test "fixed-point vector scaling" {
    const a = Fx16Vec3.splatBits(40239104);
    const b = Fx16Vec3.splatBits(25158740);
    const c = Fx16Vec3.splatBits(-1304);
    try std.testing.expectEqual(
        [_]I16F16{I16F16.fromBits(1035433821)} ** 3,
        Fx16Vec3.scale(a, b, c).array(),
    );
}
