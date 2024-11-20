//! DoomParse's concrete ("green") syntax tree structures.

const builtin = @import("builtin");
const std = @import("std");

const root = @import("root.zig");
const SyntaxKind = root.SyntaxKind;
const TextSize = root.TextSize;
const ThinSlice = @import("thin_slice.zig").ThinSlice;
const TokenKey = root.TokenKey;

/// All elements with this node as a parent
/// fall within allocations provided by `Head.arena`.
pub const Subtree = struct {
    pub const Head = struct {
        kind: SyntaxKind,
        text_len: TextSize,
        arena: *std.heap.ArenaAllocator,
        num_children: u32,
    };

    pub const Data = ThinSlice(Head, Element);

    data: *const Head,

    /// This takes ownership of `arena`, which gets destroyed by `deinit`.
    pub fn initFromSlice(
        arena: *std.heap.ArenaAllocator,
        syntax: SyntaxKind,
        child_elems: []const Element,
    ) std.mem.Allocator.Error!Subtree {
        var text_len: TextSize = 0;
        for (child_elems) |child| text_len += child.textLen();

        const data = try Subtree.Data.init(arena.allocator(), child_elems.len);
        data.head().* = Subtree.Head{
            .kind = syntax,
            .text_len = text_len,
            .arena = arena,
            .num_children = @truncate(child_elems.len),
        };
        for (child_elems, 0..) |child, i| data.items()[i] = child;

        return Subtree{ .data = data.head() };
    }

    pub fn deinit(self: *const Subtree) void {
        const arena = self.data.arena.*;
        arena.child_allocator.destroy(self.data.arena);
        arena.deinit();
    }

    pub fn kind(self: Subtree) SyntaxKind {
        return self.data.kind;
    }

    pub fn textLen(self: Subtree) TextSize {
        return self.data.text_len;
    }

    pub fn children(self: Subtree) []const Element {
        const p: *const Data.Payload = @alignCast(@fieldParentPtr("head", self.data));
        const pa: [*]const Data.Payload = @ptrCast(p);
        const items_ptr: [*]const Element = @ptrCast(pa + 1);

        if (builtin.is_test) {
            std.debug.assert(std.mem.isAligned(@intFromPtr(items_ptr), @alignOf(Element)));
        }

        return items_ptr[0..self.data.num_children];
    }
};

pub const Node = struct {
    pub const Head = struct {
        kind: SyntaxKind,
        text_len: TextSize,
        num_children: u32,
    };

    pub const Data = ThinSlice(Head, Element);

    data: *Head,

    /// This does not take ownership of `arena`, which is meant to come from a `Subtree`.
    pub fn initFromSlice(
        arena: *std.heap.ArenaAllocator,
        syntax: SyntaxKind,
        child_elems: []const Element,
    ) std.mem.Allocator.Error!Node {
        var text_len: TextSize = 0;
        for (child_elems) |child| text_len += child.textLen();

        const data = try Node.Data.init(arena.allocator(), child_elems.len);
        data.head().* = Node.Head{
            .kind = syntax,
            .text_len = text_len,
            .num_children = @truncate(child_elems.len),
        };
        for (child_elems, 0..) |child, i| data.items()[i] = child;

        return Node{ .data = data.head() };
    }

    pub fn kind(self: Node) SyntaxKind {
        return self.data.kind;
    }

    pub fn textLen(self: Node) TextSize {
        return self.data.text_len;
    }

    pub fn children(self: Node) []const Element {
        const p: *const Data.Payload = @alignCast(@fieldParentPtr("head", self.data));
        const pa: [*]const Data.Payload = @ptrCast(p);
        const items_ptr: [*]const Element = @ptrCast(pa + 1);

        if (builtin.is_test) {
            std.debug.assert(std.mem.isAligned(@intFromPtr(items_ptr), @alignOf(Element)));
        }

        return items_ptr[0..self.data.num_children];
    }
};

pub const Leaf = struct {
    pub const Head = struct {
        kind: SyntaxKind,
        text_len: TextSize,
        key: TokenKey,
    };

    data: *const Head,

    pub fn kind(self: Leaf) SyntaxKind {
        return self.data.kind;
    }

    pub fn textLen(self: Leaf) TextSize {
        return self.data.text_len;
    }

    pub fn token(self: Leaf) TokenKey {
        return self.data.key;
    }
};

pub const Element = union(enum) {
    node: Node,
    leaf: Leaf,
    subtree: Subtree,

    pub fn textLen(self: Element) TextSize {
        return switch (self) {
            .node => |n| n.textLen(),
            .leaf => |l| l.textLen(),
            .subtree => |s| s.textLen(),
        };
    }
};
