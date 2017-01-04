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
#![feature(deque_extras, box_syntax, try_from, alloc_system)]

#![cfg_attr(feature = "fuzzy", feature(plugin))]
#![cfg_attr(feature = "fuzzy", plugin(afl_plugin))]
#[cfg(feature = "fuzzy")]
extern crate afl;

extern crate alloc_system;

#[macro_use(error, debug, log)]
extern crate log;
extern crate env_logger;

#[macro_use(bitflags)]
extern crate bitflags;
extern crate bit_vec;
extern crate fnv;
extern crate itertools;
extern crate lru_cache as lru;
extern crate shlex;
extern crate schedule_recv as timer;
extern crate picto;
#[macro_use(arg)]
extern crate control_code as control;

extern crate unicode_segmentation;
extern crate unicode_width;

extern crate regex;
extern crate app_dirs;
extern crate toml;
extern crate clap;
use clap::{App, Arg, ArgMatches};

extern crate libc;

#[cfg(all(feature = "x11", unix))]
pub extern crate xcb;
#[cfg(all(feature = "x11", unix))]
pub extern crate xcb_util as xcbu;
#[cfg(all(feature = "x11", unix))]
pub extern crate xkb;

#[cfg(all(feature = "quartz", target_os = "macos"))]
#[macro_use(msg_send, sel)]
pub extern crate objc;
#[cfg(all(feature = "quartz", target_os = "macos"))]
pub extern crate cocoa;
#[cfg(all(feature = "quartz", target_os = "macos"))]
pub extern crate core_foundation;
#[cfg(all(feature = "quartz", target_os = "macos"))]
pub extern crate core_graphics;

#[macro_use]
mod util;
mod error;
mod ffi;
mod sys;

mod config;
mod font;
mod style;

mod platform;
mod renderer;

mod interface;
mod terminal;
mod overlay;

