const builtin = @import("builtin");
const std = @import("std");

/// A pointer to a length integer, followed by that many instances of `Item`.
fn ThinSlice(Head: type, Item: type) type {
    if (@sizeOf(Item) < 1) {
        @compileError("zero-sized types not yet supported");
    }

    return struct {
        const Self = @This();

        pub const alignment = @max(@alignOf(Self), @alignOf(Item));

        pub const Payload = struct {
            head: Head,
            len: usize,
        };

        payload: *align(alignment) Payload,

        pub fn init(alloc: std.mem.Allocator, len: usize) std.mem.Allocator.Error!Self {
            var size = std.mem.alignForward(usize, @sizeOf(Payload), @alignOf(Item));
            size += (@sizeOf(Item) * len);
            const allocation = try alloc.alignedAlloc(u8, alignment, size);
            const payload: *Payload = @ptrCast(@alignCast(allocation));
            payload.len = len;
            return Self{ .payload = payload };
        }

        pub fn deinit(self: Self, alloc: std.mem.Allocator) void {
            const ptr: [*]align(alignment) u8 = @ptrCast(self.payload);
            var size = std.mem.alignForward(usize, @sizeOf(Self), @alignOf(Item));
            size += (@sizeOf(Item) * self.payload.len);
            alloc.free(ptr[0..size]);
        }

        pub fn head(self: Self) *Head {
            return &self.payload.head;
        }

        pub fn items(self: Self) []Item {
            const s: [*]Payload = @ptrCast(self.payload);
            const items_ptr: [*]Item = @ptrCast(s + 1);

            if (builtin.is_test) {
                std.debug.assert(std.mem.isAligned(@intFromPtr(items_ptr), @alignOf(Item)));
            }

            return items_ptr[0..self.payload.len];
        }

        pub fn typeErase(self: Self) TypeErasedThinSlice {
            return TypeErasedThinSlice{ .payload = self.payload };
        }
    };
}

fn TypeErasedThinSliceAligned(comptime alignment: usize) type {
    return struct {
        payload: *align(alignment) anyopaque,

        pub fn cast(
            self: @This(),
            Head: type,
            Item: type,
        ) ThinSlice(Head, Item) {
            return ThinSlice(Head, Item){ .payload = @ptrCast(self.payload) };
        }
    };
}

const TypeErasedThinSlice = struct {
    payload: *anyopaque,

    pub fn cast(
        self: TypeErasedThinSlice,
        Head: type,
        Item: type,
    ) ThinSlice(Head, Item) {
        std.debug.assert(std.mem.isAligned(
            @intFromPtr(self.payload),
            ThinSlice(Head, Item).alignment,
        ));

        return ThinSlice(Head, Item){ .payload = @ptrCast(self.payload) };
    }
};

test ThinSlice {
    // Semantic check, check for leakage, check for misaligned alloc or free...
    const t = struct {
        fn testWithType(Item: type) void {
            const T = ThinSlice(struct { i: Item }, Item);
            const thin = try T.init(std.testing.allocator, 3);
            defer thin.deinit(std.testing.allocator);
            thin.head = .{ .i = 3 };
            thin.items()[0] = 0;
            thin.items()[1] = 1;
            thin.items()[2] = 2;
        }
    };

    t.testWithType(u8);
    t.testWithType(u16);
    t.testWithType(u32);
    t.testWithType(u64);
    t.testWithType(u128);
}
