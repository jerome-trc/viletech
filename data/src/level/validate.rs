use util::EditorNum;

use super::{repr::BspNodeChild, Error, LevelDef, SideTexture};

impl LevelDef {
	/// Verifies:
	/// - Level geometry and BSP tree reference indices, to ensure that none are out-of-bounds.
	/// - That no sides or sectors reference a non-existent texture.
	/// - Thing editor numbers, to ensure that no non-existent things have been placed.
	/// - That there is at least a player 1 start spot thing in the level.
	///
	/// Returns the number of errors raised.
	pub fn validate(
		&self,
		mut err_handler: impl FnMut(Error),
		mut texture_exists: impl FnMut(&str) -> bool,
		mut ednum_exists: impl FnMut(EditorNum) -> bool,
	) -> usize {
		let mut ret = 0;

		for (i, linedef) in self.geom.linedefs.iter().enumerate() {
			if linedef.side_right >= self.geom.sidedefs.len() {
				err_handler(Error::InvalidLinedefSide {
					linedef: i,
					left: false,
					sidedef: linedef.side_right,
					sides_len: self.geom.sidedefs.len(),
				});

				ret += 1;
			}

			let Some(side_left) = linedef.side_left else { continue; };

			if side_left >= self.geom.sidedefs.len() {
				err_handler(Error::InvalidLinedefSide {
					linedef: i,
					left: true,
					sidedef: side_left,
					sides_len: self.geom.sidedefs.len(),
				});

				ret += 1;
			}
		}

		for (i, node) in self.bsp.nodes.iter().enumerate() {
			match node.child_l {
				BspNodeChild::SubSector(ssector) => {
					err_handler(Error::InvalidNodeSubsector {
						node: i,
						left: true,
						ssector,
						ssectors_len: self.bsp.subsectors.len(),
					});

					ret += 1;
				}
				BspNodeChild::SubNode(subnode) => {
					err_handler(Error::InvalidSubnode {
						node: i,
						left: true,
						subnode,
						nodes_len: self.bsp.nodes.len(),
					});

					ret += 1;
				}
			}

			match node.child_r {
				BspNodeChild::SubSector(ssector) => {
					err_handler(Error::InvalidNodeSubsector {
						node: i,
						left: false,
						ssector,
						ssectors_len: self.bsp.subsectors.len(),
					});

					ret += 1;
				}
				BspNodeChild::SubNode(subnode) => {
					err_handler(Error::InvalidSubnode {
						node: i,
						left: false,
						subnode,
						nodes_len: self.bsp.nodes.len(),
					});

					ret += 1;
				}
			}
		}

		for (i, sectordef) in self.geom.sectordefs.iter().enumerate() {
			if let Some(tex_floor) = &sectordef.tex_floor {
				if !texture_exists(tex_floor.as_str()) {
					err_handler(Error::UnknownFlat {
						sector: i,
						ceiling: false,
						name: *tex_floor,
					});

					ret += 1;
				}
			}

			if let Some(tex_ceil) = &sectordef.tex_ceil {
				if !texture_exists(tex_ceil.as_str()) {
					err_handler(Error::UnknownFlat {
						sector: i,
						ceiling: true,
						name: *tex_ceil,
					});

					ret += 1;
				}
			}
		}

		for (i, seg) in self.bsp.segs.iter().enumerate() {
			if seg.linedef >= self.geom.linedefs.len() {
				err_handler(Error::InvalidSegLinedef {
					seg: i,
					linedef: seg.linedef,
					lines_len: self.geom.linedefs.len(),
				});

				ret += 1;
			}
		}

		for (i, sidedef) in self.geom.sidedefs.iter().enumerate() {
			if let Some(tex_bottom) = &sidedef.tex_bottom {
				if !texture_exists(tex_bottom.as_str()) {
					err_handler(Error::UnknownSideTex {
						sidedef: i,
						which: SideTexture::Bottom,
						name: *tex_bottom,
					});

					ret += 1;
				}
			}

			if let Some(tex_mid) = &sidedef.tex_mid {
				if !texture_exists(tex_mid.as_str()) {
					err_handler(Error::UnknownSideTex {
						sidedef: i,
						which: SideTexture::Middle,
						name: *tex_mid,
					});

					ret += 1;
				}
			}

			if let Some(tex_top) = &sidedef.tex_top {
				if !texture_exists(tex_top.as_str()) {
					err_handler(Error::UnknownSideTex {
						sidedef: i,
						which: SideTexture::Top,
						name: *tex_top,
					});

					ret += 1;
				}
			}

			if sidedef.sector >= self.geom.sectordefs.len() {
				err_handler(Error::InvalidSidedefSector {
					sidedef: i,
					sector: sidedef.sector,
					sectors_len: self.geom.sectordefs.len(),
				});

				ret += 1;
			}
		}

		for (i, subsector) in self.bsp.subsectors.iter().enumerate() {
			if subsector.seg0 >= self.bsp.segs.len() {
				err_handler(Error::InvalidSubsectorSeg {
					subsector: i,
					seg: subsector.seg0,
					segs_len: self.bsp.segs.len(),
				});

				ret += 1;
			}
		}

		let mut player1start = false;

		for (i, thingdef) in self.thingdefs.iter().enumerate() {
			if thingdef.ed_num == 1 {
				player1start = true;
			}

			if !ednum_exists(thingdef.ed_num) {
				err_handler(Error::UnknownEdNum {
					thingdef: i,
					ed_num: thingdef.ed_num,
				});

				ret += 1;
			}
		}

		if !player1start {
			err_handler(Error::NoPlayer1Start);
			ret += 1;
		}

		ret
	}
}
