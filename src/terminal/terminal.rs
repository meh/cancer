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
use std::mem;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use picto::Area;
use picto::color::Rgba;
use error::{self, Error};
use config::Config;
use style::{self, Style};
use terminal::{Dirty, Cell, Key, cell, iter};
use terminal::mode::{self, Mode};
use terminal::cursor::{self, Cursor, CursorCell};
use control::{self, Control, C0, C1, CSI, SGR};

#[derive(Debug)]
pub struct Terminal {
	config: Arc<Config>,
	area:   Area,
	cache:  Option<Vec<u8>>,

	mode:   Mode,
	cursor: Cursor,
	cells:  Vec<Cell>,
	dirty:  Dirty,
}

impl Terminal {
	pub fn open(config: Arc<Config>, width: u32, height: u32) -> error::Result<Self> {
		let area  = Area::from(0, 0, width, height);
		let style = Rc::new(Style::default());
		let cells = area.absolute().map(|(x, y)| Cell::new(x, y, style.clone()));

		Ok(Terminal {
			config: config,
			area:   area,
			cache:  Default::default(),

			mode:   Mode::empty(),
			cursor: Cursor::new(width, height),
			cells:  cells.collect(),
			dirty:  Dirty::default(),
		})
	}

	pub fn columns(&self) -> u32 {
		self.area.width
	}

	pub fn rows(&self) -> u32 {
		self.area.height
	}

	pub fn mode(&self) -> Mode {
		self.mode
	}

	pub fn cursor(&self) -> CursorCell {
		let (x, y) = self.cursor.position();
		CursorCell::new(&self.cursor, self.get(x, y))
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
		if value {
			self.mode.insert(mode::BLINK);
		}
		else {
			self.mode.remove(mode::BLINK);
		}

		self.iter().filter(|c| c.style().attributes().contains(style::BLINK))
	}

	pub fn key<'a, O: Write>(&'a mut self, key: Key, output: O) -> error::Result<impl Iterator<Item = &'a Cell>> {
		try!(key.write(output));
		Ok(iter::empty())
	}

	pub fn input<'a, I: AsRef<str>, O: Write>(&'a mut self, input: I, mut output: O) -> error::Result<impl Iterator<Item = &'a Cell>> {
		try!(output.write_all(input.as_ref().as_bytes()));
		Ok(iter::empty())
	}

	pub fn handle<'a, I: AsRef<[u8]>, O: Write>(&'a mut self, input: I, mut output: O) -> error::Result<impl Iterator<Item = &'a Cell>> {
		// Juggle the incomplete buffer cache and the real input.
		let     input  = input.as_ref();
		let mut buffer = self.cache.take();

		if let Some(buffer) = buffer.as_mut() {
			buffer.extend_from_slice(input);
		}

		let     buffer = buffer.as_ref();
		let mut input  = buffer.as_ref().map(AsRef::as_ref).unwrap_or(input);

		// Input parsing loop.
		loop {
			match control::parse(input) {
				control::Result::Error(e) => {
					return Err(Error::Message(e.to_string()));
				}

				// The given input isn't a complete sequence, cache the current input.
				control::Result::Incomplete(_) => {
					self.cache = Some(input.to_vec());
					break;
				}

				// We got a sequence or a raw input.
				control::Result::Done(rest, item) => {
					input = rest;

					println!("{:?}", item);

					match item {
						Control::None(string) => {
							for ch in string.graphemes(true) {
								self.insert(ch.into());
							}
						}

						Control::C1(C1::ControlSequence(CSI::CursorPosition { x, y })) => {
							self.cursor.travel(cursor::Position(Some(x), Some(y)), &mut self.dirty);
						}

						Control::C1(C1::ControlSequence(CSI::CursorUp(n))) => {
							self.cursor.travel(cursor::Up(n), &mut self.dirty);
						}

						Control::C1(C1::ControlSequence(CSI::CursorDown(n))) => {
							self.cursor.travel(cursor::Down(n), &mut self.dirty);
						}

						Control::C1(C1::ControlSequence(CSI::CursorBack(n))) => {
							self.cursor.travel(cursor::Left(n), &mut self.dirty);
						}

						Control::C1(C1::ControlSequence(CSI::CursorForward(n))) => {
							self.cursor.travel(cursor::Right(n), &mut self.dirty);
						}

						Control::C0(C0::CarriageReturn) => {
							self.cursor.travel(cursor::Position(Some(0), None), &mut self.dirty);
						}

						Control::C0(C0::LineFeed) => {
							self.cursor.travel(cursor::Down(1), &mut self.dirty);
						}

						Control::C0(C0::Backspace) => {
							self.cursor.travel(cursor::Left(1), &mut self.dirty);
						}

						Control::C1(C1::ControlSequence(CSI::InsertBlankCharacter(n))) => {
							self.blank(n);
						}

						Control::C1(C1::ControlSequence(CSI::SelectGraphicalRendition(attrs))) => {
							let mut style = **self.cursor.style();

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

							self.cursor.update(style);
						}

						control => {
							debug!("unhandled control: {:?}", control);
						}
					}
				}
			}
		}

		let dirty = self.dirty.take();
		Ok(iter::Indexed::new(self, dirty.into_iter()))
	}

	// TODO(meh): handle wrapping.
	// TODO(meh): collapse references
	fn insert(&mut self, ch: String) -> u32 {
		let width = ch.width() as u32;
		let index = self.cursor.y() * self.area.width + self.cursor.x();
		let cells = &mut self.cells[index as usize .. (index + width) as usize];

		let (cell, rest) = cells.split_at_mut(1);
		let cell         = &mut cell[0];

		let changed = if let Some(char) = cell.char() {
			ch != char || cell.style() != &**self.cursor.style()
		}
		else {
			true
		};

		if changed {
			cell.make_char(ch, false);
			cell.set_style(self.cursor.style().clone());

			self.dirty.mark(cell.x(), cell.y());

			for c in rest {
				self.dirty.mark(c.x(), c.y());
				c.make_reference(cell.x(), cell.y());
			}
		}

		self.cursor.travel(cursor::Right(width), &mut self.dirty);
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

		&SGR::Color::Cmy(..) |
		&SGR::Color::Cmyk(..) =>
			unreachable!(),
	}
}
