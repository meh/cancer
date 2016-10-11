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
use std::collections::VecDeque;
use std::iter;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use picto::Area;
use picto::color::Rgba;
use error::{self, Error};
use config::Config;
use style::{self, Style};
use terminal::{Iter, Dirty, Cell, Key, cell};
use terminal::mode::{self, Mode};
use terminal::cursor::{self, Cursor};
use control::{self, Control, C0, C1, CSI, SGR};

#[derive(Debug)]
pub struct Terminal {
	config: Arc<Config>,
	area:   Area,
	cache:  Option<Vec<u8>>,

	mode:   Mode,
	cursor: Cursor,
	rows:   VecDeque<Vec<Cell>>,
	dirty:  Dirty,
	scroll: Option<u32>,
	empty:  Rc<Style>,
}

macro_rules! term {
	($term:ident; row for $y:expr) => (
		($y + $term.scroll.unwrap_or_else(|| $term.rows.len() as u32 - $term.area.height)) as usize
	);

	($term:ident; extend) => (
		$term.rows.push_back(vec![Cell::empty($term.empty.clone()); $term.area.width as usize]);
	);

	($term:ident; cursor $($travel:tt)*) => (
		if let Some(n) = $term.cursor.travel(cursor::$($travel)*, &mut $term.dirty) {
			$term.dirty.all();

			for _ in 0 .. n {
				term!($term; extend);
			}
		}
	);

	($term:ident; dirty) => (
		$term.dirty.all();
	);

	($term:ident; dirty line $y:expr) => (
		$term.dirty.line($y);
	);

	($term:ident; dirty ($x:expr, $y:expr)) => (
		$term.dirty.mark($x, $y);
	);

	($term:ident; dirty $pair:expr) => (
		$term.dirty.push($pair);
	);

	($term:ident; mut cell $x:expr, $y:expr) => ({
		let row = term!($term; row for $y);
		&mut $term.rows[row][$x as usize]
	})
}