#[cfg(not(feature = "fuzzy"))]
fn main() {
	use std::sync::Arc;
	use std::sync::mpsc::{Sender, channel};
	use std::iter;
	use std::mem;
	use std::io::Write;
	use std::thread;

	use picto::Region;
	use config::Config;
	use font::Font;
	use renderer::Renderer;
	use interface::{Interface, Action};
	use terminal::Terminal;
	use overlay::Overlay;
	use platform::{Window, Tty, Event, Proxy};
	use platform::mouse::{self, Mouse};

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
			.takes_value(true)
			.help("Specify the TERM environment variable to use."))
		.arg(Arg::with_name("title")
			.long("title")
			.takes_value(true)
			.help("Specify the window title."))
		.arg(Arg::with_name("tic")
			.short("T")
			.long("tic")
			.help("Print the terminfo database to stdout and exit."))
		.get_matches();

	if matches.is_present("tic") {
		print!("{}", include_str!("../assets/cancer.info"));
		return;
	}

	let config = Arc::new(Config::load(matches.value_of("config")).unwrap());
	let font   = Arc::new(Font::load(matches.value_of("font").unwrap_or(config.style().font())).unwrap());

	let mut window = Window::new(matches.value_of("name"), config.clone(), font.clone()).unwrap();
	let     proxy  = window.proxy();

	if let Some(title) = matches.value_of("title") {
		proxy.set_title(title.into());
	}

	let _ = window.run(spawn(&matches, config.clone(), font.clone(), proxy).unwrap());

	fn spawn<W: platform::Proxy + 'static>(matches: &ArgMatches, config: Arc<Config>, font: Arc<Font>, mut window: W) -> error::Result<Sender<Event>> {
		let (sender, events) = channel();
		window.prepare(sender.clone());

		let mut surface = window.surface().unwrap();
		let     (w, h)  = window.dimensions();

		let mut renderer = Renderer::new(config.clone(), font.clone(), &surface, w, h);

		let mut interface = Interface::from(Terminal::new(config.clone(),
			(font.width(), font.height() + config.style().spacing()),
			(renderer.columns(), renderer.rows()))?);

		let mut tty = Tty::spawn(
			matches.value_of("term").or_else(|| config.environment().term()),
			matches.value_of("execute").or_else(|| config.environment().program()),
			(font.width(), font.height() + config.style().spacing()),
			(renderer.columns(), renderer.rows()))?;

		let mut focused = true;
		let mut visible = true;

		let     blink    = timer::periodic_ms(config.style().blink());
		let mut blinking = true;

		let (_batcher, mut batch) = channel();
		let mut batching          = None;
		let mut batched           = None;

		let input = tty.output();

		macro_rules! render {
			(options) => ({
				let mut options = renderer::Options::empty();

				if interface.mode().contains(terminal::mode::BLINK) {
					options.insert(renderer::option::BLINKING);
				}

				if focused {
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

			(handle $what:expr) => ({
				let (actions, touched) = try!(continue $what);

				if touched.is_total() && batched.is_none() && config.environment().batch().is_some() {
					batching = Some(true);
				}
				else if batched.is_none() && !touched.is_empty() {
					render!(touched);
				}

				for action in actions {
					match action {
						Action::Urgent => {
							window.urgent();
						}

						Action::Overlay(true) => {
							interface = Overlay::new(try!(return interface.into_inner(tty.by_ref()))).into();
							render!(interface.region().absolute());
						}

						Action::Overlay(false) => {
							interface = try!(return interface.into_inner(tty.by_ref())).into();
							render!(interface.region().absolute());
						}

						Action::Title(string) => {
							window.set_title(string);
						}

						Action::Resize(columns, rows) => {
							let (width, height) = Renderer::dimensions(columns, rows, &config, &font);
							window.resize(width, height);
						}

						Action::Copy(name, value) => {
							window.copy(name, value);
						}

						Action::Paste(name) => {
							window.paste(name)
						}

						Action::Open(through, what) => {
							window.open(through.as_ref().map(AsRef::as_ref), what.as_ref()).unwrap();
						}
					}
				}

				try!(return tty.flush());
			});

			($iter:expr) => ({
				let iter = $iter;

				if visible {
					window.render(&mut surface, ||
						renderer.render(render!(options), None, &interface, iter));
				}
			});
		}

		thread::spawn(move || {
			let _batcher = _batcher;

			loop {
				match batching.take() {
					Some(true) => {
						batched = Some(mem::replace(&mut batch,
							timer::oneshot_ms(config.environment().batch().unwrap())));
					}

					Some(false) => {
						if let Some(empty) = batched.take() {
							batch = empty;
							render!(interface.region().absolute());
						}
					}

					None => ()
				}

				select! {
					_ = batch.recv() => {
						batching = Some(false);
					},

					_ = blink.recv() => {
						blinking = !blinking;

						let blinked = interface.blinking(blinking);
						if (!blinked.is_empty() || interface.cursor().blink()) && batched.is_none() {
							render!(blinked);
						}
					},

					event = events.recv() => {
						let event = try!(return event);
						debug!(target: "cancer::runner", "{:?}", event);

						match event {
							Event::Closed => return,

							Event::Show(value) => {
								visible = value;
							}

							Event::Redraw => {
								let width   = renderer.width();
								let height  = renderer.height();
								let rows    = renderer.rows();
								let columns = renderer.columns();

								window.render(&mut surface, ||
									renderer.render(render!(options), Some(Region::from(0, 0, width, height)),
										&interface, Region::from(0, 0, columns, rows).absolute()));
							}

							Event::Damaged(region) => {
								let damaged = renderer.damaged(&region);

								window.render(&mut surface, ||
									renderer.render(render!(options), Some(region), &interface, damaged.relative()));
							}

							Event::Focus(focus) => {
								focused = focus;
								try!(return interface.focus(focus, tty.by_ref()));
								render!(iter::empty());
							}

							Event::Resize(width, height) => {
								if renderer.width() == width && renderer.height() == height {
									continue;
								}

								if interface.overlay() {
									interface = try!(return interface.into_inner(tty.by_ref())).into();
								}

								surface = window.surface().unwrap();
								renderer.resize(&surface, width, height);

								let rows    = renderer.rows();
								let columns = renderer.columns();

								if interface.columns() != columns || interface.rows() != rows {
									try!(return tty.resize(columns, rows));
									interface.resize(columns, rows);
								}
							}

							Event::Paste(value) => {
								try!(return interface.paste(&value, tty.by_ref()));
								try!(return tty.flush());
							}

							Event::Key(key) => {
								render!(handle interface.key(key, tty.by_ref()));
							}

							Event::Mouse(mut event) => {
								match event {
									Mouse::Click(mouse::Click { ref mut position, .. }) |
									Mouse::Motion(mouse::Motion { ref mut position, .. }) => {
										if let Some((x, y)) = renderer.position(position.x, position.y) {
											position.x = x;
											position.y = y;
										}
										else {
											continue;
										}
									}
								}

								render!(handle interface.mouse(event, tty.by_ref()));
							}
						}
					},

					input = input.recv() => {
						render!(handle interface.input(&try!(return input), tty.by_ref()));
					}
				}
			}
		});

		Ok(sender)
	}
}

#[cfg(feature = "fuzzy")]
fn main() {
	use std::sync::Arc;
	use std::fs::File;
	use std::io::{Cursor, Read};
	use std::panic::UnwindSafe;

	use config::Config;
	use terminal::Terminal;

	env_logger::init().unwrap();

	let matches = App::new("cancer")
		.version(env!("CARGO_PKG_VERSION"))
		.author("meh. <meh@schizofreni.co>")
		.arg(Arg::with_name("test")
			.short("T")
			.long("test")
			.takes_value(true)
			.help("Test a crasher."))
		.get_matches();

	let mut terminal = Terminal::new(Arc::new(Config::default()),
		(6, 11), (80, 24)).unwrap();

	if matches.is_present("test") {
		let mut content = Vec::new();
		let mut file    = File::open(matches.value_of("test").unwrap()).expect("cannot open crasher");
		file.read_to_end(&mut content).unwrap();

		terminal.input(&content, Cursor::new(vec![])).unwrap();
	}
	else {
		unsafe { afl::init() }

		impl UnwindSafe for Terminal { }

		afl::handle_bytes(move |input| {
			terminal.input(&input, Cursor::new(vec![])).unwrap();
		});
	}
}
