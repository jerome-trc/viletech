//! # VileTech Client

mod commands;
#[allow(dead_code)]
mod core;

use std::{boxed::Box, env, error::Error, time::Instant};

use clap::Parser;
use indoc::printdoc;
use log::{error, info, warn};
use vile::{
	console::Console,
	data::{Catalog, CatalogExt},
	gfx::{core::GraphicsCore, render},
	utils::duration_to_hhmmss,
};
use winit::{
	dpi::PhysicalSize,
	event::{Event as WinitEvent, VirtualKeyCode, WindowEvent},
	event_loop::EventLoop,
};

use crate::core::ClientCore;

fn main() -> Result<(), Box<dyn Error>> {
	let start_time = Instant::now();
	let args = Clap::parse();

	if args.version {
		println!("{}", vile::short_version_string());
		println!("{}", &version_string());
		return Ok(());
	}

	if args.about {
		printdoc! {"
VileTech Client - Copyright (C) 2022-2023 - ***REMOVED***

This program comes with ABSOLUTELY NO WARRANTY.

This is free software, and you are welcome to redistribute it under certain
conditions. See the license document that come with your installation."
		};

		return Ok(());
	}

	vile::thread_pool_init(args.threads);

	let (log_sender, log_receiver) = crossbeam::channel::unbounded();

	match vile::log_init(Some(log_sender)) {
		Ok(()) => {}
		Err(err) => {
			eprintln!("Failed to initialise logging backend: {err}");
			return Err(err);
		}
	}

	let console = Console::new(log_receiver);

	vile::log_init_diag(&version_string())?;

	let _devmode = env::args().any(|arg| arg == "-d" || arg == "--dev");

	let mut catalog = Catalog::default();

	if let Err(err) = catalog.mount_basedata() {
		error!("Failed to find and mount engine base data: {err}");
		return Err(Box::new(err));
	}

	let event_loop = EventLoop::new();

	let window = match winit::window::WindowBuilder::new()
		.with_title("VileTech")
		.with_min_inner_size(PhysicalSize::new(320, 200))
		.with_max_inner_size(PhysicalSize::new(7680, 4320))
		.with_inner_size(PhysicalSize::new(800, 600))
		.with_decorations(true)
		.with_resizable(true)
		.with_transparent(false)
		.with_window_icon(
			catalog
				.window_icon_from_file("/viletech/viletech.png")
				.map_err(|err| {
					warn!("Failed to load engine's window icon: {err}");
					err
				})
				.ok(),
		)
		.build(&event_loop)
	{
		Ok(w) => w,
		Err(err) => {
			return Err(Box::new(err));
		}
	};

	let mut gfx = match GraphicsCore::new(window, &event_loop) {
		Ok(g) => g,
		Err(err) => {
			error!("Graphics subsystem initialisation failed: {}", err);
			return Err(err);
		}
	};

	let shader = vile::gfx::create_shader_module(
		&gfx.device,
		"hello-tri",
		catalog
			.get_file("/viletech/shaders/hello-tri.wgsl")
			.expect("Engine base data validity check is compromised.")
			.try_read_str()?,
	);

	let pipeline = render::pipeline_builder("Hello Triangle", &gfx.device)
		.shader_states(vile::gfx::create_shader_states(
			&shader,
			"vs_main",
			&[],
			"fs_main",
			&[Some(wgpu::ColorTargetState {
				format: gfx.surface_config.format,
				blend: Some(wgpu::BlendState::REPLACE),
				write_mask: wgpu::ColorWrites::ALL,
			})],
		))
		// Remaining defaults are acceptable.
		.build();

	gfx.pipelines.push(pipeline);

	let mut core = match ClientCore::new(start_time, catalog, gfx, console) {
		Ok(c) => c,
		Err(err) => {
			eprintln!("Client init failed: {err}");
			return Err(err);
		}
	};

	event_loop.run(move |event, _, control_flow| match event {
		WinitEvent::RedrawRequested(window_id) => {
			core.redraw_requested(window_id, control_flow);
		}
		WinitEvent::MainEventsCleared => {
			core.main_events_cleared(control_flow);
		}
		WinitEvent::WindowEvent {
			ref event,
			window_id,
		} if window_id == core.gfx.window.id() => {
			let resp = core.gfx.egui.state.on_event(&core.gfx.egui.context, event);

			match event {
				WindowEvent::CloseRequested => {
					core.exit();
					core.scene_change(control_flow);
				}
				WindowEvent::Resized(psize) => {
					core.on_window_resize(*psize);
				}
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					core.on_window_resize(**new_inner_size);
				}
				WindowEvent::Focused(gained) => {
					if *gained {
						core.audio.resume_all();
					} else {
						core.audio.pause_all();
					}
				}
				_ => {}
			}

			if resp.consumed {
				return;
			}

			match event {
				WindowEvent::KeyboardInput { input, .. } => {
					if input.state == winit::event::ElementState::Pressed
						&& input.virtual_keycode == Some(VirtualKeyCode::Escape)
					{
						core.exit();
						core.scene_change(control_flow);
						return;
					}

					core.on_key_event(input);
				}
				WindowEvent::MouseInput { state, button, .. } => {
					core.input.on_mouse_input(button, state);
				}
				WindowEvent::ModifiersChanged(state) => {
					core.input.on_modifiers_changed(state);
				}
				WindowEvent::CursorMoved { position, .. } => {
					core.input.on_cursor_moved(position);
				}
				_ => {}
			}
		}
		WinitEvent::Suspended => {
			info!("Application suspended...");
		}
		WinitEvent::Resumed => {
			info!("Application resumed...");
		}
		WinitEvent::LoopDestroyed => {
			let uptime = core.start_time.elapsed();
			let (hh, mm, ss) = duration_to_hhmmss(uptime);
			info!("Uptime: {hh:02}:{mm:02}:{ss:02}");
		}
		_ => {}
	});
}

#[must_use]
fn version_string() -> String {
	format!("VileTech Client {}", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug, clap::Parser)]
struct Clap {
	/// Prints the client and engine versions.
	#[arg(short = 'V', long = "version")]
	version: bool,
	/// Prints license information.
	#[arg(short = 'A', long = "about")]
	about: bool,
	/// Sets the number of threads used by the global thread pool
	///
	/// If set to 0 or not set, this will be automatically selected based on the
	/// number of logical CPUs your computer has.
	#[arg(short, long)]
	threads: Option<usize>,
}
