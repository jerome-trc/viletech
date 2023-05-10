//! Functions providing ECS components to level lines.

use bevy::ecs::system::EntityCommands;

use crate::{
	data::dobj::{LevelFormat, UdmfNamespace},
	sim::line,
};

pub(super) fn _line_special_bundle(mut line: EntityCommands, format: LevelFormat, num: u16) {
	match format {
		LevelFormat::Doom => match num {
			0 => {
				// Just an ordinary line.
			}
			1 => {
				line.insert(line::Door {
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
