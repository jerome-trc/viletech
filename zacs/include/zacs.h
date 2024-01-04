/// @file
/// @brief The core of ZACS' interface.

#pragma once

#include <stddef.h>
#include <stdint.h>

#if __STDC_VERSION__ >= 202000L || (defined(__cplusplus) && __cplusplus >= 201703L)
#define ZACS_NODISCARD [[nodiscard]]
#else
#define ZACS_NODISCARD
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FBehavior* zacs_Behavior;
typedef struct FBehaviorContainer* zacs_Container;

typedef struct _zacs_SliceU8 {
	uint8_t* ptr;
	size_t len;
} zacs_SliceU8;

typedef struct _zacs_ModuleLoader {
	void* ctx;
	/// Note that it is safe for this to return `NULL`.
	zacs_Behavior (*callback)(void*, const char* name);
} zacs_ModuleLoader;

ZACS_NODISCARD zacs_Container zacs_container_new();

/// Returns non-zero upon error.
ZACS_NODISCARD int32_t zacs_container_load(zacs_Container, const zacs_SliceU8, const zacs_ModuleLoader);

void zacs_container_destroy(zacs_Container);

#ifdef __cplusplus
}
#endif // __cplusplus
