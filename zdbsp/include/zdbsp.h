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

/// @see zdbsp_ProcessConfig
typedef enum {
	/// If no configuration is provided, this is the default.
	ZDBSP_EBM_REBUILD,
	ZDBSP_EBM_CREATE0,
} zdbsp_BlockmapMode;

/// @see zdbsp_ProcessConfig
typedef enum {
	/// If no configuration is provided, this is the default.
	ZDBSP_ERM_DONTTOUCH,
	ZDBSP_ERM_CREATEZEROES,
	ZDBSP_ERM_CREATE0,
	ZDBSP_ERM_REBUILD
} zdbsp_RejectMode;

/// @see zdbsp_ProcessConfig
typedef enum {
	/// Enabled by default.
	ZDBSP_PROCF_BUILDNODES = 1 << 0,
	/// Disabled by default. "Conforming" GL nodes are those which use the same
	/// basic information as non-GL nodes. This results in sub-optimal non-GL nodes
	/// but makes it easier to compare the two sets of nodes to verify the correctness
	/// of the GL nodes.
	ZDBSP_PROCF_CONFORMNODES = 1 << 2,
	/// Disabled by default. "Pruning" is the process by which the node builder:
	/// - removes 0-length lines
	/// - removes sides not referenced by any lines
	/// - removes sectors not referenced by any sides
	/// 0-length line removal cannot be disabled, but setting this flag prevents
	/// removal of extraneous sides and sectors.
	ZDBSP_PROCF_NOPRUNE = 1 << 3,
	/// Enabled by default.
	ZDBSP_PROCF_CHECKPOLYOBJS = 1 << 4,

	/// Disabled by default.
	ZDBSP_PROCF_BUILDGLNODES = 1 << 5,
	/// Disabled by default.
	ZDBSP_PROCF_GLONLY = 1 << 6,
	/// Disabled by default. Note that this will be forced anyway if one of the
	/// following conditions is met during a processor run:
	/// - there is a combined total of more than 32767 vertices between
	/// raw input and generated GL vertices
	/// - more than 65534 GL segs get built
	/// - more than 32767 GL nodes get built
	/// - more than 32767 GL subsectors get built
	ZDBSP_PROCF_V5GL = 1 << 7,
	/// Disabled by default.
	ZDBSP_PROCF_WRITECOMMENTS = 1 << 8,

	/// Disabled by default. Note that this will be forced anyway if one of the
	/// following conditions is met during a processor run:
	/// - the processor determines that GL nodes need to be compressed
	/// - there is a combined total of more than 65535 vertices between
	/// raw input and generated GL vertices
	/// - more than 65535 segs get built
	/// - more than 32767 subsectors get built
	/// - more than 32767 nodes get built
	///
	/// @see ZDBSP_PROCF_COMPRESSGLNODES
	ZDBSP_PROCF_COMPRESSNODES = 1 << 9,
	/// Disabled by default. Note that this will be forced anyway if one of the
	/// following conditions is met during a processor run:
	/// - there is a combined total of more than 32767 vertices between
	/// raw input and generated GL vertices
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
	/// The cost for avoiding diagonal splitters (16 by default).
	int32_t aa_preference;
	/// The maximum number of segs to consider at each node (64 by default).
	int32_t max_segs;
	/// The cost to split a seg (8 by default).
	int32_t split_cost;
} zdbsp_NodeConfig;

/// A level vertex as per the original WAD format.
typedef struct {
	int16_t x, y;
} zdbsp_VertexRaw;

/// A level vertex in terms of 32-bit fixed-point numbers.
typedef struct {
	zdbsp_I16F16 x, y;
} zdbsp_VertexFxp;

typedef struct {
	zdbsp_I16F16 x, y;
	int32_t index;
} zdbsp_VertexEx;

/// A binary space partition tree node as per the original WAD format.
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
} zdbsp_NodeExO;

/// A binary space partition tree node in terms of 32-bit fixed-point numbers.
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

