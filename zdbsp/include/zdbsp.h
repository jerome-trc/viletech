/// @file
/// @brief ZDBSP's entire publicly-exported C interface.

#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
	/// If no configuration is provided, this is the default.
	ZDBSP_EBM_REBUILD,
	ZDBSP_EBM_CREATE0,
} zdbsp_BlockmapMode;

typedef enum {
	/// If no configuration is provided, this is the default.
	ZDBSP_ERM_DONTTOUCH,
	ZDBSP_ERM_CREATEZEROES,
	ZDBSP_ERM_CREATE0,
	ZDBSP_ERM_REBUILD
} zdbsp_RejectMode;

typedef enum {
	/// Enabled by default.
	ZDBSP_PROCF_BUILDNODES = 1 << 0,
	/// Disabled by default.
	ZDBSP_PROCF_CONFORMNODES = 1 << 2,
	/// Disabled by default.
	ZDBSP_PROCF_NOPRUNE = 1 << 3,
	/// Enabled by default.
	ZDBSP_PROCF_CHECKPOLYOBJS = 1 << 4,

	/// Disabled by default.
	ZDBSP_PROCF_BUILDGLNODES = 1 << 5,
	/// Disabled by default.
	ZDBSP_PROCF_GLONLY = 1 << 6,
	/// Disabled by default.
	ZDBSP_PROCF_V5GL = 1 << 7,
	/// Disabled by default.
	ZDBSP_PROCF_WRITECOMMENTS = 1 << 8,

	/// Disabled by default.
	ZDBSP_PROCF_COMPRESSNODES = 1 << 9,
	/// Disabled by default.
	ZDBSP_PROCF_COMPRESSGLNODES = 1 << 10,
	/// Disabled by default.
	ZDBSP_PROCF_FORCECOMPRESSION = 1 << 11,
} zdbsp_ProcessFlags;

typedef struct {
	zdbsp_ProcessFlags flags;
	zdbsp_RejectMode reject_mode;
	zdbsp_BlockmapMode blockmap_mode;
} zdbsp_ProcessConfig;

typedef struct {
	int32_t aa_preference;
	int32_t max_segs;
	int32_t split_cost;
} zdbsp_NodeConfig;

typedef struct {
	int16_t x, y, dx, dy;
	int16_t bbox[2][4];
	uint16_t children[2];
} zdbsp_MapNode;

typedef struct {
	int32_t x, y, dx, dy;
	int16_t bbox[2][4];
	uint32_t children[2];
} zdbsp_MapNodeEx;

typedef struct FLevel* zdbsp_LevelPtr;
typedef struct FWadReader* zdbsp_WadReaderPtr;
typedef struct FProcessor* zdbsp_ProcessorPtr;

typedef void (*zdbsp_NodeVisitor)(void*, const zdbsp_MapNode*);
typedef void (*zdbsp_NodeExVisitor)(void*, const zdbsp_MapNodeEx*);

/// The returned object is owned by the caller, and should be freed using
/// `zdbsp_wadreader_destroy`.
/// `bytes` must live at least as long as the reader.
zdbsp_WadReaderPtr zdbsp_wadreader_new(const uint8_t* bytes);

void zdbsp_wadreader_destroy(zdbsp_WadReaderPtr wad);

/// The returned object is owned by the caller, and should be freed using
/// `zdbsp_processor_destroy`.
/// `wad` must live at least as long as the processor.
/// Note that passing in a `NULL` `config` is valid here.
zdbsp_ProcessorPtr zdbsp_processor_new(zdbsp_WadReaderPtr wad, const zdbsp_ProcessConfig* config);

/// Note that passing in a `NULL` `config` is valid here.
void zdbsp_processor_run(zdbsp_ProcessorPtr p, const zdbsp_NodeConfig* config);

size_t zdbsp_processor_nodesx_count(zdbsp_ProcessorPtr p);

void zdbsp_processor_nodes_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeVisitor);
void zdbsp_processor_nodesx_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeExVisitor);

void zdbsp_processor_destroy(zdbsp_ProcessorPtr p);

#ifdef __cplusplus
}
#endif // __cplusplus
