//! @file
//! @brief Common components of the C++17 API for the ZDFS library.

#pragma once

#include <cstdint>

#if __cplusplus >= 201703L
#define ZDFS_NODISCARD [[nodiscard]]
#else
#define ZDFS_NODISCARD
#endif

namespace zdfs {
	using LumpNum = int32_t;
	using ULumpNum = uint32_t;
}
