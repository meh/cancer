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
#![feature(deque_extras, integer_atomics)]
#![recursion_limit="100"]

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate bitflags;
extern crate bit_vec;
extern crate fnv;
extern crate itertools;
extern crate lru_cache as lru;
extern crate shlex;
extern crate picto;
extern crate schedule_recv as timer;
#[macro_use]
extern crate control_code as control;

extern crate unicode_segmentation;
extern crate unicode_width;

extern crate app_dirs;
extern crate toml;
extern crate clap;
use clap::{App, Arg};

extern crate libc;

#[cfg(target_os = "linux")]
extern crate xcb;
#[cfg(target_os = "linux")]
extern crate xcb_util as xcbu;
#[cfg(target_os = "linux")]
extern crate xkbcommon;

#[cfg(target_os = "macos")]
extern crate cocoa;

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

mod interface;
pub use interface::Interface;

mod overlay;
pub use overlay::Overlay;

mod style;

mod platform;
use platform::{Event, Window, Tty, key};

mod renderer;
use renderer::Renderer;

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

	let     config    = Arc::new(Config::load(matches.value_of("config")).unwrap());
	let     font      = Arc::new(Font::load(matches.value_of("font").unwrap_or(config.style().font())).unwrap());
	let     batch     = timer::periodic_ms(((1.0 / config.environment().batch() as f32) * 1000.0).round() as u32);
	let     blink     = timer::periodic_ms(config.style().blink());
	let mut blinking  = true;
	let mut batched   = false;
	let mut window    = Window::open(matches.value_of("name"), &config, &font).unwrap();
	let mut surface   = window.surface();
	let mut renderer  = Renderer::new(config.clone(), font.clone(), &surface, window.width(), window.height());
	let mut interface = Interface::from(Terminal::open(config.clone(), renderer.columns(), renderer.rows()).unwrap());
	let mut tty       = Tty::spawn(renderer.columns(), renderer.rows(),
	                               matches.value_of("term").or_else(|| config.environment().term()),
	                               matches.value_of("execute").or_else(|| config.environment().program())).unwrap();

	let input  = tty.output();
	let events = window.events();

	macro_rules! render {
		(options) => ({
			let mut options = renderer::Options::empty();

			if interface.mode().contains(terminal::mode::BLINK) {
				options.insert(renderer::option::BLINKING);
			}

			if window.has_focus() {
				options.insert(renderer::option::FOCUS);
			}

			if interface.mode().contains(terminal::mode::REVERSE) {
				options.insert(renderer::option::REVERSE);
			}

			if interface.cursor().is_visible() {
				options.insert(renderer::option::CURSOR);
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
				if options.cursor() {
					o.cursor(&interface.cursor(), options);
				}
				else {
					o.cell(&interface.cursor().cell(), options);
				}
			});

			surface.flush();
			window.flush();
		});

		(handle $what:expr) => ({
			let (actions, touched) = try!(continue $what);

			if touched.all() {
				batched = true;
			}
			else if !batched {
				render!(touched);
			}

			for action in actions {
				match action {
					Action::Title(string) => {
						window.set_title(string);
					}

					Action::Resize(columns, rows) => {
						let (width, height) = Renderer::dimensions(columns, rows, &config, &font);
						window.resize(width, height);
					}

					Action::Clipboard(name, value) => {
						window.clipboard(name, value);
					}
				}
			}

			try!(break tty.flush());
		});

		($iter:expr) => ({
			let iter    = $iter;
			let options = render!(options);

			renderer.update(|mut o| {
				for cell in interface.iter(iter) {
					o.cell(&cell, options);
				}

				if options.cursor() {
					o.cursor(&interface.cursor(), options);
				}
			});

			surface.flush();
			window.flush();
		});
	}

	loop {
		select! {
			_ = blink.recv() => {
				blinking = !blinking;
				render!(interface.blinking(blinking));
			},

			_ = batch.recv() => {
				if batched {
					render!(interface.region().absolute());
					batched = false;
				}
			},

			event = events.recv() => {
				match try!(break event) {
					Event::Redraw(region) => {
						let options = render!(options!);
			
						renderer.update(|mut o| {
							// Redraw margins.
							o.margin(&region);
			
							// Redraw the cells that fall within the damaged region.
							for cell in interface.iter(o.damaged(&region).relative()) {
								o.cell(&cell, options);
							}
			
							// Redraw the cursor.
							if options.cursor() {
								o.cursor(&interface.cursor(), options);
							}
							else {
								o.cell(&interface.cursor().cell(), options);
							}
						});
			
						surface.flush();
						window.flush();
					}

					Event::Focus(_) => {
						render!(cursor);
					}

					Event::Resize(width, height) => {
						if interface.overlay() {
							interface = try!(break interface.into_inner(&mut tty)).into();
						}

						renderer.resize(width, height);
						surface.resize(width, height);

						let rows    = renderer.rows();
						let columns = renderer.columns();

						if interface.columns() != columns || interface.rows() != rows {
							try!(break tty.resize(columns, rows));
							interface.resize(columns, rows);
						}
					}

					Event::Key(key) => {
						if interface.overlay() &&
						   ((key.value() == &key::Value::Button(key::Button::Escape) && key.modifier().is_empty()) ||
						    (key.value() == &key::Value::Char("c".into()) && key.modifier() == key::CTRL) ||
							  (key.value() == &key::Value::Char("i".into()) && key.modifier().is_empty()))
						{
							interface = try!(break interface.into_inner(&mut tty)).into();
							render!(interface.region().absolute());
							continue;
						}

						if !interface.overlay() && &key == config.input().prefix() {
							interface = Overlay::new(try!(break interface.into_inner(&mut tty))).into();
							render!(interface.region().absolute());
							continue;
						}

						render!(handle interface.key(key, tty.by_ref()));
					}
				}
			},

			input = input.recv() => {
				let input = try!(break input);
				render!(handle interface.handle(&input, tty.by_ref()));
			}
		}
	}
}
