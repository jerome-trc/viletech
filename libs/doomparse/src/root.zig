//! Parsers for all manner of languages in the id Tech 1-descendant ecosystem.

const std = @import("std");

pub const concrete = @import("concrete.zig");
pub const syntax = @import("syntax.zig");

pub const SyntaxKind = u16;
pub const TextSize = u32;

pub const TokenKey = packed struct(u32) {
    index: u31,
    is_static: bool,
};

pub const Interner = struct {
    arena: std.heap.ArenaAllocator,
    map: std.StringArrayHashMapUnmanaged(void),

    pub fn init(allocator: std.mem.Allocator) Interner {
        return Interner{
            .arena = std.heap.ArenaAllocator.init(allocator),
            .map = std.StringArrayHashMapUnmanaged(void){},
        };
    }

    pub fn deinit(self: *Interner) void {
        self.arena.deinit();
    }

    /// All previously interned `TokenKey`s are invalidated by calling this.
    pub fn reset(self: *Interner) void {
        self.map = .{};
        _ = self.arena.reset(.retain_capacity);
    }

    pub fn intern(self: *Interner, lexeme: []const u8) std.mem.Allocator.Error!TokenKey {
        const result = try self.map.getOrPut(self.arena.allocator(), lexeme);
        if (!result.found_existing) result.key_ptr.* = try self.arena.allocator().dupe(u8, lexeme);
        return TokenKey{ .index = @truncate(result.index), .is_static = false };
    }

    pub fn get(self: *const Interner, key: TokenKey) []const u8 {
        std.debug.assert(!key.is_static);
        return self.map.keys()[key.index];
    }

    pub fn tryGet(self: *const Interner, key: TokenKey) ?[]const u8 {
        if (key.is_static) return null;
        if (key.index >= self.map.values().len) return null;
        return self.map.keys()[key.index];
    }
};

pub fn Token(K: type) type {
    comptime {
        if (@typeInfo(K).Enum.tag_type != u16) {
            @compileError("token kind must be a u16-based enum");
        }
    }

    return struct {
        pub const Kind = K;

        kind: Kind,
        start: u32,
        end: u32,

        pub fn textLen(self: @This()) TextSize {
            return self.end - self.start;
        }
    };
}

pub const doomtools = @import("doomtools.zig");

test {
    @import("std").testing.refAllDeclsRecursive(@This());
}
