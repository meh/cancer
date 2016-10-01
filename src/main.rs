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

#![feature(question_mark, mpsc_select)]

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate xdg;
extern crate toml;
extern crate clap;
use clap::{App, Arg, SubCommand, ArgMatches};

extern crate xcb;
extern crate xcb_util as xcbu;
extern crate xkbcommon;
extern crate picto;

extern crate unicode_segmentation;
extern crate unicode_width;

extern crate libc;
extern crate nix;

mod error;
mod ffi;
mod sys;

mod config;
use config::Config;

mod font;
use font::Font;

mod terminal;
use terminal::Terminal;

mod style;
use style::Style;

mod window;
use window::Window;

mod renderer;
use renderer::Renderer;

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
				.help("The X11 display.")
				.takes_value(true)))
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
	let mut font     = Arc::new(Font::load(config.clone())?);
	let mut window   = Window::open(config.clone(), font.clone())?;
	let mut render   = Renderer::new(config.clone(), font.clone(), &window, window.width(), window.height());
	let mut terminal = Terminal::open(config.clone(), render.columns(), render.rows())?;

	let input  = terminal.input();
	let events = window.events();

	loop {
		select! {
			input = input.recv() => {
				// TODO: handle input
			},

			event = events.recv() => {
				let event = event.unwrap();

				match event.response_type() {
					xcb::EXPOSE => {
						let event = xcb::cast_event::<xcb::ExposeEvent>(&event);

						render.update(|mut o| {
							for cell in terminal.area(o.damaged(event.x(), event.y(), event.width(), event.height())) {
								o.cell(cell.x(), cell.y(), cell.as_ref(), cell.style());
							}
						});

						window.flush();
					}

					e => {
						debug!("unhandled event: {:?}", e);
					}
				}
			}
		}
	}

	Ok(())
}