typedef enum {
	ZDBSP_NODEVERS_UNKNOWN,
	ZDBSP_NODEVERS_1,
	ZDBSP_NODEVERS_2,
	ZDBSP_NODEVERS_3,
} zdbsp_NodeVersion;

/// If a processor reports a version below `ZDBSP_NODEVERS_2`,
/// you should be serializing these to WAD entries.
typedef struct {
	uint32_t v1;
	uint32_t partner;
	uint16_t linedef;
	uint8_t side;
} zdbsp_SegGlXV1;

/// If a processor reports a version at or above `ZDBSP_NODEVERS_2`,
/// you should be serializing these to WAD entries.
typedef struct {
	uint32_t v1;
	uint32_t partner;
	uint32_t linedef;
	uint8_t side;
} zdbsp_SegGlXV2V3;

typedef struct FLevel* zdbsp_LevelPtr;
typedef struct FWadReader* zdbsp_WadReaderPtr;
typedef struct FProcessor* zdbsp_ProcessorPtr;

typedef void (*zdbsp_NodeVisitor)(void*, const zdbsp_NodeRaw*);
typedef void (*zdbsp_NodeExVisitor)(void*, const zdbsp_NodeEx*);
typedef void (*zdbsp_NodeExOVisitor)(void*, const zdbsp_NodeExO*);
typedef void (*zdbsp_SegVisitor)(void*, const zdbsp_SegRaw*);
typedef void (*zdbsp_SegGlVisitor)(void*, const zdbsp_SegGl*);
typedef void (*zdbsp_SegGlExVisitor)(void*, const zdbsp_SegGlEx*);
typedef void (*zdbsp_SubsectorVisitor)(void*, const zdbsp_SubsectorRaw*);
typedef void (*zdbsp_SubsectorExVisitor)(void*, const zdbsp_SubsectorEx*);
typedef void (*zdbsp_VertexExVisitor)(void*, const zdbsp_VertexEx*);

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

void zdbsp_processor_destroy(zdbsp_ProcessorPtr p);

/// Note that passing in a `NULL` `config` is valid here.
void zdbsp_processor_run(zdbsp_ProcessorPtr p, const zdbsp_NodeConfig* config);

/// Notes:
/// - If the processor has not been run yet, it will always return `ZDBSP_NODEVERS_UNKNOWN`.
/// - It will also return `ZDBSP_NODEVERS_UNKNOWN` if the last run did not build any GL nodes.
nodiscard zdbsp_NodeVersion zdbsp_processor_nodeversion(zdbsp_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
nodiscard size_t zdbsp_processor_nodesx_count(zdbsp_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
nodiscard size_t zdbsp_processor_nodesgl_count(zdbsp_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
nodiscard size_t zdbsp_processor_ssectorsgl_count(zdbsp_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
nodiscard size_t zdbsp_processor_segsglx_count(zdbsp_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
nodiscard size_t zdbsp_processor_vertsorig_count(zdbsp_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
nodiscard size_t zdbsp_processor_vertsgl_count(zdbsp_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
nodiscard size_t zdbsp_processor_vertsnew_count(zdbsp_ProcessorPtr p);

void zdbsp_processor_nodes_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeVisitor);
void zdbsp_processor_nodesgl_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeVisitor);
void zdbsp_processor_nodesx_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeExVisitor);
void zdbsp_processor_nodesx_v5_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeExOVisitor);

void zdbsp_processor_segs_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegVisitor);
void zdbsp_processor_segsgl_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegGlVisitor);
void zdbsp_processor_segsglx_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegGlExVisitor);
void zdbsp_processor_segsglx_v5_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegGlExVisitor);

void zdbsp_processor_ssectors_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorVisitor);
void zdbsp_processor_ssectorsgl_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorVisitor);
void zdbsp_processor_ssectorsx_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorExVisitor);
void zdbsp_processor_ssectorsx_v5_foreach(
	zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorExVisitor
);

void zdbsp_processor_vertsx_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_VertexExVisitor);

#undef nodiscard

#ifdef __cplusplus
}
#endif // __cplusplus
