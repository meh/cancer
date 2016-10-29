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

#![feature(question_mark, mpsc_select, conservative_impl_trait, slice_patterns)]
#![feature(static_in_const, trace_macros, type_ascription, inclusive_range_syntax)]
#![feature(pub_restricted, deque_extras)]
#![recursion_limit="100"]

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate bitflags;
extern crate bit_vec;
extern crate fnv;
extern crate lru_cache as lru;
extern crate shlex;
#[macro_use]
extern crate control_code as control;

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

#[macro_use]
mod util;
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
use terminal::{Terminal, Action};

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
				.help("Program to execute."))
			.arg(Arg::with_name("font")
				.short("f")
				.long("font")
				.takes_value(true)
				.help("Font to use with the terminal.")))
		.get_matches();

	match matches.subcommand() {
		("open", Some(matches)) =>
			open(matches).unwrap(),

		_ => ()
	}
}

fn open(matches: &ArgMatches) -> error::Result<()> {
	use std::sync::Arc;
	use std::io::Write;

	let     config   = Arc::new(Config::load(matches.value_of("config"))?);
	let     font     = Arc::new(Font::load(matches.value_of("font").unwrap_or(config.style().font()))?);
	let     timer    = Timer::spawn(config.clone());
	let mut window   = Window::open(config.clone(), font.clone())?;
	let mut keyboard = window.keyboard()?;
	let mut render   = Renderer::new(config.clone(), font.clone(), &window, window.width(), window.height());
	let mut terminal = Terminal::open(config.clone(), render.columns(), render.rows())?;
	let mut tty      = Tty::spawn(config.clone(), matches.value_of("execute").map(|s| s.into()), render.columns(), render.rows())?;

	let input  = tty.output();
	let events = window.events();

	macro_rules! render {
		(options) => ({
			let mut options = renderer::Options::empty();

			if terminal.mode().contains(terminal::mode::BLINK) {
				options.insert(renderer::option::BLINKING);
			}

			if window.has_focus() {
				options.insert(renderer::option::FOCUS);
			}

			if terminal.mode().contains(terminal::mode::REVERSE) {
				options.insert(renderer::option::REVERSE);
			}

			options
		});

		(cursor) => ({
			let options = render!(options);

			// Redraw the cursor.
			render.update(|mut o| {
				if terminal.cursor().is_visible() {
					o.cursor(&terminal.cursor(), options);
				}
				else {
					o.cell(&terminal.cursor().cell(), options, true);
				}
			});

			window.flush();
		});

		(damaged $area:expr) => ({
			let options = render!(options);
			let area    = $area;

			render.update(|mut o| {
				// Redraw margins.
				o.margin(&area);

				// Redraw the cells that fall within the damaged area.
				for cell in terminal.iter(o.damaged(&area).relative()) {
					o.cell(&cell, options, true);
				}

				// Redraw the cursor.
				if terminal.cursor().is_visible() {
					o.cursor(&terminal.cursor(), options);
				}
				else {
					o.cell(&terminal.cursor().cell(), options, true);
				}
			});

			window.flush();
		});

		($iter:expr) => ({
			let options = render!(options);
			let iter    = $iter;

			render.update(|mut o| {
				for cell in terminal.iter(iter) {
					o.cell(&cell, options, false);
				}

				if terminal.cursor().is_visible() {
					o.cursor(&terminal.cursor(), options);
				}
			});

			window.flush();
		});
	}

	loop {
		select! {
			timer = timer.recv() => {
				match timer.unwrap() {
					// Handle blinking.
					timer::Event::Blink(blinking) => {
						render!(terminal.blinking(blinking));
					}
				}
			},

			event = events.recv() => {
				let event = event.unwrap();

				match event.response_type() {
					// Redraw the areas that have been damaged.
					xcb::EXPOSE => {
						let event = xcb::cast_event::<xcb::ExposeEvent>(&event);

						render!(damaged Area::from(event.x() as u32, event.y() as u32,
							event.width() as u32, event.height() as u32));
					}

					// Handle focus changes.
					xcb::FOCUS_IN | xcb::FOCUS_OUT => {
						window.focus(event.response_type() == xcb::FOCUS_IN);
						render!(cursor);
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
								render!(terminal.resize(columns, rows));
								tty.resize(columns, rows).unwrap();
							}
						}
					}

					// Handle keyboard.
					e if keyboard.owns_event(e) => {
						keyboard.handle(&event);
					}

					xcb::KEY_PRESS => {
						let event = xcb::cast_event::<xcb::KeyPressEvent>(&event);

						if let Some(key) = keyboard.key(event.detail()) {
							render!(terminal.key(key, &mut tty).unwrap());
							tty.flush().unwrap();
						}
					}

					e => {
						debug!(target: "cancer::x11", "unhandled X event: {:?}", e);
					}
				}
			},

			input = input.recv() => {
				let input = try!(break input);

				{
					let (actions, touched) = try!(continue terminal.handle(&input, &mut tty));
					render!(touched);

					for action in actions {
						match action {
							Action::Title(string) => {
								window.set_title(&string);
							}
						}
					}
				}

				try!(break tty.flush());
			}
		}
	}

	Ok(())
}
