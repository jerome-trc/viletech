## Compiles FLECS 3.2.11 (198607d) and wraps its C API.

{.compile: "../../depend/flecs/flecs.c".}

const hFlecs = "<flecs.h>"

type Entity* {.header: hFlecs, importc: "ecs_entity_t".} = distinct uint64
const NullEntity* = 0.Entity

proc `==` *(a, b: Entity): bool {.borrow.}

type
    WorldObj* {.header: hFlecs, importc: "ecs_world_t", incompleteStruct.} = object
    World* = ptr WorldObj

proc init*(_: typedesc[World]): World
    {.importc: "ecs_init", header: hFlecs.}

proc initWithArgs*(_: typedesc[World], argc: cint, argv: cstringArray): World
    {.importc: "ecs_init_w_args", header: hFlecs.}

proc mini*(_: typedesc[World]): World
    {.importc: "ecs_mini", header: hFlecs.}

proc reset*(self: World): cint
    {.importc: "ecs_fini", header: hFlecs.}

proc isFinished*(self: World): bool
    {.importc: "ecs_is_fini", header: hFlecs.}

proc newId*(self: World): Entity
    {.importc: "ecs_new_id", header: hFlecs.}
