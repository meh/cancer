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
#![feature(deque_extras, box_syntax, try_from)]

#![feature(plugin)]
#![plugin(afl_plugin)]
extern crate afl;

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
extern crate palette;
extern crate schedule_recv as timer;
#[macro_use]
extern crate control_code as control;

extern crate unicode_segmentation;
extern crate unicode_width;

extern crate regex;
extern crate app_dirs;
extern crate toml;
extern crate clap;
use clap::{App, Arg, ArgMatches};

extern crate libc;

#[cfg(target_os = "linux")]
extern crate xcb;
#[cfg(target_os = "linux")]
extern crate xcb_util as xcbu;
#[cfg(target_os = "linux")]
extern crate xkbcommon;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;
#[cfg(target_os = "macos")]
extern crate cocoa;
#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
extern crate core_graphics;

#[macro_use]
mod util;
use util::Region;

mod error;
mod ffi;
mod sys;

mod config;
use config::Config;

mod font;
use font::Font;

mod style;

mod interface;
mod overlay;
mod terminal;
use terminal::Terminal;

mod platform;

use std::sync::Arc;
use std::fs::File;
use std::io::{Cursor, Read};
use std::panic::UnwindSafe;

impl UnwindSafe for Terminal { }

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
		.arg(Arg::with_name("font")
			.short("f")
			.long("font")
			.takes_value(true)
			.help("Font to use with the terminal."))
		.arg(Arg::with_name("term")
			.short("t")
			.long("term")
			.takes_value(true).
			help("Specify the TERM environment variable to use."))
		.arg(Arg::with_name("test")
			.short("T")
			.long("test")
			.takes_value(true)
			.help("Test a crasher."))
		.get_matches();

	let mut terminal = Terminal::new(Arc::new(Config::load(matches.value_of("config")).unwrap()), 80, 24).unwrap();

	if matches.is_present("test") {
		let mut content = Vec::new();
		let mut file    = File::open(matches.value_of("test").unwrap()).expect("cannot open crasher");
		file.read_to_end(&mut content).unwrap();

		terminal.input(&content, Cursor::new(vec![])).unwrap();
	}
	else {
		unsafe { afl::init() }

		afl::handle_bytes(move |input| {
			terminal.input(&input, Cursor::new(vec![])).unwrap();
		});
	}
}
