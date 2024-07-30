/// @file
/// @brief General-purpose preprocessor macros.

#if defined(__GNUC__)
  // Includes Clang
  #define _U_ __attribute__((unused))
#elif defined(_MSC_VER)
  #define _U_ __pragma(warning(suppress:4100 4189))
#else
  #define _U_
#endif
