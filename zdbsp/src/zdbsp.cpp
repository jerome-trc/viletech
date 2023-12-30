/// @file
/// @brief Bridge between the C interface and the internal C++.

#include "common.hpp"
#include "processor.hpp"
#include "zdbsp.h"

// Upstream ZDBSP aliased `DWORD` to `unsigned long` on win32 and `uint32_t`
// everywhere else. These check that using fixed-width types everywhere is sound.
static_assert(sizeof(unsigned char) == sizeof(uint8_t));
static_assert(sizeof(unsigned short) == sizeof(uint16_t));
static_assert(sizeof(unsigned int) == sizeof(uint32_t));

static void processor_nodes_foreach(void*, zdbsp_NodeVisitor, const zdbsp_NodeEx*, size_t);
static void processor_ssectors_foreach(
	void*, zdbsp_SubsectorVisitor, const zdbsp_SubsectorEx*, size_t
);
static void processor_ssectorsx_foreach(
	void*, zdbsp_SubsectorExVisitor, const zdbsp_SubsectorEx*, size_t
);
static void processor_verticesx_foreach(
	void*, zdbsp_VertexExVisitor, const zdbsp_VertexEx*, size_t
);

zdbsp_ProcessFlags zdbsp_processflags_default(void) {
	return zdbsp_ProcessFlags(ZDBSP_PROCF_BUILDNODES | ZDBSP_PROCF_CHECKPOLYOBJS);
}

zdbsp_RejectMode zdbsp_rejectmode_default(void) {
	return ZDBSP_ERM_DONTTOUCH;
}

zdbsp_BlockmapMode zdbsp_blockmapmode_default(void) {
	return ZDBSP_EBM_REBUILD;
}

void zdbsp_pcfg_extended(zdbsp_ProcessConfig* pcfg) {
	int32_t i = pcfg->flags;
	i |= ZDBSP_PROCF_COMPRESSNODES;
	i |= ZDBSP_PROCF_COMPRESSGLNODES;
	i &= ~ZDBSP_PROCF_FORCECOMPRESSION;
	pcfg->flags = static_cast<zdbsp_ProcessFlags>(i);
}

zdbsp_ProcessorPtr zdbsp_processor_new_vanilla(zdbsp_Level level) {
	return std::make_unique<FProcessor>(
		level, false
	).release();
}

zdbsp_ProcessorPtr zdbsp_processor_new_extended(zdbsp_Level level) {
	return std::make_unique<FProcessor>(
		level, true
	).release();
}

zdbsp_ProcessorPtr zdbsp_processor_new_udmf(zdbsp_LevelUdmf level) {
	return std::make_unique<FProcessor>(level).release();
}

void zdbsp_processor_configure(zdbsp_ProcessorPtr p, const zdbsp_ProcessConfig *config) {
	if (config != nullptr) {
		p->blockmap_mode = config->blockmap_mode;
		p->reject_mode = config->reject_mode;

		p->build_nodes = (config->flags & ZDBSP_PROCF_BUILDNODES);
		p->build_gl_nodes = (config->flags & ZDBSP_PROCF_BUILDGLNODES);
		p->check_poly_objs = (config->flags & ZDBSP_PROCF_CHECKPOLYOBJS);
		p->compress_nodes = (config->flags & ZDBSP_PROCF_COMPRESSNODES);
		p->compress_gl_nodes = (config->flags & ZDBSP_PROCF_COMPRESSGLNODES);
		p->conform_nodes = (config->flags & ZDBSP_PROCF_CONFORMNODES);
		p->force_compression = (config->flags & ZDBSP_PROCF_FORCECOMPRESSION);
		p->gl_only = (config->flags & ZDBSP_PROCF_GLONLY);
		p->no_prune = (config->flags & ZDBSP_PROCF_NOPRUNE);
		p->v5gl = (config->flags & ZDBSP_PROCF_V5GL);
		p->write_comments = (config->flags & ZDBSP_PROCF_WRITECOMMENTS);
	}

	if (p->conform_nodes || p->v5gl) {
		p->build_gl_nodes = true;
	}

	if (p->gl_only) {
		p->conform_nodes = false;
	}
}

