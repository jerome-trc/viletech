cmake_minimum_required (VERSION 3.1)
project (zt-bcc)
add_executable( zt-bcc
        src/parse/token/user.c
        src/parse/phase.c
        src/semantic/phase.c
        src/codegen/phase.c
        src/version.c
        src/task.c
        src/gbuf.c
        src/common.c
        src/builtin.c
        src/semantic/asm.c
        src/semantic/dec.c
        src/semantic/expr.c
        src/semantic/stmt.c
        src/semantic/type.c
        src/parse/asm.c
        src/parse/dec.c
        src/parse/expr.c
        src/parse/library.c
        src/parse/stmt.c
        src/parse/token/dirc.c
        src/parse/token/expr.c
        src/parse/token/info.c
        src/parse/token/output.c
        src/parse/token/queue.c
        src/parse/token/source.c
        src/parse/token/stream.c
        src/codegen/asm.c
        src/codegen/chunk.c
        src/codegen/dec.c
        src/codegen/expr.c
        src/codegen/linear.c
        src/codegen/obj.c
        src/codegen/pcode.c
        src/codegen/stmt.c
        src/cache/archive.c
        src/cache/cache.c
        src/cache/field.c
        src/cache/library.c
        src/main.c)
target_include_directories(zt-bcc PUBLIC
        src/parse
        src/codegen
        src/cache
        src/semantic
        src)
set_property(TARGET zt-bcc PROPERTY C_STANDARD 99)
