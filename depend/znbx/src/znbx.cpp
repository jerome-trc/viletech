/// @file
/// @brief Bridge between the C interface and the internal C++.

#include "common.hpp"
#include "processor.hpp"
#include "znbx.h"

// Upstream ZDBSP aliased `DWORD` to `unsigned long` on win32 and `uint32_t`
// everywhere else. These check that using fixed-width types everywhere is sound.
static_assert(sizeof(unsigned char) == sizeof(uint8_t));
static_assert(sizeof(unsigned short) == sizeof(uint16_t));
static_assert(sizeof(unsigned int) == sizeof(uint32_t));

static void processor_nodes_foreach(void*, znbx_NodeVisitor, const znbx_NodeEx*, size_t);
static void processor_ssectors_foreach(
	void*, znbx_SubsectorVisitor, const znbx_SubsectorEx*, size_t
);
static void processor_ssectorsx_foreach(
	void*, znbx_SubsectorExVisitor, const znbx_SubsectorEx*, size_t
);
static void processor_verticesx_foreach(
	void*, znbx_VertexExVisitor, const znbx_VertexEx*, size_t
);

znbx_ProcessFlags znbx_processflags_default(void) {
	return znbx_ProcessFlags(ZNBX_PROCF_BUILDNODES | ZNBX_PROCF_CHECKPOLYOBJS);
}

znbx_RejectMode znbx_rejectmode_default(void) {
	return ZNBX_ERM_DONTTOUCH;
}

znbx_BlockmapMode znbx_blockmapmode_default(void) {
	return ZNBX_EBM_REBUILD;
}

void znbx_pcfg_extended(znbx_ProcessConfig* pcfg) {
	int32_t i = pcfg->flags;
	i |= ZNBX_PROCF_COMPRESSNODES;
	i |= ZNBX_PROCF_COMPRESSGLNODES;
	i &= ~ZNBX_PROCF_FORCECOMPRESSION;
	pcfg->flags = static_cast<znbx_ProcessFlags>(i);
}

znbx_ProcessorPtr znbx_processor_new_vanilla(znbx_Level level) {
	return std::make_unique<FProcessor>(
		level, false
	).release();
}

znbx_ProcessorPtr znbx_processor_new_extended(znbx_Level level) {
	return std::make_unique<FProcessor>(
		level, true
	).release();
}

znbx_ProcessorPtr znbx_processor_new_udmf(znbx_LevelUdmf level) {
	return std::make_unique<FProcessor>(level).release();
}

void znbx_processor_configure(znbx_ProcessorPtr p, const znbx_ProcessConfig *config) {
	if (config != nullptr) {
		p->blockmap_mode = config->blockmap_mode;
		p->reject_mode = config->reject_mode;

		p->build_nodes = (config->flags & ZNBX_PROCF_BUILDNODES);
		p->build_gl_nodes = (config->flags & ZNBX_PROCF_BUILDGLNODES);
		p->check_poly_objs = (config->flags & ZNBX_PROCF_CHECKPOLYOBJS);
		p->compress_nodes = (config->flags & ZNBX_PROCF_COMPRESSNODES);
		p->compress_gl_nodes = (config->flags & ZNBX_PROCF_COMPRESSGLNODES);
		p->conform_nodes = (config->flags & ZNBX_PROCF_CONFORMNODES);
		p->force_compression = (config->flags & ZNBX_PROCF_FORCECOMPRESSION);
		p->gl_only = (config->flags & ZNBX_PROCF_GLONLY);
		p->no_prune = (config->flags & ZNBX_PROCF_NOPRUNE);
		p->v5gl = (config->flags & ZNBX_PROCF_V5GL);
		p->write_comments = (config->flags & ZNBX_PROCF_WRITECOMMENTS);
	}

	if (p->conform_nodes || p->v5gl) {
		p->build_gl_nodes = true;
	}

	if (p->gl_only) {
		p->conform_nodes = false;
	}
}

void znbx_processor_destroy(znbx_ProcessorPtr p) {
	auto _ = std::unique_ptr<FProcessor>(p);
}

void znbx_processor_run(znbx_ProcessorPtr p, const znbx_NodeConfig* const config) {
	p->Process(config);
}

znbx_NodeVersion znbx_processor_nodeversion(const znbx_ProcessorPtr p) {
	return p->get_node_version();
}

