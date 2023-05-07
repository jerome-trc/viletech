//! Functions for assembling a level's compact asset representation into ECS form.

use std::collections::{hash_map::RandomState, HashMap, HashSet};

use asset::{LevelFormat, UdmfNamespace};
use bevy::{ecs::system::EntityCommands, prelude::*, render::render_resource::PrimitiveTopology};
use indexmap::IndexMap;
use rayon::prelude::*;
use triangulate::{formats::IndexedListFormat, ListFormat, Polygon};

use crate::{data::asset, sim::level, sim::ActiveMarker, sparse::SparseSet, BaseGame};

use super::{
	line::{self, Line},
	sector::{self, Sector},
	Side, SideIndex, Udmf, VertIndex,
};

pub fn init(
	mut cmds: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	base: asset::Handle<asset::Level>,
	active: bool,
) {
	let level_id = if active {
		cmds.spawn(ActiveMarker)
	} else {
		cmds.spawn(())
	}
	.id();

	let mut verts = SparseSet::with_capacity(base.vertices.len(), base.vertices.len());
	let mut sides = SparseSet::with_capacity(base.sidedefs.len(), base.sidedefs.len());

	for (i, vert) in base.vertices.iter().enumerate() {
		verts.insert(VertIndex(i), vert.clone());
	}

	let mut lines = IndexMap::with_capacity(base.linedefs.len());
	let mut sectors = IndexMap::with_capacity(base.sectors.len());

	let mut sectors_by_trigger: HashMap<_, _, RandomState> = HashMap::default();

	for linedef in &base.linedefs {
		let line_id = cmds.spawn(()).id();

		lines.insert(
			Line(line_id),
			(
				line::Core {
					udmf_id: linedef.udmf_id,
					vert_start: VertIndex(linedef.vert_start),
					vert_end: VertIndex(linedef.vert_end),
					flags: linedef.flags,
					side_right: SideIndex(linedef.side_right),
					side_left: linedef.side_left.map(SideIndex),
				},
				linedef.special,
			),
		);
	}

	for sectordef in &base.sectors {
		let sect_id = cmds.spawn(()).id();
		sectors.insert(
			Sector(sect_id),
			(sector::Core { lines: vec![] }, sectordef.special),
		);
		let trigger = line::Trigger(sectordef.trigger);

		let sect_grp = sectors_by_trigger.entry(trigger).or_insert(vec![]);
		sect_grp.push(Sector(sect_id));
	}

	for (i, sidedef) in base.sidedefs.iter().enumerate() {
		sides.insert(
			SideIndex(i),
			Side {
				offset: sidedef.offset,
				sector: *sectors.get_index(sidedef.sector).unwrap().0,
				udmf: Udmf::default(),
			},
		);
	}

	for (line_id, (line, _)) in &lines {
		let side_r = &sides[line.side_right];
		let (sect, _) = sectors.get_mut(&side_r.sector).unwrap();
		sect.lines.push(*line_id);

		if let Some(side_l_idx) = line.side_left {
			let side_l = &sides[side_l_idx];
			let (sect, _) = sectors.get_mut(&side_l.sector).unwrap();
			sect.lines.push(*line_id);
		}
	}

	let mesh = Mesh::new(PrimitiveTopology::TriangleList);

	let mut poly_buf = vec![];
	let mut vert_set = HashSet::<_, RandomState>::default();
	let mut tri_indices = Vec::<usize>::default();

	for (_, (sect, _)) in &sectors {
		poly_buf.clear();
		vert_set.clear();
		tri_indices.clear();

		for line_id in &sect.lines {
			let (line, _) = lines.get(line_id).unwrap();

			if !vert_set.contains(&line.vert_start) {
				let vert = &verts[line.vert_start];
				vert_set.insert(line.vert_start);
				poly_buf.push((vert.0.x, vert.0.y))
			}

			if !vert_set.contains(&line.vert_end) {
				let vert = &verts[line.vert_end];
				vert_set.insert(line.vert_end);
				poly_buf.push((vert.0.x, vert.0.y));
			}
		}

		let center = center_of_sector(&poly_buf);

		poly_buf.par_sort_by(|v1, v2| {
			let slope1 = (v1.0 - center.0, v1.1 - center.1);
			let slope2 = (v2.0 - center.0, v2.1 - center.1);
			let theta1 = slope1.1.atan2(slope1.0);
			let theta2 = slope2.1.atan2(slope2.0);
			theta1.total_cmp(&theta2)
		});

		let format = IndexedListFormat::new(&mut tri_indices).into_fan_format();

		match poly_buf.triangulate(format) {
			Ok(_) => {
				// TODO: Due to implicit edges, this is not necessarily correct.
				// The next step is to involve nodes, segs, and sub-sectors.
			}
			Err(err) => {
				unimplemented!(
					"Error handling for sector triangulation failure is unimplemented. ({err})"
				);
			}
		}
	}

	for (line_id, (line, special)) in lines {
		let mut ent = cmds.get_entity(line_id.0).unwrap();
		ent.insert(line);
		line_special_bundle(ent, base.format, special);
		cmds.get_entity(level_id).unwrap().add_child(line_id.0);
	}

	for (sect_id, (sect, special)) in sectors {
		let mut ent = cmds.get_entity(sect_id.0).unwrap();
		ent.insert(sect);
		sector_special_bundle(ent, BaseGame::Doom, base.format, special);
		cmds.get_entity(level_id).unwrap().add_child(sect_id.0);
	}

	cmds.get_entity(level_id).unwrap().insert(level::Core {
		base: Some(base.clone()),
		flags: level::Flags::empty(),
		ticks_elapsed: 0,
		geom: level::Geometry {
			mesh: meshes.add(mesh),
			verts,
			sides,
			triggers: sectors_by_trigger,
			num_sectors: base.sectors.len(),
		},
	});
}

