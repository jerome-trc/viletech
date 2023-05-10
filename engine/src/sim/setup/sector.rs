//! Functions providing ECS components to level sectors.

use bevy::ecs::system::EntityCommands;

use crate::{
	data::dobj::{LevelFormat, UdmfNamespace},
	sim::sector,
	BaseGame,
};

pub(super) fn _sector_special_bundle(
	sector: EntityCommands,
	game: BaseGame,
	format: LevelFormat,
	num: u16,
) {
	match game {
		BaseGame::Doom => match format {
			LevelFormat::Doom => _sector_special_bundle_boom(sector, num),
			LevelFormat::Udmf(UdmfNamespace::ZDoom) => _sector_special_bundle_zdoom(sector, num),
			_ => unimplemented!("Unsupported configuration: {game:#?}/{format:#?}"),
		},
		BaseGame::Hexen => {
			_sector_special_bundle_zdoom(sector, num);
		}
		BaseGame::Heretic => {
			_sector_special_bundle_heretic(sector, num);
		}
		BaseGame::Strife => {
			_sector_special_bundle_strife(sector, num);
		}
		BaseGame::ChexQuest => {
			// TODO: Not sure yet.
		}
	}
}

fn _sector_special_bundle_boom(mut sector: EntityCommands, num: u16) {
	if (num & 96) != 0 {
		sector.insert(sector::Damaging {
			damage: 20,
			interval: 35,
			leak_chance: 6,
		});
	} else if (num & 64) != 0 {
		sector.insert(sector::Damaging {
			damage: 10,
			interval: 35,
			leak_chance: 0,
		});
	} else if (num & 32) != 0 {
		sector.insert(sector::Damaging {
			damage: 5,
			interval: 35,
			leak_chance: 0,
		});
	}

	if (num & 128) != 0 {
		sector.insert(sector::Secret);
	}

	if (num & 256) != 0 {
		unimplemented!("Boom friction effects are unimplemented.");
	}

	if (num & 512) != 0 {
		unimplemented!("Boom conveyor effects are unimplemented.");
	}

	match num {
		9 => {
			sector.insert(sector::Secret);
		}
		10 => {
			sector.insert(sector::CloseAfter { ticks: 35 * 30 });
		}
		11 => {
			sector.insert(sector::Ending { threshold: 11 });

			sector.insert(sector::Damaging {
				damage: 20,
				interval: 35,
				leak_chance: 6, // Q: Suit leak on ending damage floors?
			});
		}
		14 => {
			sector.insert(sector::OpenAfter { ticks: 35 * 300 });
		}
		16 => {
			sector.insert(sector::Damaging {
				damage: 20,
				interval: 35,
				leak_chance: 16,
			});
		}
		other => unimplemented!("Boom sector special {other} is unimplemented."),
	}
}

fn _sector_special_bundle_heretic(mut _sector: EntityCommands, _num: u16) {
	unimplemented!("Heretic sector specials are unimplemented.")
}

fn _sector_special_bundle_strife(mut _sector: EntityCommands, _num: u16) {
	unimplemented!("Strife sector specials are unimplemented.")
}

fn _sector_special_bundle_zdoom(mut cmds: EntityCommands, num: u16) {
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