impl Terminal {
	pub fn open(config: Arc<Config>, width: u32, height: u32) -> error::Result<Self> {
		let area  = Area::from(0, 0, width, height);
		let style = Rc::new(Style::default());
		let rows  = (0 .. height).map(|_|
			vec![Cell::empty(style.clone()); width as usize]).collect();

		Ok(Terminal {
			config: config,
			area:   area,
			cache:  Default::default(),

			mode:   Mode::empty(),
			cursor: Cursor::new(width, height),
			rows:   rows,
			dirty:  Dirty::default(),
			scroll: None,
			empty:  style.clone(),
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

	/// Get the cursor.
	pub fn cursor(&self) -> cursor::Cell {
		cursor::Cell::new(&self.cursor, self.get(self.cursor.x(), self.cursor.y()))
	}

	/// Get the cell at the given position.
	pub fn get(&self, x: u32, y: u32) -> cell::Position {
		cell::Position::new(x, y, &self.rows[term!(self; row for y)][x as usize])
	}

	/// Return the cells within the given area.
	pub fn area<'a>(&'a self, area: Area) -> impl Iterator<Item = cell::Position<'a>> {
		Iter::new(self, area.absolute())
	}

	/// Return all cells.
	pub fn iter<'a>(&'a self) -> impl Iterator<Item = cell::Position<'a>> {
		Iter::new(self, self.area.absolute())
	}

	/// Resize the terminal.
	// TODO: handle resize
	pub fn resize<'a>(&'a mut self, width: u32, height: u32) -> impl Iterator<Item = cell::Position<'a>> {
		self.iter()
	}

	/// Enable or disable blinking and return the affected cells.
	pub fn blinking<'a>(&'a mut self, value: bool) -> impl Iterator<Item = cell::Position<'a>> {
		if value {
			self.mode.insert(mode::BLINK);
		}
		else {
			self.mode.remove(mode::BLINK);
		}

		self.iter().filter(|c| c.style().attributes().contains(style::BLINK))
	}

	/// Handle a key.
	pub fn key<'a, O: Write>(&'a mut self, key: Key, output: O) -> error::Result<impl Iterator<Item = cell::Position<'a>>> {
		try!(key.write(output));
		Ok(iter::empty())
	}

	/// Handle raw input.
	pub fn input<'a, I: AsRef<str>, O: Write>(&'a mut self, input: I, mut output: O) -> error::Result<impl Iterator<Item = cell::Position<'a>>> {
		try!(output.write_all(input.as_ref().as_bytes()));
		Ok(iter::empty())
	}

	/// Handle output from the tty.
	pub fn handle<'a, I: AsRef<[u8]>, O: Write>(&'a mut self, input: I, mut output: O) -> error::Result<impl Iterator<Item = cell::Position<'a>>> {
		// Juggle the incomplete buffer cache and the real input.
		let     input  = input.as_ref();
		let mut buffer = self.cache.take();

		if let Some(buffer) = buffer.as_mut() {
			buffer.extend_from_slice(input);
		}

		let     buffer = buffer.as_ref();
		let mut input  = buffer.as_ref().map(AsRef::as_ref).unwrap_or(input);

		loop {
			// Try to parse the rest of the input.
			let item = match control::parse(input) {
				// This should never happen.
				control::Result::Error(e) => {
					break;
				}

				// The given input isn't a complete sequence, cache the current input.
				control::Result::Incomplete(_) => {
					self.cache = Some(input.to_vec());
					break;
				}

				// We got a sequence or a raw input.
				control::Result::Done(rest, item) => {
					input = rest;
					item
				}
			};

			match item {
				// Movement functions.
				Control::C0(C0::CarriageReturn) => {
					term!(self; cursor Position(Some(0), None));
				}

				Control::C0(C0::LineFeed) => {
					term!(self; cursor Down(1));
				}

				Control::C0(C0::Backspace) => {
					term!(self; cursor Left(1));
				}

				Control::C1(C1::ControlSequence(CSI::CursorPosition { x, y })) => {
					term!(self; cursor Position(Some(x), Some(y)));
				}

				Control::C1(C1::ControlSequence(CSI::CursorUp(n))) => {
					term!(self; cursor Up(n));
				}

				Control::C1(C1::ControlSequence(CSI::CursorDown(n))) => {
					term!(self; cursor Down(n));
				}

				Control::C1(C1::ControlSequence(CSI::CursorBack(n))) => {
					term!(self; cursor Left(n));
				}

				Control::C1(C1::ControlSequence(CSI::CursorForward(n))) => {
					term!(self; cursor Right(n));
				}

				// Erase functions.
				Control::C1(C1::ControlSequence(CSI::EraseDisplay(CSI::Erase::ToEnd))) => {
					let (x, y) = self.cursor.position();

					for x in x .. self.area.width {
						term!(self; dirty (x, y));
						term!(self; mut cell x, y).into_empty(self.empty.clone());
					}

					for y in y .. self.area.height {
						term!(self; dirty line y);

						for x in 0 .. self.area.width {
							term!(self; mut cell x, y).into_empty(self.empty.clone());
						}
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseDisplay(CSI::Erase::ToStart))) => {
					let (x, y) = self.cursor.position();

					for x in 0 .. x {
						term!(self; dirty (x, y));
						term!(self; mut cell x, y).into_empty(self.empty.clone());
					}

					for y in 0 .. y {
						term!(self; dirty line y);

						for x in 0 .. self.area.width {
							term!(self; mut cell x, y).into_empty(self.empty.clone());
						}
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseDisplay(CSI::Erase::All))) => {
					term!(self; dirty);

					for y in 0 .. self.area.height {
						for x in 0 .. self.area.width {
							term!(self; mut cell x, y).into_empty(self.empty.clone());
						}
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseLine(CSI::Erase::ToEnd))) => {
					let (x, y) = self.cursor.position();

					for x in x .. self.area.width {
						term!(self; dirty (x, y));
						term!(self; mut cell x, y).into_empty(self.empty.clone());
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseLine(CSI::Erase::ToStart))) => {
					let (x, y) = self.cursor.position();

					for x in 0 .. x {
						term!(self; dirty (x, y));
						term!(self; mut cell x, y).into_empty(self.empty.clone());
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseLine(CSI::Erase::All))) => {
					let y = self.cursor.y();

					term!(self; dirty line y);

					for x in 0 .. self.area.width {
						term!(self; mut cell x, y).into_empty(self.empty.clone());
					}
				}

				Control::C1(C1::ControlSequence(CSI::DeleteLine(n))) => {
					let y   = self.cursor.y();
					let row = term!(self; row for y);

					// Remove the lines.
					self.rows.drain(y as usize .. (y + n) as usize);

					// Fill missing lines.
					for _ in 0 .. n {
						term!(self; extend);
					}

					// Mark the affected lines as dirty.
					for y in y .. self.area.height {
						term!(self; dirty line y);
					}
				}

				// Insertion functions.
				Control::C1(C1::ControlSequence(CSI::InsertCharacter(n))) => {
					println!("BLANK: {}", n);
				}

				Control::C1(C1::ControlSequence(CSI::InsertLine(n))) => {
					let y   = self.cursor.y();
					let row = term!(self; row for y);

					// Split the rows at the current line.
					let mut rest = self.rows.split_off(row);

					// Extend with new lines.
					for _ in 0 .. n {
						term!(self; extend);
					}

					// Remove the scrolled off lines.
					rest.drain((self.area.height - y - n) as usize ..);
					self.rows.append(&mut rest);

					// Mark the affected lines as dirty.
					for y in y .. self.area.height {
						term!(self; dirty line y);
					}
				}

				Control::None(string) => {
					for ch in string.graphemes(true) {
						let width = ch.width() as u32;

						// If the character overflows the area, wrap it down.
						if self.cursor.x() + width > self.area.width {
							term!(self; cursor Down(1));
							term!(self; cursor Position(Some(1), None));
						}

						// Change the cells appropriately.
						{
							let x            = self.cursor.x();
							let y            = self.cursor.y();
							let row          = term!(self; row for y);
							let cells        = &mut self.rows[row][x as usize .. (x + width) as usize];
							let (cell, rest) = cells.split_at_mut(1);
							let cell         = &mut cell[0];

							let changed = match cell {
								&mut Cell::Empty { .. } =>
									true,

								&mut Cell::Occupied { ref style, ref value, .. } =>
									value != ch || style != self.cursor.style(),

								&mut Cell::Reference(..) =>
									false
							};

							if changed {
								cell.into_occupied(ch.into(), self.cursor.style().clone());

								term!(self; dirty (x, y));
								for (i, c) in rest.iter_mut().enumerate() {
									c.into_reference(i as u8 + 1);
								}
							}
						}

						term!(self; cursor Right(width));
					}
				}

				// Style functions.
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

				code => {
					debug!("unhandled control code: {:?}", code);
				}
			}
		}

		let dirty = self.dirty.iter(self.area);
		Ok(Iter::new(self, dirty))
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
