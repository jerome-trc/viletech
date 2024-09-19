/// @file
/// @brief ZNBX's entire publicly-exported C interface.

#pragma once

#include <stddef.h>
#include <stdint.h>

#if __STDC_VERSION__ >= 202000L || (defined(__cplusplus) && __cplusplus >= 201703L)
#define ZNBX_NODISCARD [[nodiscard]]
#else
#define ZNBX_NODISCARD
#endif

#ifdef __cplusplus
extern "C" {
#endif

#define ZNBX_VERSION "1.19"

/// A 32-bit fixed-point decimal type,
/// comprising a 16-bit integral component and a 16-bit fractional component.
typedef int32_t znbx_I16F16;
typedef uint32_t znbx_Angle;
typedef uint8_t znbx_Bool;

/// If no configuration is provided, `ZNBX_EBM_REBUILD` is the default.
///
/// @see znbx_ProcessConfig
typedef enum {
	ZNBX_EBM_REBUILD,
	ZNBX_EBM_CREATE0,
} znbx_BlockmapMode;

/// If no configuration is provided, `ZNBX_ERM_DONTTOUCH` is the default.
///
/// @see znbx_ProcessConfig
typedef enum {
	ZNBX_ERM_DONTTOUCH,
	ZNBX_ERM_CREATEZEROES,
	ZNBX_ERM_CREATE0,
	ZNBX_ERM_REBUILD
} znbx_RejectMode;

/// @see znbx_ProcessConfig
typedef enum {
	/// Enabled by default.
	ZNBX_PROCF_BUILDNODES = 1 << 0,
	/// Disabled by default. Implies #ZNBX_PROCF_BUILDGLNODES.
	///
	/// "Conforming" GL nodes are those which use the same
	/// basic information as non-GL nodes. This results in sub-optimal non-GL nodes
	/// but makes it easier to compare the two sets of nodes to verify the correctness
	/// of the GL nodes.
	ZNBX_PROCF_CONFORMNODES = 1 << 2,
	/// Disabled by default. "Pruning" is the process by which the node builder:
	/// - removes 0-length lines
	/// - removes sides not referenced by any lines
	/// - removes sectors not referenced by any sides
	/// 0-length line removal cannot be disabled, but setting this flag prevents
	/// removal of extraneous sides and sectors.
	ZNBX_PROCF_NOPRUNE = 1 << 3,
	/// Enabled by default.
	ZNBX_PROCF_CHECKPOLYOBJS = 1 << 4,

	/// Disabled by default.
	ZNBX_PROCF_BUILDGLNODES = 1 << 5,
	/// Disabled by default. Implies #ZNBX_PROCF_BUILDGLNODES.
	ZNBX_PROCF_GLONLY = 1 << 6,
	/// Disabled by default. Implies #ZNBX_PROCF_BUILDGLNODES.
	///
	/// Note that this will be forced anyway if one of the
	/// following conditions is met during a processor run:
	/// - there is a combined total of more than 32767 vertices between
	/// raw input and generated GL vertices
	/// - more than 65534 GL segs get built
	/// - more than 32767 GL nodes get built
	/// - more than 32767 GL subsectors get built
	ZNBX_PROCF_V5GL = 1 << 7,
	/// Disabled by default.
	ZNBX_PROCF_WRITECOMMENTS = 1 << 8,

	/// Disabled by default. Note that this will be forced anyway if one of the
	/// following conditions is met during a processor run:
	/// - the processor determines that GL nodes need to be compressed
	/// - there is a combined total of more than 65535 vertices between
	/// raw input and generated GL vertices
	/// - more than 65535 segs get built
	/// - more than 32767 subsectors get built
	/// - more than 32767 nodes get built
	///
	/// @see ZNBX_PROCF_COMPRESSGLNODES
	ZNBX_PROCF_COMPRESSNODES = 1 << 9,
	/// Disabled by default. Note that this will be forced anyway if one of the
	/// following conditions is met during a processor run:
	/// - there is a combined total of more than 32767 vertices between
	/// raw input and generated GL vertices
	ZNBX_PROCF_COMPRESSGLNODES = 1 << 10,
	/// Disabled by default.
	ZNBX_PROCF_FORCECOMPRESSION = 1 << 11,
} znbx_ProcessFlags;

typedef struct {
	znbx_ProcessFlags flags;
	znbx_RejectMode reject_mode;
	znbx_BlockmapMode blockmap_mode;
} znbx_ProcessConfig;

typedef struct {
	/// The cost for avoiding diagonal splitters (16 by default).
	/// Any value lower than 1 will get forced back up to 1 internally.
	int32_t aa_preference;
	/// The maximum number of segs to consider at each node (64 by default).
	/// Any value lower than 3 will get forced back up to 3 internally.
	int32_t max_segs;
	/// The cost to split a seg (8 by default).
	/// Any value lower than 1 will get forced back up to 1 internally.
	int32_t split_cost;
} znbx_NodeConfig;

/// A level vertex as per the original WAD format.
typedef struct {
	int16_t x, y;
} znbx_VertexRaw;

/// A level vertex in terms of 32-bit fixed-point numbers.
typedef struct {
	znbx_I16F16 x, y;
} znbx_VertexFxp;

typedef struct {
	znbx_I16F16 x, y;
	int32_t index;
} znbx_VertexEx;

/// A binary space partition tree node as per the original WAD format.
typedef struct {
	int16_t x, y, dx, dy;
	int16_t bbox[2][4];
	uint16_t children[2];
} znbx_NodeRaw;

typedef struct {
	int32_t x, y, dx, dy;
	int16_t bbox[2][4];
	uint32_t children[2];
} znbx_NodeEx;

typedef struct {
	int16_t x, y, dx, dy;
	int16_t bbox[2][4];
	uint32_t children[2];
} znbx_NodeExO;

/// A binary space partition tree node in terms of 32-bit fixed-point numbers.
typedef struct {
	znbx_I16F16 x, y, dx, dy;
	znbx_I16F16 bbox[2][4];
	uint32_t int_children[2];
} znbx_NodeFxp;

typedef struct {
	uint16_t v1, v2;
	uint16_t angle, linedef;
	int16_t side, offset;
} znbx_SegRaw;

typedef struct {
	uint32_t v1, v2;
	uint16_t angle, linedef;
	int16_t side, offset;
} znbx_SegEx;

typedef struct {
	uint16_t v1, v2;
	uint16_t linedef, side, partner;
} znbx_SegGl;

typedef struct {
	uint32_t v1, v2;
	uint32_t linedef;
	uint16_t side;
	uint32_t partner;
} znbx_SegGlEx;

typedef struct {
	int16_t x, y;
	int16_t angle;
	int16_t type;
	int16_t flags;
} znbx_ThingRaw;

typedef struct {
	uint16_t thing_id;
	int16_t x, y, z;
	int16_t angle;
	int16_t type;
	int16_t flags;
	int8_t special;
	int8_t args[5];
} znbx_Thing2;

typedef struct {
	uint16_t num_lines;
	uint16_t first_line;
} znbx_SubsectorRaw;

typedef struct {
	uint32_t num_lines;
	uint32_t first_line;
} znbx_SubsectorEx;

typedef struct {
	const char *key, *value;
} znbx_UdmfKey;

typedef enum {
	ZNBX_NODEVERS_UNKNOWN,
	ZNBX_NODEVERS_1,
	ZNBX_NODEVERS_2,
	ZNBX_NODEVERS_3,
} znbx_NodeVersion;

/// If a processor reports a version below `ZNBX_NODEVERS_2`,
/// you should be serializing these to WAD entries.
typedef struct {
	uint32_t v1;
	uint32_t partner;
	uint16_t linedef;
	uint8_t side;
} znbx_SegGlXV1;

/// If a processor reports a version at or above `ZNBX_NODEVERS_2`,
/// you should be serializing these to WAD entries.
typedef struct {
	uint32_t v1;
	uint32_t partner;
	uint32_t linedef;
	uint8_t side;
} znbx_SegGlXV2V3;

typedef struct {
	const uint8_t* ptr;
	size_t len;
} znbx_SliceU8;

/// @see znbx_processor_blockmap
typedef struct {
	const uint16_t* ptr;
	size_t len;
} znbx_SliceU16;

typedef struct {
	/// This is expected to be null-terminated.
	char name[9];
	znbx_SliceU8 things, vertices, linedefs, sidedefs, sectors;
} znbx_Level;

typedef struct {
	/// This is expected to be null-terminated.
	char name[9];
	znbx_SliceU8 textmap;
} znbx_LevelUdmf;

typedef struct FLevel* znbx_LevelPtr;
typedef struct FWadReader* znbx_WadReaderPtr;
typedef struct FProcessor* znbx_ProcessorPtr;

typedef void (*znbx_NodeVisitor)(void*, const znbx_NodeRaw*);
typedef void (*znbx_NodeExVisitor)(void*, const znbx_NodeEx*);
typedef void (*znbx_NodeExOVisitor)(void*, const znbx_NodeExO*);
typedef void (*znbx_SegVisitor)(void*, const znbx_SegRaw*);
typedef void (*znbx_SegExVisitor)(void*, const znbx_SegEx*);
typedef void (*znbx_SegGlVisitor)(void*, const znbx_SegGl*);
typedef void (*znbx_SegGlExVisitor)(void*, const znbx_SegGlEx*);
typedef void (*znbx_SubsectorVisitor)(void*, const znbx_SubsectorRaw*);
typedef void (*znbx_SubsectorExVisitor)(void*, const znbx_SubsectorEx*);
typedef void (*znbx_VertexExVisitor)(void*, const znbx_VertexEx*);

ZNBX_NODISCARD znbx_ProcessFlags znbx_processflags_default(void);
ZNBX_NODISCARD znbx_RejectMode znbx_rejectmode_default(void);
ZNBX_NODISCARD znbx_BlockmapMode znbx_blockmapmode_default(void);

void znbx_pcfg_extended(znbx_ProcessConfig*);

/// The returned object is owned by the caller, and should be freed using
/// `znbx_processor_destroy`.
/// Ownership of `level`'s bytes are not taken by this function; the caller
/// should free those bytes themselves.
ZNBX_NODISCARD znbx_ProcessorPtr znbx_processor_new_vanilla(znbx_Level level);

/// The returned object is owned by the caller, and should be freed using
/// `znbx_processor_destroy`.
/// Ownership of `level`'s bytes are not taken by this function; the caller
/// should free those bytes themselves.
ZNBX_NODISCARD znbx_ProcessorPtr znbx_processor_new_extended(znbx_Level level);

/// The returned object is owned by the caller, and should be freed using
/// `znbx_processor_destroy`.
/// Ownership of `level`'s bytes are not taken by this function; the caller
/// should free those bytes themselves.
ZNBX_NODISCARD znbx_ProcessorPtr znbx_processor_new_udmf(znbx_LevelUdmf level);

/// Calling with a `NULL` `config` is a valid no-op here.
void znbx_processor_configure(znbx_ProcessorPtr p, const znbx_ProcessConfig* config);

void znbx_processor_destroy(znbx_ProcessorPtr p);

/// Note that passing in a `NULL` `config` is valid here.
void znbx_processor_run(znbx_ProcessorPtr p, const znbx_NodeConfig* config);

/// Notes:
/// - If the processor has not been run yet, it will always return `ZNBX_NODEVERS_UNKNOWN`.
/// - It will also return `ZNBX_NODEVERS_UNKNOWN` if the last run did not build any GL nodes.
ZNBX_NODISCARD znbx_NodeVersion znbx_processor_nodeversion(const znbx_ProcessorPtr p);

/// Returns a (static) string containing the 4 bytes needed to be written into
/// a combined nodes/subsectors/segs WAD entry if possible.
///
/// Be aware that if the node version is unknown (i.e. a run has not yet been completed,
/// or no GL nodes were built in the last run) and `compress` is false then no
/// magic number is applicable, and this function will return a null pointer.
ZNBX_NODISCARD const char* znbx_processor_magicnumber(const znbx_ProcessorPtr p, znbx_Bool compress);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
ZNBX_NODISCARD size_t znbx_processor_nodes_count(const znbx_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
ZNBX_NODISCARD size_t znbx_processor_nodesgl_count(const znbx_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
ZNBX_NODISCARD size_t znbx_processor_segs_count(const znbx_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
ZNBX_NODISCARD size_t znbx_processor_segsglx_count(const znbx_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
ZNBX_NODISCARD size_t znbx_processor_ssectors_count(const znbx_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
ZNBX_NODISCARD size_t znbx_processor_ssectorsgl_count(const znbx_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
ZNBX_NODISCARD size_t znbx_processor_vertsorig_count(const znbx_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
ZNBX_NODISCARD size_t znbx_processor_vertsgl_count(const znbx_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
ZNBX_NODISCARD size_t znbx_processor_vertsnewx_count(const znbx_ProcessorPtr p);

/// Beware that if this number is going to be written to a WAD entry,
/// it should be serialized into a `uint32_t`.
ZNBX_NODISCARD size_t znbx_processor_vertsnewgl_count(const znbx_ProcessorPtr p);

ZNBX_NODISCARD znbx_SliceU16 znbx_processor_blockmap(const znbx_ProcessorPtr p);

void znbx_processor_nodes_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_NodeVisitor);
void znbx_processor_nodesx_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_NodeExVisitor);
void znbx_processor_nodesgl_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_NodeVisitor);
void znbx_processor_nodesglx_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_NodeExVisitor);
void znbx_processor_nodesx_v5_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_NodeExOVisitor);

void znbx_processor_segs_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_SegVisitor);
void znbx_processor_segsx_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_SegExVisitor);
void znbx_processor_segsgl_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_SegGlVisitor);
void znbx_processor_segsglx_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_SegGlExVisitor);
void znbx_processor_segsglx_v5_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SegGlExVisitor
);

void znbx_processor_ssectors_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SubsectorVisitor
);
void znbx_processor_ssectorsgl_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SubsectorVisitor
);
void znbx_processor_ssectorsx_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SubsectorExVisitor
);
void znbx_processor_ssectorsglx_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SubsectorExVisitor
);
void znbx_processor_ssectorsx_v5_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SubsectorExVisitor
);

void znbx_processor_vertsx_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_VertexExVisitor);
void znbx_processor_vertsgl_foreach(const znbx_ProcessorPtr p, void* ctx, znbx_VertexExVisitor);

#ifdef __cplusplus
}
#endif // __cplusplus
