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

use std::rc::Rc;
use std::sync::Arc;
use std::io::Write;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use picto::Area;
use picto::color::Rgba;
use nom::IResult;
use error::{self, Error};
use config::Config;
use style::{self, Style};
use terminal::{Cell, Key, cell, iter};
use control::{self, C0, C1, CSI, SGR, Format};

#[derive(Debug)]
pub struct Terminal {
	config: Arc<Config>,
	area:   Area,

	style:    Rc<Style>,
	cells:    Vec<Cell>,
	cursor:   (u32, u32),
	blinking: bool,
}

impl Terminal {
	pub fn open(config: Arc<Config>, width: u32, height: u32) -> error::Result<Self> {
		let area  = Area::from(0, 0, width, height);
		let style = Rc::new(Style::default());
		let cells = area.absolute().map(|(x, y)| Cell::new(x, y, style.clone()));

		Ok(Terminal {
			config: config,
			area:   area,

			style:    style.clone(),
			cells:    cells.collect(),
			cursor:   (0, 0),
			blinking: false,
		})
	}

	pub fn columns(&self) -> u32 {
		self.area.width
	}

	pub fn rows(&self) -> u32 {
		self.area.height
	}

	pub fn is_blinking(&self) -> bool {
		self.blinking
	}

	pub fn cursor(&self) -> &Cell {
		self.get(self.cursor.0, self.cursor.1)
	}

	pub fn get(&self, x: u32, y: u32) -> &Cell {
		&self.cells[(y * self.area.width + x) as usize]
	}

	pub fn area(&self, area: Area) -> iter::Area {
		iter::Area::new(&self, area)
	}

	pub fn iter(&self) -> iter::Area {
		iter::Area::new(&self, self.area)
	}

	pub fn resize<'a>(&'a mut self, width: u32, height: u32) -> impl Iterator<Item = &'a Cell> {
		self.iter()
	}

	pub fn blinking<'a>(&'a mut self, value: bool) -> impl Iterator<Item = &'a Cell> {
		self.blinking = value;
		self.iter().filter(|c| c.style().attributes().contains(style::BLINK))
	}

	pub fn key<'a, O: Write>(&'a mut self, key: Key, mut output: O) -> error::Result<impl Iterator<Item = &'a Cell>> {
		macro_rules! write {
			($item:expr) => (
				try!(($item.into(): control::Item).fmt(output.by_ref(), true));
			);
		}

		match key {
			Key::Enter => {
				write!(C0::LineFeed);
			}
		}

		Ok(iter::Area::new(self, Area::from(0, 0, 0, 0)))
	}

	pub fn input<'a, I: AsRef<str>, O: Write>(&'a mut self, input: I, mut output: O) -> error::Result<impl Iterator<Item = &'a Cell>> {
		try!(output.write_all(input.as_ref().as_bytes()));

		Ok(iter::Area::new(self, Area::from(0, 0, 0, 0)))
	}

	pub fn handle<'a, I: AsRef<[u8]>, O: Write>(&'a mut self, input: I, mut output: O) -> error::Result<impl Iterator<Item = &'a Cell>> {
		let mut input = input.as_ref();

		loop {
			match control::parse(input) {
				IResult::Done(rest, item) => {
					input = rest;

					match item {
						control::Item::String(string) => {
							for ch in string.graphemes(true) {
								self.insert(ch.into());
							}
						}

						control::Item::C0(C0::CarriageReturn) => {
							self.cursor.0 = 0;
						}

						// TODO: properly scroll
						control::Item::C0(C0::LineFeed) => {
							self.cursor.1 += 1;
						}

						// TODO: properly wrap back
						control::Item::C0(C0::Backspace) => {
							self.cursor.0 -= 1;
						}

						control::Item::C1(C1::ControlSequence(CSI::SelectGraphicalRendition(attrs))) => {
							let mut style = *self.style;

							for attr in &attrs {
								match attr {
									&SGR::Reset =>
										style = Style::default(),

									&SGR::Font(SGR::Weight::Normal) =>
										style.attributes.remove(style::BOLD | style::FAINT),

									&SGR::Font(SGR::Weight::Bold) => {
										style.attributes.remove(style::FAINT);
										style.attributes.insert(style::BOLD);
									}

									&SGR::Font(SGR::Weight::Faint) => {
										style.attributes.remove(style::BOLD);
										style.attributes.insert(style::FAINT);
									}

									&SGR::Italic(true) =>
										style.attributes.insert(style::ITALIC),
									&SGR::Italic(false) =>
										style.attributes.remove(style::ITALIC),

									&SGR::Underline(true) =>
										style.attributes.insert(style::UNDERLINE),
									&SGR::Underline(false) =>
										style.attributes.remove(style::UNDERLINE),

									&SGR::Blink(true) =>
										style.attributes.insert(style::BLINK),
									&SGR::Blink(false) =>
										style.attributes.remove(style::BLINK),

									&SGR::Reverse(true) =>
										style.attributes.insert(style::REVERSE),
									&SGR::Reverse(false) =>
										style.attributes.remove(style::REVERSE),

									&SGR::Invisible(true) =>
										style.attributes.insert(style::INVISIBLE),
									&SGR::Invisible(false) =>
										style.attributes.remove(style::INVISIBLE),

									&SGR::Struck(true) =>
										style.attributes.insert(style::STRUCK),
									&SGR::Struck(false) =>
										style.attributes.remove(style::STRUCK),

									&SGR::Foreground(ref color) =>
										style.foreground = Some(to_rgba(color, &self.config, true)),

									&SGR::Background(ref color) =>
										style.background = Some(to_rgba(color, &self.config, false)),
								}
							}

							if style != *self.style {
								self.style = Rc::new(style);
							}
						}

						control => {
							debug!("unhandled control: {:?}", control);
						}
					}
				}

				IResult::Incomplete(_) => {
					// TODO(meh): fill cache
					break;
				}

				IResult::Error(e) => {
					return Err(Error::Message(e.to_string()));
				}
			}
		}

		Ok(self.iter())
	}

	// TODO(meh): handle wrapping.
	// TODO(meh): collapse references
	fn insert(&mut self, ch: String) -> u32 {
		let width = ch.width() as u32;
		let index = self.cursor.1 * self.area.width + self.cursor.0;
		let cells = &mut self.cells[index as usize .. (index + width) as usize];

		let (cell, rest) = cells.split_at_mut(1);
		cell[0].make_char(ch, false);
		cell[0].set_style(self.style.clone());

		for c in rest {
			c.make_reference(cell[0].x(), cell[0].y());
		}

		self.cursor.0 += width;
		width
	}

	fn delete(&mut self) -> u32 {
		0
	}
}

fn to_rgba(color: &SGR::Color, config: &Config, foreground: bool) -> Rgba<f64> {
	match color {
		&SGR::Color::Default => {
			if foreground {
				*config.style().color().foreground()
			}
			else {
				*config.style().color().background()
			}
		}

		&SGR::Color::Transparent =>
			Rgba::new(0.0, 0.0, 0.0, 0.0),

		&SGR::Color::Index(index) =>
			*config.color().get(index),

		&SGR::Color::Rgb(r, g, b) =>
			Rgba::new_u8(r, g, b, 255),

		&SGR::Color::Cmy(c, m, y) =>
			unreachable!(),

		&SGR::Color::Cmyk(c, m, y, k) =>
			unreachable!(),
	}
}
