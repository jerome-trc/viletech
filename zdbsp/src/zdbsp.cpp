/// @file
/// @brief Bridge between the C interface and the internal C++.

#include "zdbsp.h"

#include "common.hpp"
#include "processor.hpp"
#include "wad.hpp"

// Upstream ZDBSP aliased `DWORD` to `unsigned long` on win32 and `uint32_t`
// everywhere else. These check that using fixed-width types everywhere is sound.
static_assert(sizeof(unsigned char) == sizeof(uint8_t));
static_assert(sizeof(unsigned short) == sizeof(uint16_t));
static_assert(sizeof(unsigned int) == sizeof(uint32_t));

zdbsp_ProcessFlags zdbsp_processflags_default(void) {
	return zdbsp_ProcessFlags(ZDBSP_PROCF_BUILDNODES | ZDBSP_PROCF_CHECKPOLYOBJS);
}

zdbsp_RejectMode zdbsp_rejectmode_default(void) {
	return ZDBSP_ERM_DONTTOUCH;
}

zdbsp_BlockmapMode zdbsp_blockmapmode_default(void) {
	return ZDBSP_EBM_REBUILD;
}

zdbsp_WadReaderPtr zdbsp_wadreader_new(const uint8_t* bytes) {
	auto wad = std::make_unique<FWadReader>(bytes);
	return wad.release();
}

void zdbsp_wadreader_destroy(zdbsp_WadReaderPtr wad) {
	auto _ = std::unique_ptr<FWadReader>(wad);
}

zdbsp_ProcessorPtr zdbsp_processor_new(
	zdbsp_WadReaderPtr wad, const zdbsp_ProcessConfig* const config
) {
	auto p = std::make_unique<FProcessor>(*wad, 0);

	if (config != nullptr) {
		p->blockmap_mode = config->blockmap_mode;
		p->reject_mode = config->reject_mode;

		p->build_nodes = (config->flags & ZDBSP_PROCF_BUILDNODES);
		p->conform_nodes = (config->flags & ZDBSP_PROCF_CONFORMNODES);
		p->no_prune = (config->flags & ZDBSP_PROCF_NOPRUNE);
		p->check_poly_objs = (config->flags & ZDBSP_PROCF_CHECKPOLYOBJS);
		p->build_gl_nodes = (config->flags & ZDBSP_PROCF_BUILDGLNODES);
		p->gl_only = (config->flags & ZDBSP_PROCF_GLONLY);
		p->v5gl = (config->flags & ZDBSP_PROCF_V5GL);
		p->write_comments = (config->flags & ZDBSP_PROCF_WRITECOMMENTS);
		p->compress_nodes = (config->flags & ZDBSP_PROCF_COMPRESSNODES);
		p->compress_gl_nodes = (config->flags & ZDBSP_PROCF_COMPRESSGLNODES);
		p->force_compression = (config->flags & ZDBSP_PROCF_FORCECOMPRESSION);
	}

	return p.release();
}

void zdbsp_processor_run(zdbsp_ProcessorPtr p, const zdbsp_NodeConfig* const config) {
	p->Process(config);
}

size_t zdbsp_processor_nodesx_count(zdbsp_ProcessorPtr p) {
	return p->get_level().NumNodes;
}

static void processor_nodes_foreach(
	void* ctx, zdbsp_NodeVisitor callback, const zdbsp_NodeEx* node_array, size_t node_count
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

static void processor_subsectors_foreach(
	void* ctx, zdbsp_SubsectorVisitor callback, const zdbsp_SubsectorEx* ss_array, size_t ss_count
) {
	for (size_t i = 0; i < ss_count; ++i) {
		const auto& ss = ss_array[i];
		zdbsp_SubsectorRaw subsect = {};

		subsect.first_line = LittleShort((uint16_t)ss.first_line);
		subsect.num_lines = LittleShort((uint16_t)ss.num_lines);

		callback(ctx, &subsect);
	}
}

void zdbsp_processor_nodes_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeVisitor callback) {
	auto& level = p->get_level();
	processor_nodes_foreach(ctx, callback, level.Nodes, level.NumNodes);
}

void zdbsp_processor_segs_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegVisitor callback) {
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

void zdbsp_processor_ssectors_foreach(
	zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorVisitor callback
) {
	auto& level = p->get_level();
	processor_subsectors_foreach(ctx, callback, level.Subsectors, level.NumSubsectors);
}

void zdbsp_processor_glnodes_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_NodeVisitor callback) {
	auto& level = p->get_level();
	processor_nodes_foreach(ctx, callback, level.GLNodes, level.NumGLNodes);
}

void zdbsp_processor_glsegs_foreach(zdbsp_ProcessorPtr p, void* ctx, zdbsp_SegGlVisitor callback) {
	auto& level = p->get_level();

	for (size_t i = 0; i < level.NumGLSegs; ++i) {
		const auto& s = level.GLSegs[i];
		zdbsp_SegGl seg = {};

		if (s.v1 < (uint32_t)level.NumOrgVerts) {
			seg.v1 = LittleShort((uint16_t)s.v1);
		} else {
			seg.v1 = LittleShort(0x8000 | (uint16_t)(s.v1 - level.NumOrgVerts));
		}

		if (s.v2 < (DWORD)level.NumOrgVerts) {
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

void zdbsp_processor_glssectors_foreach(
	zdbsp_ProcessorPtr p, void* ctx, zdbsp_SubsectorVisitor callback
) {
	auto& level = p->get_level();
	processor_subsectors_foreach(ctx, callback, level.GLSubsectors, level.NumGLSubsectors);
}

void zdbsp_processor_destroy(zdbsp_ProcessorPtr p) {
	auto _ = std::unique_ptr<FProcessor>(p);
}