void zdbsp_processor_destroy(zdbsp_ProcessorPtr p) {
	auto _ = std::unique_ptr<FProcessor>(p);
}

void zdbsp_processor_run(zdbsp_ProcessorPtr p, const zdbsp_NodeConfig* const config) {
	p->Process(config);
}

zdbsp_NodeVersion zdbsp_processor_nodeversion(const zdbsp_ProcessorPtr p) {
	return p->get_node_version();
}

const char* zdbsp_processor_magicnumber(const zdbsp_ProcessorPtr p, zdbsp_Bool compress) {
	switch (p->get_node_version()) {
	case ZDBSP_NODEVERS_1:
		if (compress) {
			return "ZGLN";
		} else {
			return "XGLN";
		}
	case ZDBSP_NODEVERS_2:
		if (compress) {
			return "ZGL2";
		} else {
			return "XGL2";
		}
	case ZDBSP_NODEVERS_3:
		if (compress) {
			return "ZGL3";
		} else {
			return "XGL3";
		}
	default:
		if (compress) {
			return "ZNOD";
		} else {
			return NULL;
		}
	}
}

size_t zdbsp_processor_nodes_count(const zdbsp_ProcessorPtr p) {
	return p->get_level().NumNodes;
}

size_t zdbsp_processor_nodesgl_count(const zdbsp_ProcessorPtr p) {
	return p->get_level().NumGLNodes;
}

size_t zdbsp_processor_segs_count(const zdbsp_ProcessorPtr p) {
	return p->get_level().NumSegs;
}

size_t zdbsp_processor_segsglx_count(const zdbsp_ProcessorPtr p) {
	return p->get_level().NumGLSegs;
}

size_t zdbsp_processor_ssectors_count(const zdbsp_ProcessorPtr p) {
	return p->get_level().NumSubsectors;
}

size_t zdbsp_processor_ssectorsgl_count(const zdbsp_ProcessorPtr p) {
	return p->get_level().NumGLSubsectors;
}

size_t zdbsp_processor_vertsorig_count(const zdbsp_ProcessorPtr p) {
	return p->get_level().NumOrgVerts;
}

size_t zdbsp_processor_vertsgl_count(const zdbsp_ProcessorPtr p) {
	return p->get_level().NumGLVertices;
}

size_t zdbsp_processor_vertsnewx_count(const zdbsp_ProcessorPtr p) {
	auto& level = p->get_level();
	return level.NumVertices - level.NumOrgVerts;
}

size_t zdbsp_processor_vertsnewgl_count(const zdbsp_ProcessorPtr p) {
	auto& level = p->get_level();
	return level.NumGLVertices - level.NumOrgVerts;
}

zdbsp_SliceU16 zdbsp_processor_blockmap(const zdbsp_ProcessorPtr p) {
	zdbsp_SliceU16 slice = {};
	slice.ptr = p->get_level().Blockmap;
	slice.len = static_cast<size_t>(p->get_level().BlockmapSize);
	return slice;
}

// Node iterators //////////////////////////////////////////////////////////////

void zdbsp_processor_nodes_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeVisitor callback
) {
	auto& level = p->get_level();
	processor_nodes_foreach(ctx, callback, level.Nodes, level.NumNodes);
}

void zdbsp_processor_nodesx_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumNodes; ++i) {
		callback(ctx, &level.Nodes[i]);
	}
}

void zdbsp_processor_nodesgl_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeVisitor callback
) {
	auto& level = p->get_level();
	processor_nodes_foreach(ctx, callback, level.GLNodes, level.NumGLNodes);
}

void zdbsp_processor_nodesglx_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLNodes; ++i) {
		callback(ctx, &level.GLNodes[i]);
	}
}

