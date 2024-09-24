#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#if !defined(RATBOOM_ZIG) // If included by Zig, don't expand to anything.

#define UNREACHABLE \
    do { \
        fprintf(stderr, "unreachable code: %s:%u", __FILE__, __LINE__); \
        abort(); \
    } while (0)

typedef struct SliceU8 { const char* ptr; size_t len; } SliceU8;

/// The returned pointer is not null-terminated!
SliceU8 pathStem(const char* path);

#endif // if !defined(RATBOOM_ZIG)
