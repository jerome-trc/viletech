/// @file
/// @brief ZDBSP's entire publicly-exported C interface.

#pragma once

#include <stddef.h>
#include <stdint.h>

#if __STDC_VERSION__ >= 202000L || (defined(__cplusplus) && __cplusplus >= 201703L)
#define nodiscard [[nodiscard]]
#else
#define nodiscard
#endif

#ifdef __cplusplus
extern "C" {
#endif

#define ZDBSP_VERSION "1.19"

/// A 32-bit fixed-point decimal type,
/// comprising a 16-bit integral component and a 16-bit fractional component.
typedef int32_t zdbsp_I16F16;
typedef uint32_t zdbsp_Angle;

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
	int16_t x, y;
} zdbsp_VertexRaw;

typedef struct {
	zdbsp_I16F16 x, y;
} zdbsp_VertexFxp;

typedef struct {
	zdbsp_I16F16 x, y;
	uint32_t index;
} zdbsp_VertexWide;

typedef struct {
	int16_t x, y, dx, dy;
	int16_t bbox[2][4];
	uint16_t children[2];
} zdbsp_NodeRaw;

typedef struct {
	int32_t x, y, dx, dy;
	int16_t bbox[2][4];
	uint32_t children[2];
} zdbsp_NodeEx;

typedef struct {
	int16_t x, y, dx, dy;
	int16_t bbox[2][4];
	uint32_t children[2];
} zdbsp_NodeEx0;

typedef struct {
	zdbsp_I16F16 x, y, dx, dy;
	zdbsp_I16F16 bbox[2][4];
	uint32_t int_children[2];
} zdbsp_NodeFxp;

typedef struct {
	uint16_t v1, v2;
	uint16_t angle, linedef;
	int16_t side, offset;
} zdbsp_SegRaw;

typedef struct {
	uint32_t v1, v2;
	uint16_t angle, linedef;
	int16_t side, offset;
} zdbsp_SegEx;

typedef struct {
	uint16_t v1, v2;
	uint16_t linedef, side, partner;
} zdbsp_SegGl;

typedef struct {
	uint32_t v1, v2;
	uint32_t linedef;
	uint16_t side;
	uint32_t partner;
} zdbsp_SegGlEx;

typedef struct {
	int16_t x, y;
	int16_t angle;
	int16_t type;
	int16_t flags;
} zdbsp_ThingRaw;

typedef struct {
	uint16_t thing_id;
	int16_t x, y, z;
	int16_t angle;
	int16_t type;
	int16_t flags;
	int8_t special;
	int8_t args[5];
} zdbsp_Thing2;

typedef struct {
	uint16_t num_lines;
	uint16_t first_line;
} zdbsp_SubsectorRaw;

typedef struct {
	uint32_t num_lines;
	uint32_t first_line;
} zdbsp_SubsectorEx;

typedef struct {
	const char *key, *value;
} zdbsp_UdmfKey;

typedef struct FLevel* zdbsp_LevelPtr;
typedef struct FWadReader* zdbsp_WadReaderPtr;
typedef struct FProcessor* zdbsp_ProcessorPtr;

typedef void (*zdbsp_NodeVisitor)(void*, const zdbsp_NodeRaw*);
typedef void (*zdbsp_NodeExVisitor)(void*, const zdbsp_NodeEx*);
typedef void (*zdbsp_SegVisitor)(void*, const zdbsp_SegRaw*);
typedef void (*zdbsp_SegGlVisitor)(void*, const zdbsp_SegGl*);
typedef void (*zdbsp_SubsectorVisitor)(void*, const zdbsp_SubsectorRaw*);

nodiscard zdbsp_ProcessFlags zdbsp_processflags_default(void);
nodiscard zdbsp_RejectMode zdbsp_rejectmode_default(void);
nodiscard zdbsp_BlockmapMode zdbsp_blockmapmode_default(void);

/// The returned object is owned by the caller, and should be freed using
/// `zdbsp_wadreader_destroy`.
/// `bytes` must live at least as long as the reader.
nodiscard zdbsp_WadReaderPtr zdbsp_wadreader_new(const uint8_t* bytes);

void zdbsp_wadreader_destroy(zdbsp_WadReaderPtr wad);

/// The returned object is owned by the caller, and should be freed using
/// `zdbsp_processor_destroy`.
/// `wad` must live at least as long as the processor.
/// Note that passing in a `NULL` `config` is valid here.
nodiscard zdbsp_ProcessorPtr
	zdbsp_processor_new(zdbsp_WadReaderPtr wad, const zdbsp_ProcessConfig* config);

/// Note that passing in a `NULL` `config` is valid here.
void zdbsp_processor_run(zdbsp_ProcessorPtr p, const zdbsp_NodeConfig* config);

nodiscard size_t zdbsp_processor_nodesx_count(zdbsp_ProcessorPtr p);

void zdbsp_processor_nodes_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeVisitor);
void zdbsp_processor_segs_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegVisitor);
void zdbsp_processor_ssectors_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorVisitor);

void zdbsp_processor_glnodes_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeVisitor);
void zdbsp_processor_glsegs_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegGlVisitor);
void zdbsp_processor_glssectors_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorVisitor);

void zdbsp_processor_destroy(zdbsp_ProcessorPtr p);

#undef nodiscard

#ifdef __cplusplus
}
#endif // __cplusplus