void zdbsp_processor_nodesx_v5_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeExOVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLNodes; ++i) {
		const auto& n = level.GLNodes[i];
		zdbsp_NodeExO node = {};

		const short* inodes = &n.bbox[0][0];
		short* coord = &node.bbox[0][0];

		for (size_t j = 0; j < 2 * 4; ++j) {
			coord[j] = LittleShort(inodes[j]);
		}

		node.x = LittleShort(n.x >> 16);
		node.y = LittleShort(n.y >> 16);
		node.dx = LittleShort(n.dx >> 16);
		node.dy = LittleShort(n.dy >> 16);

		for (size_t j = 0; j < 2; ++j) {
			node.children[j] = LittleLong(n.children[j]);
		}

		callback(ctx, &node);
	}
}

// Seg iterators ///////////////////////////////////////////////////////////////

void zdbsp_processor_segs_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumSegs; ++i) {
		const auto& s = level.Segs[i];
		zdbsp_SegRaw seg = {};

		seg.v1 = LittleShort(uint16_t(s.v1));
		seg.v2 = LittleShort(uint16_t(s.v2));
		seg.angle = LittleShort(s.angle);
		seg.linedef = LittleShort(s.linedef);
		seg.side = LittleShort(s.side);
		seg.offset = LittleShort(s.offset);

		callback(ctx, &seg);
	}
}

void zdbsp_processor_segsx_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumSegs; ++i) {
		callback(ctx, &level.Segs[i]);
	}
}

void zdbsp_processor_segsgl_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegGlVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLSegs; ++i) {
		const auto& s = level.GLSegs[i];
		zdbsp_SegGl seg = {};

		if (s.v1 < (uint32_t)level.NumOrgVerts) {
			seg.v1 = LittleShort((uint16_t)s.v1);
		} else {
			seg.v1 = LittleShort(0x8000 | (uint16_t)(s.v1 - level.NumOrgVerts));
		}

		if (s.v2 < (uint32_t)level.NumOrgVerts) {
			seg.v2 = (uint16_t)LittleShort(s.v2);
		} else {
			seg.v2 = LittleShort(0x8000 | (uint16_t)(s.v2 - level.NumOrgVerts));
		}

		seg.linedef = LittleShort((uint16_t)s.linedef);
		seg.side = LittleShort(s.side);
		seg.partner = LittleShort((uint16_t)s.partner);

		callback(ctx, &seg);
	}
}

void zdbsp_processor_segsglx_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegGlExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLSegs; ++i) {
		callback(ctx, &level.GLSegs[i]);
	}
}

void zdbsp_processor_segsglx_v5_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegGlExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLSegs; ++i) {
		const auto& s = level.GLSegs[i];
		zdbsp_SegGlEx seg = {};

		if (s.v1 < (uint32_t)level.NumOrgVerts) {
			seg.v1 = LittleLong(s.v1);
		} else {
			seg.v1 = LittleLong(0x80000000u | ((int32_t)s.v1 - level.NumOrgVerts));
		}

		if (s.v2 < (uint32_t)level.NumOrgVerts) {
			seg.v2 = LittleLong(s.v2);
		} else {
			seg.v2 = LittleLong(0x80000000u | ((int32_t)s.v2 - level.NumOrgVerts));
		}

		seg.linedef = LittleShort(s.linedef);
		seg.side = LittleShort(s.side);
		seg.partner = LittleShort(s.partner);

		callback(ctx, &seg);
	}
}

// Subsector iterators /////////////////////////////////////////////////////////

void zdbsp_processor_ssectors_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorVisitor callback
) {
	auto& level = p->get_level();
	processor_ssectors_foreach(ctx, callback, level.Subsectors, level.NumSubsectors);
}

void zdbsp_processor_ssectorsgl_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorVisitor callback
) {
	auto& level = p->get_level();
	processor_ssectors_foreach(ctx, callback, level.GLSubsectors, level.NumGLSubsectors);
}

