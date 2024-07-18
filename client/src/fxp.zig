const std = @import("std");

/// Returns the type of a signed fixed-point number.
/// The given number determines how many integral and fractional bits are present.
pub fn Fxp(comptime fracbits: u8) type {
    const Backing = if (fracbits == 16) i32 else if (fracbits == 32) i64 else @compileError("`fracbits` must be either 16 or 32");

    return packed struct {
        const Self = @This();

        const frac_bits = fracbits;
        const frac_unit = 1 << frac_bits;

        inner: Backing,

        pub fn from_bits(bits: Backing) Self {
            return .{ .inner = bits };
        }

        pub fn scale(a: Self, b: Self, c: Self) Self {
            const a_64 = @as(i64, a.inner);
            const b_64 = @as(i64, b.inner);
            const c_64 = @as(i64, c.inner);
            return .{ .inner = @truncate(@divTrunc(a_64 * b_64, c_64)) };
        }

        pub fn add(lhs: Self, rhs: Self) Self {
            return .{ .inner = lhs.inner + rhs.inner };
        }

        pub fn div(lhs: Self, rhs: Self) Self {
            const l = @abs(lhs.inner) >> 14;

            if (l >= @abs(rhs.inner)) {
                const x = (lhs.inner ^ rhs.inner) >> 31;
                return .{ .inner = x ^ std.math.maxInt(Backing) };
            } else {
                const x = @as(i64, lhs.inner) << frac_bits;
                // return .{ .inner = @as(Backing, x / rhs.inner) };
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
            return .{ .inner = lhs.inner - rhs.inner };
        }
    };
}

pub const I16F16 = Fxp(16);
pub const I32F32 = Fxp(32);

test "fixed-point division" {
    const lhs = I16F16.from_bits(40239104);
    const rhs = I16F16.from_bits(25158740);
    try std.testing.expectEqual(104818, lhs.div(rhs).inner);
}

test "fixed-point remainder" {
    const lhs = I16F16.from_bits(40239104);
    const rhs = I16F16.from_bits(25158740);
    try std.testing.expectEqual(15080364, lhs.rem(rhs).inner);
}

test "fixed-point multiplication" {
    const lhs = I16F16.from_bits(40960);
    const rhs = I16F16.from_bits(-2085);
    try std.testing.expectEqual(-1304, lhs.mul(rhs).inner);
}

test "fixed-point scaling" {
    const a = I16F16.from_bits(40239104);
    const b = I16F16.from_bits(25158740);
    const c = I16F16.from_bits(-1304);
    try std.testing.expectEqual(1035433821, I16F16.scale(a, b, c).inner);
}
