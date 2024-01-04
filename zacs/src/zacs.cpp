/// @file
/// @brief Implements zacs.h.

#include "zacs.hpp"

zacs_Container zacs_container_new() {
	return new FBehaviorContainer();
}

int32_t zacs_container_load(zacs_Container ctr, const zacs_SliceU8 slice, const zacs_ModuleLoader mloader) {
	if (!ctr->LoadModule(slice, mloader)) {
		return 1;
	}

	return 0;
}

void zacs_container_destroy(zacs_Container ctr) {
	delete ctr;
}
