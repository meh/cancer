// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// This file is part of cancer.
//
// cancer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cancer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cancer.  If not, see <http://www.gnu.org/licenses/>.

#![feature(question_mark, mpsc_select, conservative_impl_trait)]

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate bitflags;
extern crate fnv;
extern crate shlex;

extern crate xdg;
extern crate toml;
extern crate clap;
use clap::{App, Arg, SubCommand, ArgMatches};

extern crate libc;
extern crate xcb;
extern crate xcb_util as xcbu;
extern crate xkbcommon;

extern crate picto;
use picto::Area;

extern crate unicode_segmentation;
extern crate unicode_width;

mod error;
mod ffi;
mod sys;

mod config;
use config::Config;

mod font;
use font::Font;

mod timer;
use timer::Timer;

mod terminal;
use terminal::Terminal;

mod style;

mod platform;
use platform::Window;

mod renderer;
use renderer::Renderer;

mod tty;
use tty::Tty;

fn main() {
	env_logger::init().unwrap();

	let matches = App::new("cancer")
		.version(env!("CARGO_PKG_VERSION"))
		.author("meh. <meh@schizofreni.co>")
		.subcommand(SubCommand::with_name("open")
			.about("Open the terminal in a window.")
			.arg(Arg::with_name("config")
				.short("c")
				.long("config")
				.help("The path to the configuration file.")
				.takes_value(true))
			.arg(Arg::with_name("display")
				.short("d")
				.long("display")
				.takes_value(true)
				.help("The X11 display."))
			.arg(Arg::with_name("execute")
				.short("e")
				.long("execute")
				.takes_value(true)
				.help("Program to execute.")))
		.get_matches();

	match matches.subcommand() {
		("open", Some(matches)) =>
			open(matches).unwrap(),

		_ => ()
	}
}

fn open(matches: &ArgMatches) -> error::Result<()> {
	use std::sync::Arc;

	let     config   = Arc::new(Config::load(matches.value_of("config"))?);
	let     font     = Arc::new(Font::load(config.clone())?);
	let     timer    = Timer::spawn(config.clone());
	let mut window   = Window::open(config.clone(), font.clone())?;
	let mut keyboard = window.keyboard()?;
	let mut render   = Renderer::new(config.clone(), font.clone(), &window, window.width(), window.height());
	let mut terminal = Terminal::open(config.clone(), render.columns(), render.rows())?;
	let mut tty      = Tty::spawn(config.clone(), matches.value_of("execute").map(|s| s.into()), render.columns(), render.rows())?;

	let output = tty.output();
	let events = window.events();

	loop {
		select! {
			timer = timer.recv() => {
				match timer.unwrap() {
					// Handle blinking.
					timer::Event::Blink(blinking) => {
						// Redraw the cells that blink and the cursor.
						render.update(|mut o| {
							for cell in terminal.blinking(blinking) {
								o.cell(cell, blinking);
							}

							o.cursor(terminal.cursor(), blinking, window.has_focus());
						});

						window.flush();
					}
				}
			},

			event = events.recv() => {
				let event = event.unwrap();

				match event.response_type() {
					// Redraw the areas that have been damaged.
					xcb::EXPOSE => {
						let event  = xcb::cast_event::<xcb::ExposeEvent>(&event);
						let area   = Area::from(event.x() as u32, event.y() as u32,
							event.width() as u32, event.height() as u32);

						render.update(|mut o| {
							// Redraw margins.
							o.margin(&area);

							// Redraw the cells that fall within the damaged area.
							for cell in terminal.area(o.damaged(&area)) {
								o.cell(cell, terminal.is_blinking());
							}

							// Redraw the cursor.
							o.cursor(terminal.cursor(), terminal.is_blinking(), window.has_focus());
						});

						window.flush();
					}

					// Handle focus changes.
					xcb::FOCUS_IN | xcb::FOCUS_OUT => {
						window.focus(event.response_type() == xcb::FOCUS_IN);

						// Redraw the cursor.
						render.update(|mut o| {
							o.cursor(terminal.cursor(), terminal.is_blinking(), window.has_focus());
						});

						window.flush();
					}

					// Handle resizes.
					xcb::CONFIGURE_NOTIFY => {
						let event  = xcb::cast_event::<xcb::ConfigureNotifyEvent>(&event);
						let width  = event.width() as u32;
						let height = event.height() as u32;

						if window.width() != width || window.height() != height {
							window.resized(width, height);
							render.resize(width, height);

							let rows    = render.rows();
							let columns = render.columns();

							if terminal.columns() != columns || terminal.rows() != rows {
								terminal.resize(columns, rows);
							}
						}
					}

					// Handle keyboard.
					e if keyboard.owns_event(e) => {
						keyboard.handle(&event);
					}

					xcb::KEY_PRESS => {
						let event = xcb::cast_event::<xcb::KeyPressEvent>(&event);

						match keyboard.symbol(event.detail()) {
							sym => {
								println!("key: {:?} {:?}", sym, keyboard.string(event.detail()));
							}
						}
					}

					_ => ()
				}
			},

			output = output.recv() => {
				// TODO: handle output
			}
		}
	}
}
