/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

mod commands;

use std::{
	error::Error,
	io::{self, Write},
	net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
	sync::atomic::AtomicBool,
	time::{Duration, Instant, SystemTime},
};

use clap::Parser;
use impure::terminal::Terminal;
use log::{error, info};

use commands::{Command, Flags as CommandFlags, Request as CommandRequest};
use renet::{RenetConnectionConfig, RenetServer, ServerAuthentication, ServerEvent};
use sha3::{Digest, Sha3_256};

#[must_use]
pub fn version_string() -> String {
	format!(
		"Impure dedicated server version: {}",
		env!("CARGO_PKG_VERSION")
	)
}

pub struct ServerCore {
	start_time: Instant,
	terminal: Terminal<Command>,
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
	#[clap(long, value_parser, default_value_t = 64)]
	max_clients: usize,
	/// Can be empty.
	#[clap(long, value_parser, default_value = "")]
	password: String,
	#[clap(long, value_parser, default_value_t = 6666)]
	port: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
	let start_time = Instant::now();
	let args = Args::parse();

	match impure::log_init(None) {
		Ok(()) => {}
		Err(err) => {
			eprintln!("Failed to initialise logging backend: {}", err);
			return Err(err);
		}
	}

	impure::log_init_diag(&version_string())?;

	let passhash = if !args.password.is_empty() {
		let mut hasher = Sha3_256::new();
		hasher.update(args.password);
		// TODO: Is there a way to salt this?
		Some(hasher.finalize())
	} else {
		None
	};

	let ipv4 = Ipv4Addr::new(0, 0, 0, 0);
	let addr = IpAddr::V4(ipv4);
	let public_addr = SocketAddr::new(addr, args.port);
	let socket = UdpSocket::bind(public_addr)?;
	let protocol_id = {
		let mut hasher = Sha3_256::new();
		hasher.update(env!("CARGO_PKG_VERSION"));
		bytemuck::try_cast_slice::<u8, u64>(&hasher.finalize()[0..16])
	}
	.expect("Failed to hash protocol ID from package version.")[0];

	let mut server = RenetServer::new(
		SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?,
		renet::ServerConfig {
			max_clients: args.max_clients,
			protocol_id,
			public_addr,
			authentication: ServerAuthentication::Unsecure,
		},
		RenetConnectionConfig::default(),
		socket,
	)?;

	const LOBBY_WAIT: Duration = Duration::from_millis(250);

	let lobby_running = AtomicBool::new(true);
	let mut cmd_buffer = String::with_capacity(64);

	let mut core = ServerCore {
		start_time,
		terminal: Terminal::<Command>::new(|key| {
			info!("Unknown command: {}", key);
		}),
	};

	let res = crossbeam::thread::scope(|scope| {
		let lobby_thread = scope.spawn(|_| {
			loop {
				if !lobby_running.load(std::sync::atomic::Ordering::Relaxed) {
					break;
				}

				match server.update(LOBBY_WAIT) {
					Ok(()) => {}
					Err(err) => {
						error!("Lobby update tick failed: {}", err);
						panic!();
					}
				};

				// Check for client connections/disconnections
				while let Some(event) = server.get_event() {
					match event {
						ServerEvent::ClientConnected(id, user_data) => {
							// `user_data` format:
							// [0-64) -> User profile name
							// [64-72) -> Hashed password as u64
							let allowed = if let Some(phash) = passhash {
								let mut hasher = Sha3_256::new();
								hasher.update(&user_data[64..72]);
								hasher.finalize() == phash
							} else {
								true
							};

							if allowed {
								let usrname = std::str::from_utf8(&user_data[0..64]).expect(
									"A client illegally sent invalid UTF-8 as a user profile name.",
								);

								info!(
									"Connection established.
									Client ID: {}
									Profile name: {}",
									id, usrname
								);
							} else {
								server.disconnect(id);
								info!("Connection refused. Reason: incorrect password.");
							}
						}
						ServerEvent::ClientDisconnected(id) => {
							info!("Client disconnected, ID: {}", id);
						}
					}
				}
			}
		});

		core.terminal.register_command(
			"alias",
			Command {
				flags: CommandFlags::all(),
				func: commands::cmd_alias,
			},
			true,
		);
		core.terminal.register_command(
			"args",
			Command {
				flags: CommandFlags::all(),
				func: commands::cmd_args,
			},
			true,
		);
		core.terminal.register_command(
			"exit",
			Command {
				flags: CommandFlags::all(),
				func: commands::cmd_quit,
			},
			true,
		);
		core.terminal.register_command(
			"help",
			Command {
				flags: CommandFlags::all(),
				func: commands::cmd_help,
			},
			true,
		);
		core.terminal.register_command(
			"home",
			Command {
				flags: CommandFlags::all(),
				func: commands::cmd_home,
			},
			true,
		);
		core.terminal.register_command(
			"quit",
			Command {
				flags: CommandFlags::all(),
				func: commands::cmd_quit,
			},
			true,
		);
		core.terminal.register_command(
			"uptime",
			Command {
				flags: CommandFlags::all(),
				func: commands::cmd_uptime,
			},
			true,
		);
		core.terminal.register_command(
			"version",
			Command {
				flags: CommandFlags::all(),
				func: commands::cmd_version,
			},
			true,
		);

		'term: loop {
			print!("$ ");

			match io::stdout().flush() {
				Ok(()) => {}
				Err(err) => {
					error!("Failed to flush stdout: {}", err);
					return Err(Box::new(err));
				}
			};

			match io::stdin().read_line(&mut cmd_buffer) {
				Ok(stdin) => stdin,
				Err(err) => {
					error!("Failed to read command line: {}", err);
					return Err(Box::new(err));
				}
			};

			let cmd = cmd_buffer.trim();

			for output in core.terminal.submit(cmd) {
				match output {
					CommandRequest::None => {}
					CommandRequest::Callback(func) => {
						(func)(&mut core);
					}
					CommandRequest::Exit => {
						lobby_running.store(false, std::sync::atomic::Ordering::Release);

						match lobby_thread.join() {
							Ok(_) => {}
							Err(err) => {
								println!("Failed to join lobby thread: {:?}", err);
							}
						};

						break 'term;
					}
				}
			}

			cmd_buffer.clear();
		}

		Ok(())
	})
	.expect("Failed to open scope for lobby listening thread.");

	match res {
		Ok(()) => {
			info!("{}", impure::uptime_string(start_time));
			Ok(())
		}
		Err(err) => Err(Box::new(err)),
	}
}