const char* znbx_processor_magicnumber(const znbx_ProcessorPtr p, znbx_Bool compress) {
	switch (p->get_node_version()) {
	case ZNBX_NODEVERS_1:
		if (compress) {
			return "ZGLN";
		} else {
			return "XGLN";
		}
	case ZNBX_NODEVERS_2:
		if (compress) {
			return "ZGL2";
		} else {
			return "XGL2";
		}
	case ZNBX_NODEVERS_3:
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

size_t znbx_processor_nodes_count(const znbx_ProcessorPtr p) {
	return p->get_level().NumNodes;
}

size_t znbx_processor_nodesgl_count(const znbx_ProcessorPtr p) {
	return p->get_level().NumGLNodes;
}

size_t znbx_processor_segs_count(const znbx_ProcessorPtr p) {
	return p->get_level().NumSegs;
}

size_t znbx_processor_segsglx_count(const znbx_ProcessorPtr p) {
	return p->get_level().NumGLSegs;
}

size_t znbx_processor_ssectors_count(const znbx_ProcessorPtr p) {
	return p->get_level().NumSubsectors;
}

size_t znbx_processor_ssectorsgl_count(const znbx_ProcessorPtr p) {
	return p->get_level().NumGLSubsectors;
}

size_t znbx_processor_vertsorig_count(const znbx_ProcessorPtr p) {
	return p->get_level().NumOrgVerts;
}

size_t znbx_processor_vertsgl_count(const znbx_ProcessorPtr p) {
	return p->get_level().NumGLVertices;
}

size_t znbx_processor_vertsnewx_count(const znbx_ProcessorPtr p) {
	auto& level = p->get_level();
	return level.NumVertices - level.NumOrgVerts;
}

size_t znbx_processor_vertsnewgl_count(const znbx_ProcessorPtr p) {
	auto& level = p->get_level();
	return level.NumGLVertices - level.NumOrgVerts;
}

znbx_SliceU16 znbx_processor_blockmap(const znbx_ProcessorPtr p) {
	znbx_SliceU16 slice = {};
	slice.ptr = p->get_level().Blockmap;
	slice.len = static_cast<size_t>(p->get_level().BlockmapSize);
	return slice;
}

// Node iterators //////////////////////////////////////////////////////////////

void znbx_processor_nodes_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_NodeVisitor callback
) {
	auto& level = p->get_level();
	processor_nodes_foreach(ctx, callback, level.Nodes, level.NumNodes);
}

void znbx_processor_nodesx_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_NodeExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumNodes; ++i) {
		callback(ctx, &level.Nodes[i]);
	}
}

void znbx_processor_nodesgl_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_NodeVisitor callback
) {
	auto& level = p->get_level();
	processor_nodes_foreach(ctx, callback, level.GLNodes, level.NumGLNodes);
}

void znbx_processor_nodesglx_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_NodeExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLNodes; ++i) {
		callback(ctx, &level.GLNodes[i]);
	}
}

void znbx_processor_nodesx_v5_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_NodeExOVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLNodes; ++i) {
		const auto& n = level.GLNodes[i];
		znbx_NodeExO node = {};

		const short* inodes = &n.bbox[0][0];
		short* coord = &node.bbox[0][0];

		for (size_t j = 0; j < 2 * 4; ++j) {
			coord[j] = little_short(inodes[j]);
		}

		node.x = little_short(n.x >> 16);
		node.y = little_short(n.y >> 16);
		node.dx = little_short(n.dx >> 16);
		node.dy = little_short(n.dy >> 16);

		for (size_t j = 0; j < 2; ++j) {
			node.children[j] = little_long(n.children[j]);
		}

		callback(ctx, &node);
	}
}

// Seg iterators ///////////////////////////////////////////////////////////////

void znbx_processor_segs_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SegVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumSegs; ++i) {
		const auto& s = level.Segs[i];
		znbx_SegRaw seg = {};

		seg.v1 = little_short(uint16_t(s.v1));
		seg.v2 = little_short(uint16_t(s.v2));
		seg.angle = little_short(s.angle);
		seg.linedef = little_short(s.linedef);
		seg.side = little_short(s.side);
		seg.offset = little_short(s.offset);

		callback(ctx, &seg);
	}
}

void znbx_processor_segsx_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SegExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumSegs; ++i) {
		callback(ctx, &level.Segs[i]);
	}
}

void znbx_processor_segsgl_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SegGlVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLSegs; ++i) {
		const auto& s = level.GLSegs[i];
		znbx_SegGl seg = {};

		if (s.v1 < (uint32_t)level.NumOrgVerts) {
			seg.v1 = little_short((uint16_t)s.v1);
		} else {
			seg.v1 = little_short(0x8000 | (uint16_t)(s.v1 - level.NumOrgVerts));
		}

		if (s.v2 < (uint32_t)level.NumOrgVerts) {
			seg.v2 = (uint16_t)little_short(s.v2);
		} else {
			seg.v2 = little_short(0x8000 | (uint16_t)(s.v2 - level.NumOrgVerts));
		}

		seg.linedef = little_short((uint16_t)s.linedef);
		seg.side = little_short(s.side);
		seg.partner = little_short((uint16_t)s.partner);

		callback(ctx, &seg);
	}
}

void znbx_processor_segsglx_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SegGlExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLSegs; ++i) {
		callback(ctx, &level.GLSegs[i]);
	}
}

