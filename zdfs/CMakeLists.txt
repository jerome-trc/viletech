cmake_minimum_required(VERSION 3.13...3.19)

project(
  ZDFS
  DESCRIPTION "ZDoom's virtual filesystem as a library"
  VERSION 0.0.0
  LANGUAGES C CXX)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

macro(make_release_only)
  set(CMAKE_C_FLAGS_MINSIZEREL ${CMAKE_C_FLAGS_RELEASE})
  set(CMAKE_C_FLAGS_RELWITHDEBINFO ${CMAKE_C_FLAGS_RELEASE})
  string(REPLACE "/MT " "/MTd " CMAKE_C_FLAGS_DEBUG ${CMAKE_C_FLAGS_RELEASE})
  set(CMAKE_CXX_FLAGS_MINSIZEREL ${CMAKE_CXX_FLAGS_RELEASE})
  set(CMAKE_CXX_FLAGS_RELWITHDEBINFO ${CMAKE_CXX_FLAGS_RELEASE})
  string(REPLACE "/MT " "/MTd " CMAKE_CXX_FLAGS_DEBUG
                 ${CMAKE_CXX_FLAGS_RELEASE})
endmacro()

include(CheckFunctionExists)

check_function_exists(stricmp STRICMP_EXISTS)

if(NOT STRICMP_EXISTS)
  add_definitions(-Dstricmp=strcasecmp)
endif()

check_function_exists(strnicmp STRNICMP_EXISTS)

if(NOT STRNICMP_EXISTS)
  add_definitions(-Dstrnicmp=strncasecmp)
endif()

if(PROJECT_SOURCE_DIR STREQUAL CMAKE_SOURCE_DIR)
  # This project is being built standalone

  # Give user option to build shared or static
  option(BUILD_SHARED_LIBS "Build shared libraries" ON)

  # Enable install rules
  set(ZDFS_INSTALL ON)
else()
  # This project is being vendored by another project, set option default if the
  # parent project doesn't provide them.

  if(NOT DEFINED BUILD_SHARED_LIBS)
    set(BUILD_SHARED_LIBS ON)
  endif()

  # Although install rules can be avoided with EXCLUDE_FROM_ALL on
  # add_subdirectory, the EXPORT rules may place certain usage requirements on
  # targets shared between the two projects.
  if(NOT DEFINED ZDFS_INSTALL)
    set(ZDFS_INSTALL OFF)
  endif()
endif()

add_subdirectory(bzip2)
add_subdirectory(lzma)
add_subdirectory(miniz)

set(ZDFS_SOURCES
    src/7z.cpp
    src/directory.cpp
    src/grp.cpp
    src/resourcefile.cpp
    src/wad.cpp
    src/ancientzip.cpp
    src/files.cpp
    src/hog.cpp
    src/rff.cpp
    src/whres.cpp
    src/lump.cpp
    src/ssi.cpp
    src/critsec.cpp
    src/filesystem.cpp
    src/stringpool.cpp
    src/zip.cpp
    src/findfile.cpp
    src/mvl.cpp
    src/unicode.cpp
    src/decompress.cpp
    src/pak.cpp
    utf8proc/utf8proc.c)

add_library(zdfs ${ZDFS_SOURCES})

target_include_directories(zdfs PUBLIC include bzip2 lzma/C)
target_include_directories(zdfs SYSTEM PRIVATE miniz utf8proc)

target_link_libraries(zdfs bz2 lzma miniz)

if(ZDFS_INSTALL)
  install(
    FILES
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/zdfs
    COMPONENT devel)
endif()
