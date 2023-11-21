use viletech::{data::gfx::PictureReader, vfs::FileRef};

/// The editor's best guess at the content in a file.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContentId {
	#[default]
	Unknown,
	PlainText,

	/// Action Code Script text source. See <https://doomwiki.org/wiki/ACS>.
	Acs,
	/// Compiled Action Code Script bytecode blob. See <https://doomwiki.org/wiki/ACS>.
	AcsObject,
	/// See <https://zdoom.org/wiki/ALTHUDCF>.
	AltHudCf,
	/// See <https://doomwiki.org/wiki/ANIMATED>.
	Animated,
	/// See <https://zdoom.org/wiki/ANIMDEFS>.
	AnimDefs,
	/// See, for example, <https://doomwiki.org/wiki/ENDOOM>.
	AnsiText,
	/// See <https://doomwiki.org/wiki/Behavior>.
	Behavior,
	/// See <https://doomwiki.org/wiki/Blockmap>.
	Blockmap,
	/// See <https://doomwiki.org/wiki/COLORMAP>.
	Colormap,
	/// See <https://zdoom.org/wiki/CVARINFO>.
	CVarInfo,
	/// See <https://zdoom.org/wiki/DECALDEF>.
	DecalDef,
	/// See <https://zdoom.org/wiki/DECORATE>.
	Decorate,
	/// See <https://zdoom.org/wiki/DEFBINDS>.
	DefBinds,
	/// See <https://zdoom.org/wiki/DEFCVARS>.
	DefCVars,
	/// See <https://doomwiki.org/wiki/DeHackEd>.
	DeHackEd,
	/// See [`Self::DeHackEd`].
	DeHackEdBex,
	/// See <https://doomwiki.org/wiki/Demo>.
	Demo,
	/// See <https://doomwiki.org/wiki/DMXGUS>.
	DmxGus,
	/// See <https://doomwiki.org/wiki/MUS>.
	DmxMus,
	/// See <https://eternity.youfailit.net/wiki/EDF>.
	Edf,
	/// See <https://eternity.youfailit.net/wiki/EMAPINFO>.
	EMapInfo,
	/// See <https://doomwiki.org/wiki/Flat>.
	Flat,
	/// See <https://zdoom.org/wiki/FONTDEFS>.
	FontDefs,
	/// Global FraggleScript. See <https://zdoom.org/wiki/FraggleScript>.
	FsGlobal,
	/// See <https://zdoom.org/wiki/GAMEINFO>.
	GameInfo,
	/// See <https://doomwiki.org/wiki/GENMIDI>.
	GenMidi,
	/// See <https://zdoom.org/wiki/GLDEFS>.
	GlDefs,
	/// See <https://en.wikipedia.org/wiki/OpenGL_Shading_Language>.
	Glsl,
	/// See <https://en.wikipedia.org/wiki/High-Level_Shader_Language>.
	Hlsl,
	/// See <https://zdoom.org/wiki/IWADINFO>.
	IWadInfo,
	/// See <https://zdoom.org/wiki/KEYCONF>.
	KeyConf,
	/// See <https://doomwiki.org/wiki/Linedef>.
	LineDefs,
	/// A [Lithica](viletech::lith) source file.
	Lith,
	/// See <https://zdoom.org/wiki/LOADACS>.
	LoadAcs,
	/// See <https://zdoom.org/wiki/LOCKDEFS>.
	LockDefs,
	/// See <https://doomwiki.org/wiki/MAPINFO>.
	MapInfo,
	/// See <https://doomwiki.org/wiki/WAD#Map_data_lumps>.
	MapMarker,
	/// See <https://en.wikipedia.org/wiki/Markdown>.
	Markdown,
	/// See <https://doomwiki.org/wiki/WAD#Lump_order>.
	Marker,
	/// See <https://doomwiki.org/wiki/MIDI>.
	Midi,
	/// See <https://zdoom.org/wiki/MODELDEF>.
	ModelDef,
	/// See <https://zdoom.org/wiki/MUSINFO>.
	MusInfo,
	/// See <https://doomwiki.org/wiki/Node>.
	Nodes,
	/// See <https://zdoom.org/wiki/PALVERS>.
	PalVers,
	/// See <https://doomwiki.org/wiki/Picture_format>.
	Picture,
	/// A VileTech package metadata text file.
	PkgMeta,
	/// See <https://doomwiki.org/wiki/PLAYPAL>.
	PlayPal,
	/// See <https://doomwiki.org/wiki/PNAMES>.
	PNames,
	/// See <https://en.wikipedia.org/wiki/PNG>.
	Png,
	/// See <https://doomwiki.org/wiki/Reject>.
	Reject,
	/// See <https://zdoom.org/wiki/SBARINFO>.
	SBarInfo,
	/// See <https://doomwiki.org/wiki/Sector>.
	Sectors,
	/// See <https://doomwiki.org/wiki/Seg>.
	Segs,
	/// See <https://doomwiki.org/wiki/Sidedef>.
	SideDefs,
	/// See <https://zdoom.org/wiki/SNDINFO>.
	SndInfo,
	/// See <https://zdoom.org/wiki/SNDSEQ>.
	SndSeq,
	/// See <https://doomwiki.org/wiki/Subsector>.
	SSectors,
	/// See <https://doomwiki.org/wiki/SWITCHES>.
	Switches,
	/// See <https://zdoom.org/wiki/TERRAIN>.
	Terrain,
	/// See <https://zdoom.org/wiki/TEXTCOLO>.
	TextColo,
	/// UDMF TEXTMAP lump. See <https://doomwiki.org/wiki/UDMF>.
	TextMap,
	/// See <https://doomwiki.org/wiki/TEXTURE1_and_TEXTURE2>.
	TextureX,
	/// See <https://doomwiki.org/wiki/Thing>.
	Things,
	/// See <https://zdoom.org/wiki/TRNSLATE>.
	Trnslate,
	/// See <https://doomwiki.org/wiki/UMAPINFO>.
	UMapInfo,
	/// See <https://doomwiki.org/wiki/Vertex>.
	Vertexes,
	/// See <https://zdoom.org/wiki/VOXELDEF>.
	VoxelDef,
	/// An informal standard for level pack metadata.
	WadInfo,
	/// See <https://www.w3.org/TR/WGSL/>.
	Wgsl,
	/// See <https://zdoom.org/wiki/X11R6RGB>.
	X11R6RGB,
	/// See <https://zdoom.org/wiki/XHAIRS>.
	XHairs,
	/// [`Self::MapInfo`], but ZDoom-specific.
	ZMapInfo,
	/// See <https://zdoom.org/wiki/ZScript>.
	ZScript,
}