fn line_special_bundle(mut cmds: EntityCommands, format: LevelFormat, num: u16) {
	match format {
		LevelFormat::Doom => match num {
			1 => {
				cmds.insert(line::Door {
					stay_time: 35 * 4,
					stay_timer: 0,
					one_off: false,
					monster_usable: true,
					remote: false,
					speed: line::Door::SPEED_NORMAL,
					lock: None,
				});
			}
			other => unimplemented!("Doom line special {other} is unimplemented."),
		},
		LevelFormat::Hexen => todo!(),
		LevelFormat::Udmf(namespace) => match namespace {
			UdmfNamespace::Doom => todo!(),
			other => unimplemented!("UDMF namespace `{other:#?}` is not yet supported."),
		},
	}
}

fn sector_special_bundle(cmds: EntityCommands, game: BaseGame, format: LevelFormat, num: u16) {
	match game {
		BaseGame::Doom => match format {
			LevelFormat::Doom => sector_special_bundle_boom(cmds, num),
			LevelFormat::Udmf(UdmfNamespace::ZDoom) => sector_special_bundle_zdoom(cmds, num),
			_ => unimplemented!("Unsupported configuration: {game:#?}/{format:#?}"),
		},
		BaseGame::Hexen => {
			sector_special_bundle_zdoom(cmds, num);
		}
		BaseGame::Heretic => {
			sector_special_bundle_heretic(cmds, num);
		}
		BaseGame::Strife => {
			sector_special_bundle_strife(cmds, num);
		}
		BaseGame::ChexQuest => {
			// TODO: Not sure yet.
		}
	}
}

fn sector_special_bundle_boom(mut cmds: EntityCommands, num: u16) {
	if (num & 96) != 0 {
		cmds.insert(sector::Damaging {
			damage: 20,
			interval: 35,
			leak_chance: 6,
		});
	} else if (num & 64) != 0 {
		cmds.insert(sector::Damaging {
			damage: 10,
			interval: 35,
			leak_chance: 0,
		});
	} else if (num & 32) != 0 {
		cmds.insert(sector::Damaging {
			damage: 5,
			interval: 35,
			leak_chance: 0,
		});
	}

	if (num & 128) != 0 {
		cmds.insert(sector::Secret);
	}

	if (num & 256) != 0 {
		unimplemented!("Boom friction effects are unimplemented.");
	}

	if (num & 512) != 0 {
		unimplemented!("Boom conveyor effects are unimplemented.");
	}

	match num {
		9 => {
			cmds.insert(sector::Secret);
		}
		10 => {
			cmds.insert(sector::CloseAfter { ticks: 35 * 30 });
		}
		11 => {
			cmds.insert(sector::Ending { threshold: 11 });

			cmds.insert(sector::Damaging {
				damage: 20,
				interval: 35,
				leak_chance: 6, // Q: Suit leak on ending damage floors?
			});
		}
		14 => {
			cmds.insert(sector::OpenAfter { ticks: 35 * 300 });
		}
		16 => {
			cmds.insert(sector::Damaging {
				damage: 20,
				interval: 35,
				leak_chance: 16,
			});
		}
		other => unimplemented!("Boom sector special {other} is unimplemented."),
	}
}

fn sector_special_bundle_heretic(mut _cmds: EntityCommands, _num: u16) {
	unimplemented!("Heretic sector specials are unimplemented.")
}

fn sector_special_bundle_strife(mut _cmds: EntityCommands, _num: u16) {
	unimplemented!("Strife sector specials are unimplemented.")
}

fn sector_special_bundle_zdoom(mut cmds: EntityCommands, num: u16) {
	match num {
		115 => {
			// Instant death.
			cmds.insert(sector::Damaging {
				damage: 999,
				interval: 1,
				leak_chance: u8::MAX,
			});
		}
		196 => {
			cmds.insert(sector::Healing {
				interval: 32,
				amount: 1,
			});
		}
		other => unimplemented!("ZDoom sector special {other} is unimplemented."),
	}
}

#[must_use]
fn center_of_sector(verts: &[(f32, f32)]) -> (f32, f32) {
	// Q: Is SIMD better for this than CPU parallelism?

	let sum = verts
		.par_iter()
		.cloned()
		.reduce(|| (0.0, 0.0), |base, arg| (base.0 + arg.0, base.1 + arg.1));

	let centre = ((sum.0 / verts.len() as f32), (sum.1 / verts.len() as f32));

	// Move the centre slightly so that the angles are not all equal
	// if the sector is a perfect quadrilateral.
	(centre.0 + f32::EPSILON, centre.1 + f32::EPSILON)
}