void zdbsp_processor_ssectorsx_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorExVisitor callback
) {
	auto& level = p->get_level();
	processor_ssectorsx_foreach(ctx, callback, level.Subsectors, level.NumSubsectors);
}

void zdbsp_processor_ssectorsglx_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorExVisitor callback
) {
	auto& level = p->get_level();
	processor_ssectorsx_foreach(ctx, callback, level.GLSubsectors, level.NumGLSubsectors);
}

void zdbsp_processor_ssectorsx_v5_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLSubsectors; ++i) {
		zdbsp_SubsectorEx ss = {};
		ss.first_line = LittleLong(level.GLSubsectors[i].first_line);
		ss.num_lines = LittleLong(level.GLSubsectors[i].num_lines);
		callback(ctx, &ss);
	}
}

// Vertex iterators ////////////////////////////////////////////////////////////

void zdbsp_processor_vertsx_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_VertexExVisitor callback
) {
	auto& level = p->get_level();

	processor_verticesx_foreach(
		ctx, callback, &level.Vertices[level.NumOrgVerts], level.NumVertices - level.NumOrgVerts
	);
}

void zdbsp_processor_vertsgl_foreach(
	const zdbsp_ProcessorPtr p, void* ctx, zdbsp_VertexExVisitor callback
) {
	auto& level = p->get_level();

	processor_verticesx_foreach(
		ctx, callback, &level.GLVertices[level.NumOrgVerts], level.NumGLVertices - level.NumOrgVerts
	);
}

// Details /////////////////////////////////////////////////////////////////////

static void processor_nodes_foreach(
	void* const ctx,
	const zdbsp_NodeVisitor callback,
	const zdbsp_NodeEx* const node_array,
	const size_t node_count
) {
	for (size_t i = 0; i < node_count; ++i) {
		auto& n = node_array[i];

		zdbsp_NodeRaw node = {};
		node.x = LittleShort(n.x >> 16);
		node.y = LittleShort(n.y >> 16);
		node.dx = LittleShort(n.dx >> 16);
		node.dy = LittleShort(n.dy >> 16);

		for (size_t ii = 0; ii < 2; ++ii) {
			for (size_t iii = 0; iii < 4; ++iii) {
				node.bbox[ii][iii] = LittleShort(n.bbox[ii][iii]);
			}
		}

		auto o = reinterpret_cast<int16_t*>(&node.children[0]);

		for (size_t ii = 0; ii < 2; ++ii) {
			uint32_t child = n.children[ii];

			if (child & NFX_SUBSECTOR) {
				*o++ = LittleShort(uint16_t(child - (NFX_SUBSECTOR + NF_SUBSECTOR)));
			} else {
				*o++ = LittleShort((uint16_t)child);
			}
		}

		callback(ctx, &node);
	}
}

static void processor_ssectorsx_foreach(
	void* const ctx,
	const zdbsp_SubsectorExVisitor callback,
	const zdbsp_SubsectorEx* const array,
	const size_t count
) {
	for (size_t i = 0; i < count; ++i) {
		callback(ctx, &array[i]);
	}
}

static void processor_ssectors_foreach(
	void* const ctx,
	const zdbsp_SubsectorVisitor callback,
	const zdbsp_SubsectorEx* const array,
	const size_t count
) {
	for (size_t i = 0; i < count; ++i) {
		const auto& ss = array[i];
		zdbsp_SubsectorRaw subsect = {};

		subsect.first_line = LittleShort((uint16_t)ss.first_line);
		subsect.num_lines = LittleShort((uint16_t)ss.num_lines);

		callback(ctx, &subsect);
	}
}

static void processor_verticesx_foreach(
	void* const ctx,
	const zdbsp_VertexExVisitor callback,
	const zdbsp_VertexEx* const array,
	const size_t count
) {
	for (size_t i = 0; i < count; ++i) {
		callback(ctx, &array[i]);
	}
}