impl std::fmt::Display for ContentId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Unknown => write!(f, "Unknown"),
			Self::PlainText => write!(f, "Plain text"),

			Self::Acs => write!(f, "ACS source"),
			Self::AcsObject => write!(f, "Compiled ACS"),
			Self::AltHudCf => write!(f, "ZDoom alt. HUD config."),
			Self::Animated => write!(f, "Boom animations"),
			Self::AnimDefs => write!(f, "ZDoom animations"),
			Self::AnsiText => write!(f, "ANSI text"),
			Self::Behavior => write!(f, "ACS map scripts"),
			Self::Blockmap => write!(f, "Blockmap data"),
			Self::Colormap => write!(f, "Colormap"),
			Self::CVarInfo => write!(f, "CVar defs"),
			Self::DecalDef => write!(f, "ZDoom decal config."),
			Self::Decorate => write!(f, "DECORATE"),
			Self::DefBinds => write!(f, "ZDoom keybind defaults"),
			Self::DefCVars => write!(f, "ZDoom CVar defaults"),
			Self::DeHackEd => write!(f, "DeHackEd"),
			Self::DeHackEdBex => write!(f, "DeH. Boom EXtended"),
			Self::Demo => write!(f, "Demo"),
			Self::DmxMus => write!(f, "MUS audio"),
			Self::DmxGus => write!(f, "DMX GUS config."),
			Self::Edf => write!(f, "Eternity definitions"),
			Self::EMapInfo => write!(f, "Eternity map info."),
			Self::Flat => write!(f, "Graphic (Flat)"),
			Self::FontDefs => write!(f, "ZDoom font config."),
			Self::FsGlobal => write!(f, "Global FraggleScript"),
			Self::GameInfo => write!(f, "ZDoom game config."),
			Self::GenMidi => write!(f, "General MIDI set"),
			Self::GlDefs => write!(f, "OpenGL config."),
			Self::Glsl => write!(f, "OpenGL shader"),
			Self::Hlsl => write!(f, "Direct3D shader"),
			Self::IWadInfo => write!(f, "ZDoom IWAD config."),
			Self::KeyConf => write!(f, "ZDoom keybindings"),
			Self::Lith => write!(f, "Lithica source"),
			Self::LineDefs => write!(f, "Map lines"),
			Self::LoadAcs => write!(f, "ACS loader"),
			Self::LockDefs => write!(f, "ZDoom lock/key config."),
			Self::MapInfo => write!(f, "Map info"),
			Self::MapMarker => write!(f, "Map marker"),
			Self::Markdown => write!(f, "Markdown"),
			Self::Marker => write!(f, "Marker"),
			Self::Midi => write!(f, "MIDI audio"),
			Self::MusInfo => write!(f, "ZDoom music config."),
			Self::ModelDef => write!(f, "ZDoom model config."),
			Self::Nodes => write!(f, "Map BSP nodes"),
			Self::PalVers => write!(f, "ZDoom palette versioning"),
			Self::Picture => write!(f, "Graphic (picture)"),
			Self::PkgMeta => write!(f, "VileTech package metadata"),
			Self::PlayPal => write!(f, "Color palette"),
			Self::PNames => write!(f, "Patch table"),
			Self::Png => write!(f, "Graphic (PNG)"),
			Self::Reject => write!(f, "Map reject table"),
			Self::SBarInfo => write!(f, "ZDoom HUD config."),
			Self::Sectors => write!(f, "Map sectors"),
			Self::Segs => write!(f, "Map segments"),
			Self::SideDefs => write!(f, "Map sides"),
			Self::SndInfo => write!(f, "ZDoom sound config."),
			Self::SndSeq => write!(f, "ZDoom sound sequence"),
			Self::SSectors => write!(f, "Map subsectors"),
			Self::Switches => write!(f, "Boom switches"),
			Self::Terrain => write!(f, "ZDoom terrain config."),
			Self::TextColo => write!(f, "Font colors"),
			Self::TextMap => write!(f, "UDMF textmap"),
			Self::TextureX => write!(f, "TEXTURE1/TEXTURE2"),
			Self::Things => write!(f, "Map object placements"),
			Self::Trnslate => write!(f, "ZDoom translations"),
			Self::UMapInfo => write!(f, "Universal map info"),
			Self::Vertexes => write!(f, "Map vertices"),
			Self::VoxelDef => write!(f, "ZDoom voxel config."),
			Self::WadInfo => write!(f, "WAD metadata"),
			Self::Wgsl => write!(f, "WebGPU shader"),
			Self::X11R6RGB => write!(f, "ZDoom X11R6RGB"),
			Self::XHairs => write!(f, "ZDoom crosshair config."),
			Self::ZMapInfo => write!(f, "ZDoom map info"),
			Self::ZScript => write!(f, "ZScript"),
		}
	}
}

