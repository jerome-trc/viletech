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

/// The returned pointer is not null-terminated!
const char* pathStem(const char* path, size_t* out_len);

#endif // if !defined(RATBOOM_ZIG)
