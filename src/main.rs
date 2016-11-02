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

#![feature(mpsc_select, conservative_impl_trait, slice_patterns, static_in_const)]
#![feature(trace_macros, type_ascription, inclusive_range_syntax, pub_restricted)]
#![feature(deque_extras)]
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
extern crate schedule_recv as timer;
#[macro_use]
extern crate control_code as control;

extern crate xdg;
extern crate toml;
extern crate clap;
use clap::{App, Arg};

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

mod terminal;
use terminal::{Terminal, Action};

mod style;

mod platform;
use platform::Window;

mod renderer;
use renderer::Renderer;

mod tty;
use tty::Tty;

use std::sync::Arc;
use std::io::Write;

fn main() {
	env_logger::init().unwrap();

	let matches = App::new("cancer")
		.version(env!("CARGO_PKG_VERSION"))
		.author("meh. <meh@schizofreni.co>")
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
			.help("Font to use with the terminal."))
		.arg(Arg::with_name("name")
			.short("n")
			.long("name")
			.takes_value(true)
			.help("Name for the window."))
		.arg(Arg::with_name("term")
			.short("t")
			.long("term")
			.takes_value(true).
			help("Specify the TERM environment variable to use."))
		.get_matches();

	let     config   = Arc::new(Config::load(matches.value_of("config")).unwrap());
	let     font     = Arc::new(Font::load(matches.value_of("font").unwrap_or(config.style().font())).unwrap());
	let     batch    = timer::periodic_ms(((1.0 / config.environment().batch() as f32) * 1000.0).round() as u32);
	let     blink    = timer::periodic_ms(config.style().blink());
	let mut blinking = true;
	let mut batched  = false;
	let mut window   = Window::open(matches.value_of("name"), &config, &font).unwrap();
	let mut keyboard = window.keyboard().unwrap();
	let mut renderer = Renderer::new(config.clone(), font.clone(), &window, window.width(), window.height());
	let mut terminal = Terminal::open(config.clone(), renderer.columns(), renderer.rows()).unwrap();
	let mut tty      = Tty::spawn(renderer.columns(), renderer.rows(),
	                              matches.value_of("term").or_else(|| config.environment().term()),
	                              matches.value_of("execute").or_else(|| config.environment().program())).unwrap();

	let input  = tty.output();
	let events = window.events();

	macro_rules! render {
		(resize $width:expr, $height:expr) => ({
			window.resized($width, $height);
			renderer.resize($width, $height);

			let rows    = renderer.rows();
			let columns = renderer.columns();

			if terminal.columns() != columns || terminal.rows() != rows {
				render!(terminal.resize(columns, rows));
				tty.resize(columns, rows).unwrap();
			}
		});

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

		(options!) => ({
			let mut options = render!(options);
			options.insert(renderer::option::DAMAGE);

			options
		});

		(cursor) => ({
			let options = render!(options!);

			// Redraw the cursor.
			renderer.update(|mut o| {
				if terminal.cursor().is_visible() {
					o.cursor(&terminal.cursor(), options);
				}
				else {
					o.cell(&terminal.cursor().cell(), options);
				}
			});

			window.flush();
		});

		(damaged $area:expr) => ({
			let area    = $area;
			let options = render!(options!);

			renderer.update(|mut o| {
				// Redraw margins.
				o.margin(&area);

				// Redraw the cells that fall within the damaged area.
				for cell in terminal.iter(o.damaged(&area).relative()) {
					o.cell(&cell, options);
				}

				// Redraw the cursor.
				if terminal.cursor().is_visible() {
					o.cursor(&terminal.cursor(), options);
				}
				else {
					o.cell(&terminal.cursor().cell(), options);
				}
			});

			window.flush();
		});

		(batched $iter:expr) => ({
			let iter = $iter;

			if iter.all() {
				batched = true;
			}

			if !batched {
				render!(iter);
			}
		});

		($iter:expr) => ({
			let iter    = $iter;
			let options = render!(options);

			renderer.update(|mut o| {
				for cell in terminal.iter(iter) {
					o.cell(&cell, options);
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
			_ = blink.recv() => {
				blinking = !blinking;
				render!(batched terminal.blinking(blinking));
			},

			_ = batch.recv() => {
				if batched {
					render!(terminal.area().absolute());
					batched = false;
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
							render!(resize width, height);
						}
					}

					// Handle keyboard.
					e if keyboard.owns_event(e) => {
						keyboard.handle(&event);
					}

					xcb::KEY_PRESS => {
						let event = xcb::cast_event::<xcb::KeyPressEvent>(&event);

						if let Some(key) = keyboard.key(event.detail()) {
							terminal.key(key, &mut tty).unwrap();
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
				let (actions, touched) = try!(continue terminal.handle(&input, tty.by_ref()));
				render!(batched touched);

				for action in actions {
					match action {
						Action::Title(string) => {
							window.set_title(&string);
						}

						Action::Resize(columns, rows) => {
							let (width, height) = Renderer::dimensions(columns, rows, &config, &font);
							window.resize(width, height);
							render!(resize width, height);
						}

						Action::Clipboard(name, value) => {
							// TODO: handle clipboard setting
						}
					}
				}

				try!(break tty.flush());
			}
		}
	}
}