impl ContentId {
	/// TODO: might be worthwhile to make this a hashmap.
	const FILE_STEMS: &'static [(&'static str, Self)] = &[
		("ALTHUDCF", Self::AltHudCf),
		("ANIMATED", Self::Animated),
		("ANIMDEFS", Self::AnimDefs),
		("BEHAVIOR", Self::Behavior),
		("BLOCKMAP", Self::Blockmap),
		("COLORMAP", Self::Colormap),
		("CVARINFO", Self::CVarInfo),
		("DECALDEF", Self::DecalDef),
		("DECORATE", Self::Decorate),
		("DEFBINDS", Self::DefBinds),
		("DEFCVARS", Self::DefCVars),
		("DEHACKED", Self::DeHackEd),
		("DMXGUS", Self::DmxGus),
		("DMXGUSC", Self::DmxGus),
		("EMAPINFO", Self::EMapInfo),
		("ENDOOM", Self::AnsiText),
		("FONTDEFS", Self::FontDefs),
		("FSGLOBAL", Self::FsGlobal),
		("GAMEINFO", Self::GameInfo),
		("GENMIDI", Self::GenMidi),
		("GLDEFS", Self::GlDefs),
		("IWADINFO", Self::IWadInfo),
		("KEYCONF", Self::KeyConf),
		("LINEDEFS", Self::LineDefs),
		("LOADACS", Self::LoadAcs),
		("LOCKDEFS", Self::LockDefs),
		("MAPINFO", Self::MapInfo),
		("MODELDEF", Self::ModelDef),
		("MUSINFO", Self::MusInfo),
		("NODES", Self::Nodes),
		("PALVERS", Self::PalVers),
		("PLAYPAL", Self::PlayPal),
		("PNAMES", Self::PNames),
		("REJECT", Self::Reject),
		("SBARINFO", Self::SBarInfo),
		("SECTORS", Self::Sectors),
		("SEGS", Self::Segs),
		("SIDEDEFS", Self::SideDefs),
		("SNDINFO", Self::SndInfo),
		("SNDSEQ", Self::SndSeq),
		("SSECTORS", Self::SSectors),
		("SWITCHES", Self::Switches),
		("TERRAIN", Self::Terrain),
		("TEXTCOLO", Self::TextColo),
		("TEXTMAP", Self::TextMap),
		("TEXTURE1", Self::TextureX),
		("TEXTURE2", Self::TextureX),
		("THINGS", Self::Things),
		("TRNSLATE", Self::Trnslate),
		("UMAPINFO", Self::UMapInfo),
		("VERTEXES", Self::Vertexes),
		("VOXELDEF", Self::VoxelDef),
		("WADINFO", Self::WadInfo),
		("X11R6RGB", Self::X11R6RGB),
		("XHAIRS", Self::XHairs),
		("ZMAPINFO", Self::ZMapInfo),
		("ZSCRIPT", Self::ZScript),
	];

	const FILE_STEM_MARKER_PREFIXES: &'static [&'static str] = &[
		"A_", "F_", "F1_", "F2_", "HI_", "P_", "PP_", "P1_", "P2_", "P3_", "S_", "SS_", "T_", "TX_",
	];

	const EXTENSIONS: &'static [(&'static str, Self)] = &[
		("acs", Self::Acs),
		("bex", Self::DeHackEdBex),
		("dec", Self::Decorate),
		("deh", Self::DeHackEd),
		("edf", Self::Edf),
		("glsl", Self::Glsl),
		("hlsl", Self::Hlsl),
		("lith", Self::Lith),
		("md", Self::Markdown),
		("txt", Self::PlainText),
		("wgsl", Self::Wgsl),
		("zs", Self::ZScript),
		("zsc", Self::ZScript),
	];

	#[must_use]
	pub(super) fn is_text(self) -> bool {
		// Don't change this to use `matches`.
		// We want a compiler error to get raised if new, unhandled variants are added.

		match self {
			Self::PlainText
			| Self::Acs
			| Self::AltHudCf
			| Self::CVarInfo
			| Self::DecalDef
			| Self::Decorate
			| Self::DefBinds
			| Self::DefCVars
			| Self::DeHackEd
			| Self::DeHackEdBex
			| Self::Edf
			| Self::EMapInfo
			| Self::FontDefs
			| Self::FsGlobal
			| Self::GameInfo
			| Self::GenMidi
			| Self::GlDefs
			| Self::Glsl
			| Self::Hlsl
			| Self::IWadInfo
			| Self::KeyConf
			| Self::Lith
			| Self::LoadAcs
			| Self::LockDefs
			| Self::MapInfo
			| Self::Markdown
			| Self::ModelDef
			| Self::MusInfo
			| Self::PalVers
			| Self::PkgMeta
			| Self::SBarInfo
			| Self::SndInfo
			| Self::Terrain
			| Self::TextColo
			| Self::TextMap
			| Self::Trnslate
			| Self::UMapInfo
			| Self::VoxelDef
			| Self::WadInfo
			| Self::Wgsl
			| Self::X11R6RGB
			| Self::XHairs
			| Self::ZMapInfo
			| Self::ZScript => true,
			Self::AnimDefs
			| Self::Unknown
			| Self::AcsObject
			| Self::Animated
			| Self::AnsiText
			| Self::Behavior
			| Self::Blockmap
			| Self::Colormap
			| Self::Demo
			| Self::DmxGus
			| Self::DmxMus
			| Self::Flat
			| Self::LineDefs
			| Self::MapMarker
			| Self::Marker
			| Self::Midi
			| Self::Nodes
			| Self::Picture
			| Self::PlayPal
			| Self::PNames
			| Self::Png
			| Self::Reject
			| Self::Sectors
			| Self::Segs
			| Self::SideDefs
			| Self::SndSeq
			| Self::SSectors
			| Self::Switches
			| Self::TextureX
			| Self::Things
			| Self::Vertexes => false,
		}
	}

	#[must_use]
	pub(super) fn deduce(vfile: &FileRef, bytes: &[u8], markers: WadMarkers) -> Self {
		if let Some(next) = vfile.next_sibling() {
			if next.name().eq_ignore_ascii_case("THINGS")
				|| next.name().eq_ignore_ascii_case("TEXTMAP")
			{
				return Self::MapMarker;
			}
		}

		let fname = vfile.name();

		if let Some(ext) = fname.extension() {
			if fname
				.file_prefix()
				.is_some_and(|p| p.eq_ignore_ascii_case("meta"))
				&& ext.eq_ignore_ascii_case("toml")
			{
				return Self::PkgMeta;
			}

			if let Some(tuple) = Self::EXTENSIONS
				.iter()
				.find(|t| t.0.eq_ignore_ascii_case(ext))
			{
				return tuple.1;
			}
		}

		if let Some(fstem) = fname.file_stem() {
			let fstem = fstem.as_str();

			if bytes.is_empty() {
				let buf = fstem.to_ascii_uppercase();

				if Self::FILE_STEM_MARKER_PREFIXES
					.iter()
					.any(|&p| buf.starts_with(p))
					&& (buf.ends_with("START") || buf.ends_with("END"))
				{
					return ContentId::Marker;
				}
			}

			if let Some(tuple) = Self::FILE_STEMS
				.iter()
				.find(|t| t.0.eq_ignore_ascii_case(fstem))
			{
				return tuple.1;
			}

			if fstem
				.get(0..4)
				.is_some_and(|f4| f4.eq_ignore_ascii_case("demo"))
			{
				return Self::Demo;
			}
		}

		if viletech::mus::is_dmxmus(bytes) {
			return Self::DmxMus;
		}

		if viletech::audio::MidiFormat::deduce(bytes).is_some() {
			return Self::Midi;
		}

		if viletech::util::io::is_png(bytes) {
			return Self::Png;
		}

		if viletech::data::acs::is_object(bytes) {
			return Self::AcsObject;
		}

		match markers {
			WadMarkers::None => {}
			WadMarkers::Flats => {
				if bytes.len() == 4096 {
					return Self::Flat;
				}
			}
		}

		if PictureReader::new(bytes).is_ok() {
			return Self::Picture;
		}

		Self::Unknown
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WadMarkers {
	None,
	Flats,
}
