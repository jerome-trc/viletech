//! Wrapper around FLECS' C API for more ergonomic use by Zig code.

const std = @import("std");
const log = std.log.scoped(.flecs);

const c = @import("flecs.h.zig");

pub const Entity = c.ecs_entity_t;
pub const FTime = c.ecs_ftime_t;
pub const Id = c.ecs_id_t;
pub const IdRecord = c.ecs_id_record_t;
pub const Iter = c.ecs_iter_t;
pub const Mixins = c.ecs_mixins_t;

pub const EntityDesc = extern struct {
    /// Used for validity testing. Must be 0.
    _canary: i32 = 0,
    id: Entity,
    parent: Entity,
    name: [*:0]const u8,
    sep: ?[*:0]const u8 = null,
    root_sep: ?[*:0]const u8 = null,
    symbol: ?[*:0]const u8 = null,
    use_low_id: bool = false,
    add: [*:0]const Id = &[_:0]Id{},
    set: [*:Value.nil]const Value = &[_:Value.nil]Value{},
    add_expr: [*:0]const u8 = "",
};

pub const Value = extern struct {
    valtype: Entity,
    ptr: ?*anyopaque,

    const nil = Value{
        .valtype = 0,
        .ptr = null,
    };
};

pub const World = packed struct {
    const Self = @This();

    ptr: *c.ecs_world_t,

    pub fn init() !Self {
        return Self{
            .ptr = c.ecs_init() orelse return error.WorldInitNull,
        };
    }

    pub fn deinit(self: Self) !void {
        if (c.ecs_fini(self.ptr) != 0) {
            return error.WorldDeinitFail;
        }
    }

    pub fn defineComponent(self: Self, T: type) !void {
        var id: Entity = 0;
        var desc = std.mem.zeroes(c.ecs_component_desc_t);
        var edesc = std.mem.zeroes(EntityDesc);
        edesc.id = id;
        edesc.use_low_id = true;
        edesc.name = @typeName(T);
        edesc.symbol = @typeName(T);
        desc.entity = self.entityInit(&edesc);
        desc.type.size = @sizeOf(T);
        desc.type.alignment = @alignOf(T);
        id = c.ecs_component_init(self.ptr, &desc);

        if (id == 0) {
            return error.CreateComponentFail;
        }
    }

    pub fn delete(self: Self, e: Entity) void {
        c.ecs_delete(self.ptr, e);
    }

    pub fn entityInit(self: Self, desc: *const EntityDesc) Entity {
        return c.ecs_entity_init(self.ptr, @ptrCast(desc));
    }

    pub fn getName(self: Self, e: Entity) [*:0]const u8 {
        return @ptrCast(c.ecs_get_name(self.ptr, e));
    }

    pub fn isAlive(self: Self, e: Entity) bool {
        return c.ecs_is_alive(self.ptr, e);
    }

    pub fn lookup(self: Self, path: [:0]const u8) Entity {
        return c.ecs_lookup(self.ptr, path.ptr);
    }

    pub fn new(self: Self) Entity {
        return c.ecs_new(self.ptr);
    }
};

pub const Error = error{
    CreateComponentFail,
    WorldDeinitFail,
    WorldInitNull,
};

pub const null_entity: Entity = 0;