void znbx_processor_segsglx_v5_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SegGlExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLSegs; ++i) {
		const auto& s = level.GLSegs[i];
		znbx_SegGlEx seg = {};

		if (s.v1 < (uint32_t)level.NumOrgVerts) {
			seg.v1 = little_long(s.v1);
		} else {
			seg.v1 = little_long(0x80000000u | ((int32_t)s.v1 - level.NumOrgVerts));
		}

		if (s.v2 < (uint32_t)level.NumOrgVerts) {
			seg.v2 = little_long(s.v2);
		} else {
			seg.v2 = little_long(0x80000000u | ((int32_t)s.v2 - level.NumOrgVerts));
		}

		seg.linedef = little_short(s.linedef);
		seg.side = little_short(s.side);
		seg.partner = little_short(s.partner);

		callback(ctx, &seg);
	}
}

// Subsector iterators /////////////////////////////////////////////////////////

void znbx_processor_ssectors_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SubsectorVisitor callback
) {
	auto& level = p->get_level();
	processor_ssectors_foreach(ctx, callback, level.Subsectors, level.NumSubsectors);
}

void znbx_processor_ssectorsgl_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SubsectorVisitor callback
) {
	auto& level = p->get_level();
	processor_ssectors_foreach(ctx, callback, level.GLSubsectors, level.NumGLSubsectors);
}

void znbx_processor_ssectorsx_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SubsectorExVisitor callback
) {
	auto& level = p->get_level();
	processor_ssectorsx_foreach(ctx, callback, level.Subsectors, level.NumSubsectors);
}

void znbx_processor_ssectorsglx_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SubsectorExVisitor callback
) {
	auto& level = p->get_level();
	processor_ssectorsx_foreach(ctx, callback, level.GLSubsectors, level.NumGLSubsectors);
}

void znbx_processor_ssectorsx_v5_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_SubsectorExVisitor callback
) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLSubsectors; ++i) {
		znbx_SubsectorEx ss = {};
		ss.first_line = little_long(level.GLSubsectors[i].first_line);
		ss.num_lines = little_long(level.GLSubsectors[i].num_lines);
		callback(ctx, &ss);
	}
}

// Vertex iterators ////////////////////////////////////////////////////////////

void znbx_processor_vertsx_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_VertexExVisitor callback
) {
	auto& level = p->get_level();

	processor_verticesx_foreach(
		ctx, callback, &level.Vertices[level.NumOrgVerts], level.NumVertices - level.NumOrgVerts
	);
}

void znbx_processor_vertsgl_foreach(
	const znbx_ProcessorPtr p, void* ctx, znbx_VertexExVisitor callback
) {
	auto& level = p->get_level();

	processor_verticesx_foreach(
		ctx, callback, &level.GLVertices[level.NumOrgVerts], level.NumGLVertices - level.NumOrgVerts
	);
}

// Details /////////////////////////////////////////////////////////////////////

static void processor_nodes_foreach(
	void* const ctx,
	const znbx_NodeVisitor callback,
	const znbx_NodeEx* const node_array,
	const size_t node_count
) {
	for (size_t i = 0; i < node_count; ++i) {
		auto& n = node_array[i];

		znbx_NodeRaw node = {};
		node.x = little_short(n.x >> 16);
		node.y = little_short(n.y >> 16);
		node.dx = little_short(n.dx >> 16);
		node.dy = little_short(n.dy >> 16);

		for (size_t ii = 0; ii < 2; ++ii) {
			for (size_t iii = 0; iii < 4; ++iii) {
				node.bbox[ii][iii] = little_short(n.bbox[ii][iii]);
			}
		}

		auto o = reinterpret_cast<int16_t*>(&node.children[0]);

		for (size_t ii = 0; ii < 2; ++ii) {
			uint32_t child = n.children[ii];

			if (child & NFX_SUBSECTOR) {
				*o++ = little_short(uint16_t(child - (NFX_SUBSECTOR + NF_SUBSECTOR)));
			} else {
				*o++ = little_short((uint16_t)child);
			}
		}

		callback(ctx, &node);
	}
}

static void processor_ssectorsx_foreach(
	void* const ctx,
	const znbx_SubsectorExVisitor callback,
	const znbx_SubsectorEx* const array,
	const size_t count
) {
	for (size_t i = 0; i < count; ++i) {
		callback(ctx, &array[i]);
	}
}

static void processor_ssectors_foreach(
	void* const ctx,
	const znbx_SubsectorVisitor callback,
	const znbx_SubsectorEx* const array,
	const size_t count
) {
	for (size_t i = 0; i < count; ++i) {
		const auto& ss = array[i];
		znbx_SubsectorRaw subsect = {};

		subsect.first_line = little_short((uint16_t)ss.first_line);
		subsect.num_lines = little_short((uint16_t)ss.num_lines);

		callback(ctx, &subsect);
	}
}

static void processor_verticesx_foreach(
	void* const ctx,
	const znbx_VertexExVisitor callback,
	const znbx_VertexEx* const array,
	const size_t count
) {
	for (size_t i = 0; i < count; ++i) {
		callback(ctx, &array[i]);
	}
}
