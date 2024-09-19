//! @file
//! @brief C99 API for the ZDFS library.

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#if defined(__cplusplus)
extern "C" {
#endif

#if __STDC_VERSION__ >= 202000L || (defined(__cplusplus) && __cplusplus >= 201703L)
#define ZDFS_NODISCARD [[nodiscard]]
#else
#define ZDFS_NODISCARD
#endif

#define ZDFS_NILLUMP (-1)

typedef int32_t zdfs_WadNum;
typedef int32_t zdfs_LumpNum;
typedef uint32_t zdfs_ULumpNum;

enum {
	zdfs_entf_maybeflat = 1,
	zdfs_entf_fullpath = 2,
	zdfs_entf_embedded = 4,
	zdfs_entf_shortname = 8,
	zdfs_entf_compressed = 16,
	zdfs_entf_needfilestart = 32,
};
typedef uint16_t zdfs_EntryFlags;

enum {
	zdfs_msglevel_error = 1,
	zdfs_msglevel_warning = 2,
	zdfs_msglevel_attention = 3,
	zdfs_msglevel_message = 4,
	zdfs_msglevel_debugwarn = 5,
	zdfs_msglevel_debugnotify = 6,
};
typedef int zdfs_MessageLevel;

enum {
	zdfs_ns_hidden = -1,

	zdfs_ns_global = 0,
	zdfs_ns_sprites,
	zdfs_ns_flats,
	zdfs_ns_colormaps,
	zdfs_ns_acslibrary,
	zdfs_ns_newtextures,
	zdfs_ns_bloodraw, //< Unused by GZDoom.
	zdfs_ns_bloodsfx, //< Unused by GZDoom.
	zdfs_ns_bloodmisc, //< Unused by GZDoom.
	zdfs_ns_strifevoices,
	zdfs_ns_hires,
	zdfs_ns_voxels,

	// These namespaces are only used to mark lumps in special subdirectories
	// so that their contents doesn't interfere with the global namespace.
	// searching for data in these namespaces works differently for lumps coming
	// from Zips or other files.
	zdfs_ns_specialzipdirectory,
	zdfs_ns_sounds,
	zdfs_ns_patches,
	zdfs_ns_graphics,
	zdfs_ns_music,

	zdfs_ns_firstskin,
};
typedef int zdfs_Namespace;

/// @see zdfs_fs_new
typedef struct zdfs_FileSys zdfs_FileSys;
typedef struct zdfs_StringVector zdfs_StringVector;

/// An opaque memory buffer to the file's content.
/// Can either own the memory or just point to an external buffer.
typedef struct zdfs_EntryData {
	void* memory;
	size_t length;
	bool owned;
} zdfs_EntryData;

typedef union zdfs_ShortName {
	char string[9];
	/// For accessing the first 4 or 8 chars of `string` as a unit
	/// without breaking strict aliasing rules.
	uint32_t dword;
	/// For accessing the first 4 or 8 chars of `string` as a unit
	/// without breaking strict aliasing rules.
	uint64_t qword;
} zdfs_ShortName;

typedef struct zdfs_FolderEntry {
	const char* name;
	zdfs_ULumpNum num;
} zdfs_FolderEntry;

typedef int32_t (*zdfs_MsgFunc)(zdfs_MessageLevel, const char* fmt, ...);

void zdfs_set_main_thread(void);

/// @returns A pointer to an empty filesystem instance allocated on the heap.
ZDFS_NODISCARD zdfs_FileSys* zdfs_fs_new(zdfs_MsgFunc);

/// @brief Destroys a filesystem instance.
void zdfs_fs_free(const zdfs_FileSys*);

/// If the requested entry is absent, 0 will be returned (just as if the requested
/// entry exists but has no flags set), but `exists` will be set to `false`.
/// Beware that this function *assumes that `exists` is non-null!*
ZDFS_NODISCARD zdfs_EntryFlags zdfs_fs_entry_flags(const zdfs_FileSys*, bool* exists);

/// Will return `NULL` if the given lump number is invalid.
ZDFS_NODISCARD const char* zdfs_fs_entry_fullname(const zdfs_FileSys*, zdfs_LumpNum);

/// Returns the buffer size needed to load the given lump.
///
/// If the requested entry is absent, 0 will be returned (just as if the requested
/// entry exists but has length 0), but `exists` will be set to `false`.
/// Beware that this function *assumes that `exists` is non-null!*
ZDFS_NODISCARD size_t zdfs_fs_entry_len(const zdfs_FileSys*, zdfs_LumpNum, bool* exists);

/// Beware that this function *assumes that `dest` is non-null!*
/// @returns `true` if the read succeeded; `false` if the given lump number is
/// invalid, or if the read did not return all of the entry's bytes.
bool zdfs_fs_entry_read(const zdfs_FileSys*, zdfs_LumpNum, void* dest);

/// Will return `NULL` if the given lump number is invalid.
ZDFS_NODISCARD const char* zdfs_fs_entry_shortname(const zdfs_FileSys*, zdfs_LumpNum);

void zdfs_fs_init_hash_chains(zdfs_FileSys*);

bool zdfs_fs_mount(zdfs_FileSys*, const char* path);

/// This consumes `paths`; the caller should not access or free it afterwards,
/// regardless of whether this returns `true` (success) or `false` (failure).
bool zdfs_fs_mount_multi(zdfs_FileSys*, zdfs_StringVector* paths, bool allow_duplicates);

ZDFS_NODISCARD size_t zdfs_fs_num_files(const zdfs_FileSys*);

ZDFS_NODISCARD size_t zdfs_fs_num_entries(const zdfs_FileSys*);

// zdfs_StringVector methods ///////////////////////////////////////////////////

ZDFS_NODISCARD zdfs_StringVector* zdfs_strvec_new(size_t capacity);
void zdfs_strvec_push(zdfs_StringVector*, const char*);
void zdfs_strvec_free(zdfs_StringVector*);

#if defined(__cplusplus)
}
#endif
