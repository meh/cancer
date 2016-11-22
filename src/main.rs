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

extern crate regex;
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

mod style;
mod renderer;

mod terminal;
mod interface;
mod overlay;

mod runner;
mod platform;
use platform::Window;

fn main() {
	use std::sync::Arc;
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
		.arg(Arg::with_name("tic")
			.short("T")
			.long("tic")
			.help("Print the terminfo database to stdout and exit."))
		.get_matches();

	if matches.is_present("tic") {
		print!("{}", include_str!("../assets/cancer.info"));
		return;
	}

	let     config = Arc::new(Config::load(matches.value_of("config")).unwrap());
	let     font   = Arc::new(Font::load(matches.value_of("font").unwrap_or(config.style().font())).unwrap());
	let mut window = Window::new(matches.value_of("name"), config.clone(), font.clone()).unwrap();

	let runner = runner::spawn(&matches, config.clone(), font.clone(), window.proxy()).unwrap();
	window.run(runner).unwrap();
}
