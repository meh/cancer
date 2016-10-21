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
use std::mem;
use std::cmp;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use picto::Area;
use picto::color::Rgba;
use error;
use config::{self, Config};
use config::style::Shape;
use style::{self, Style};
use terminal::{Iter, Touched, Cell, Key, Action, cell};
use terminal::mode::{self, Mode};
use terminal::cursor::{self, Cursor};
use control::{self, Control, C0, C1, DEC, CSI, SGR};

#[derive(Debug)]
pub struct Terminal {
	config:  Arc<Config>,
	area:    Area,
	cache:   Option<Vec<u8>>,
	touched: Touched,
	mode:    Mode,

	rows:   VecDeque<VecDeque<Cell>>,
	scroll: Option<u32>,

	cursor: Cursor,
	saved:  Option<Cursor>,
}

macro_rules! term {
	($term:ident; charset) => (
		$term.cursor.charsets[$term.cursor.charset as usize]
	);

	($term:ident; row for $y:expr) => (
		($y + $term.scroll.unwrap_or_else(|| $term.rows.len() as u32 - $term.area.height)) as usize
	);

	($term:ident; row) => (
		vec_deque![Cell::empty($term.cursor.style().clone()); $term.area.width as usize]
	);

	($term:ident; extend $n:expr) => (
		$term.rows.extend(iter::repeat(term!($term; row)).take($n as usize));
	);

	($term:ident; scroll! up $n:tt) => (
		if $term.cursor.scroll == (0, $term.area.height - 1) {
			term!($term; extend $n);
			term!($term; touched all);

			let back   = $term.rows.len() - $term.area.height as usize;
			let scroll = $term.config.environment().scroll();

			if back > scroll {
				$term.rows.drain(.. back - scroll);
			}
		}
		else {
			term!($term; scroll up $n)
		}
	);

	($term:ident; scroll up $n:tt) => (
		term!($term; scroll up $n from $term.cursor.scroll.0)
	);

	($term:ident; scroll up $n:tt from $y:expr) => ({
		let y      = $y;
		let n      = cmp::max(0, cmp::min($term.cursor.scroll.1 - y + 1, $n));
		let row    = term!($term; row for y);
		let offset = $term.rows.len() - ($term.cursor.scroll.1 + 1) as usize;

		// Remove the lines.
		$term.rows.drain(row .. row + n as usize);

		// Fill missing lines.
		{
			let index = $term.rows.len() - offset;

			for i in 0 .. n {
				$term.rows.insert(index + i as usize, term!($term; row));
			}
		}

		// Mark the affected lines as touched.
		for y in y ... $term.cursor.scroll.1 {
			term!($term; touched line y);
		}
	});

	($term:ident; scroll down $n:tt) => (
		term!($term; scroll down $n from $term.cursor.scroll.0)
	);

	($term:ident; scroll down $n:tt from $y:expr) => ({
		let y   = $y;
		let n   = cmp::max(0, cmp::min($term.cursor.scroll.1 - y + 1, $n));
		let row = term!($term; row for y);

		// Split the rows at the current line.
		let mut rest = $term.rows.split_off(row);

		// Extend with new lines.
		term!($term; extend n);

		// Remove the scrolled off lines.
		let offset = $term.cursor.scroll.1 + 1 - y - n;
		rest.drain(offset as usize .. (offset + n) as usize);
		$term.rows.append(&mut rest);

		// Mark the affected lines as touched.
		for y in y ... $term.cursor.scroll.1 {
			term!($term; touched line y);
		}
	});

	($term:ident; style!) => (
		$term.cursor.style()
	);

	($term:ident; style) => (
		$term.cursor.style().clone()
	);

	($term:ident; cursor) => ({
		let x = $term.cursor.x();
		let y = $term.cursor.y();

		if let &Cell::Reference(offset) = term!($term; cell (x, y)) {
			(x - offset as u32, y)
		}
		else {
			(x, y)
		}
	});

	($term:ident; cursor $($travel:tt)*) => (
		$term.cursor.travel(cursor::$($travel)*, &mut $term.touched)
	);

	($term:ident; clean references ($x:expr, $y:expr)) => ({
		let x = $x;
		let y = $y;

		// Clear references.
		for x in x .. $term.area.width {
			if term!($term; cell (x, y)).is_reference() {
				term!($term; mut cell (x, y)).into_empty(term!($term; style));
			}
			else {
				break;
			}
		}
	});

	($term:ident; touched all) => (
		$term.touched.all();
	);

	($term:ident; touched line $y:expr) => (
		$term.touched.line($y);
	);

	($term:ident; touched ($x:expr, $y:expr)) => (
		$term.touched.mark($x, $y);
	);

	($term:ident; touched $pair:expr) => (
		$term.touched.push($pair);
	);

	($term:ident; mut cell ($x:expr, $y:expr)) => ({
		let row = term!($term; row for $y);
		&mut $term.rows[row][$x as usize]
	});

	($term:ident; cell ($x:expr, $y:expr)) => ({
		let row = term!($term; row for $y);
		&$term.rows[row][$x as usize]
	});
}

impl Terminal {
	pub fn open(config: Arc<Config>, width: u32, height: u32) -> error::Result<Self> {
		let area  = Area::from(0, 0, width, height);
		let style = Rc::new(Style::default());
		let rows  = vec_deque![vec_deque![Cell::empty(style.clone()); width as usize]; height as usize];

		Ok(Terminal {
			config:  config.clone(),
			area:    area,
			cache:   Default::default(),
			touched: Touched::default(),
			mode:    Mode::default(),

			rows:   rows,
			scroll: None,

			cursor: Cursor::new(config.clone(), width, height),
			saved:  None,
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
		let (x, y) = term!(self; cursor);
		cursor::Cell::new(&self.cursor, self.get(x, y))
	}

	/// Get the cell at the given position.
	pub fn get(&self, x: u32, y: u32) -> cell::Position {
		cell::Position::new(x, y, term!(self; cell (x, y)))
	}

	/// Get an iterator over positioned cells.
	pub fn iter<'a, T: Iterator<Item = (u32, u32)>>(&'a self, iter: T) -> impl Iterator<Item = cell::Position<'a>> {
		Iter::new(self, iter)
	}

	/// Resize the terminal.
	pub fn resize(&mut self, width: u32, height: u32) -> impl Iterator<Item = (u32, u32)> {
		self.cursor.resize(width, height);

		iter::empty()
	}

	/// Enable or disable blinking and return the affected cells.
	pub fn blinking(&mut self, value: bool) -> impl Iterator<Item = (u32, u32)> {
		if value {
			self.mode.insert(mode::BLINK);
		}
		else {
			self.mode.remove(mode::BLINK);
		}

		for (x, y) in self.area.absolute() {
			if let &Cell::Occupied { ref style, .. } = term!(self; cell (x, y)) {
				if style.attributes().contains(style::BLINK) {
					term!(self; touched (x, y));
				}
			}
		}

		self.touched.iter(self.area)
	}

	/// Handle a key.
	pub fn key<O: Write>(&mut self, key: Key, output: O) -> error::Result<impl Iterator<Item = (u32, u32)>> {
		if !self.mode.contains(mode::KEYBOARD_LOCK) {
			try!(key.write(self.mode, output));
		}

		Ok(iter::empty())
	}

	/// Handle output from the tty.
	pub fn handle<I: AsRef<[u8]>, O: Write>(&mut self, input: I, mut output: O) -> error::Result<(impl Iterator<Item = Action>, impl Iterator<Item = (u32, u32)>)> {
		// Juggle the incomplete buffer cache and the real input.
		let     input  = input.as_ref();
		let mut buffer = self.cache.take();

		if let Some(buffer) = buffer.as_mut() {
			buffer.extend_from_slice(input);
		}

		let     buffer  = buffer.as_ref();
		let mut input   = buffer.as_ref().map(AsRef::as_ref).unwrap_or(input);
		let mut actions = Vec::new(): Vec<Action>;

		debug!(target: "cancer::terminal::input", "input: {:?}", input);

		loop {
			if input.is_empty() {
				break;
			}

			// Try to parse the rest of the input.
			let item = match control::parse(input) {
				// This should never happen.
				control::Result::Error(err) => {
					error!("cannot parse control code: {:?}", err);
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

			debug!(target: "cancer::terminal::input::item", "item: {:?}", item);

			match item {
				// Attributes.
				Control::C1(C1::ControlSequence(CSI::DeviceAttributes(0))) => {
					try!(output.write_all(b"\033[?64;6;21c"));
				}

				Control::DEC(DEC::ScrollRegion { top, bottom }) => {
					let mut top    = top;
					let mut bottom = bottom.unwrap_or(self.area.height - 1);

					if top > bottom {
						mem::swap(&mut top, &mut bottom);
					}

					self.cursor.scroll = (top, bottom);
					term!(self; cursor Position(Some(0), Some(0)));
				}

				Control::C1(C1::ControlSequence(CSI::Set(modes))) => {
					for mode in modes {
						match mode {
							CSI::Mode::KeyboardLock =>
								self.mode.insert(mode::KEYBOARD_LOCK),

							CSI::Mode::InsertionReplacement =>
								self.mode.insert(mode::INSERT),

							CSI::Mode::SendReceive =>
								self.mode.insert(mode::ECHO),

							CSI::Mode::LineFeed =>
								self.mode.insert(mode::CRLF),

							mode =>
								debug!(target: "cancer::terminal::unhandled", "unhandled set: {:?}", mode)
						}
					}
				}

				Control::DEC(DEC::Set(modes)) => {
					for mode in modes {
						match mode {
							DEC::Mode::ApplicationCursor =>
								self.mode.insert(mode::APPLICATION_CURSOR),

							DEC::Mode::ReverseVideo => {
								self.mode.insert(mode::REVERSE);
								term!(self; touched all);
							}

							DEC::Mode::Origin => {
								self.cursor.state.insert(cursor::ORIGIN);
								term!(self; cursor Position(Some(0), Some(0)));
							}

							DEC::Mode::AutoWrap =>
								self.mode.insert(mode::WRAP),

							DEC::Mode::CursorVisible =>
								self.cursor.state.remove(cursor::VISIBLE),

							mode =>
								debug!(target: "cancer::terminal::unhandled", "unhandled set: {:?}", mode)
						}
					}
				}

				Control::C1(C1::ControlSequence(CSI::Private(b'h', None, args))) => {
					for arg in args.into_iter().flat_map(Option::into_iter) {
						match arg {
							1004 =>
								self.mode.insert(mode::FOCUS),

							2004 =>
								self.mode.insert(mode::BRACKETED_PASTE),

							n =>
								debug!(target: "cancer::terminal::unhandled", "unhandled set: {}", n)
						}
					}
				}

				Control::C1(C1::ControlSequence(CSI::Reset(modes))) => {
					for mode in modes {
						match mode {
							CSI::Mode::KeyboardLock =>
								self.mode.remove(mode::KEYBOARD_LOCK),

							CSI::Mode::InsertionReplacement =>
								self.mode.remove(mode::INSERT),

							CSI::Mode::SendReceive =>
								self.mode.remove(mode::ECHO),

							CSI::Mode::LineFeed =>
								self.mode.remove(mode::CRLF),

							mode =>
								debug!(target: "cancer::terminal::unhandled", "unhandled reset: {:?}", mode)
						}
					}
				}

				Control::DEC(DEC::Reset(modes)) => {
					for mode in modes {
						match mode {
							DEC::Mode::ApplicationCursor =>
								self.mode.remove(mode::APPLICATION_CURSOR),

							DEC::Mode::ReverseVideo => {
								self.mode.remove(mode::REVERSE);
								term!(self; touched all);
							}

							DEC::Mode::Origin =>
								self.cursor.state.remove(cursor::ORIGIN),

							DEC::Mode::AutoWrap =>
								self.mode.remove(mode::WRAP),

							DEC::Mode::CursorVisible =>
								self.cursor.state.insert(cursor::VISIBLE),

							mode =>
								debug!(target: "cancer::terminal::unhandled", "unhandled reset: {:?}", mode)
						}
					}
				}

				Control::C1(C1::ControlSequence(CSI::Private(b'l', None, args))) => {
					for arg in args.into_iter().flat_map(Option::into_iter) {
						match arg {
							1004 =>
								self.mode.remove(mode::FOCUS),

							2004 =>
								self.mode.remove(mode::BRACKETED_PASTE),

							n =>
								debug!(target: "cancer::terminal::unhandled", "unhandled reset: {:?}", n)
						}
					}
				}

				Control::DEC(DEC::ApplicationKeypad(true)) => {
					self.mode.insert(mode::APPLICATION_KEYPAD);
				}

				Control::DEC(DEC::ApplicationKeypad(false)) => {
					self.mode.remove(mode::APPLICATION_KEYPAD);
				}

				Control::C1(C1::ControlSequence(CSI::SaveCursor)) |
				Control::DEC(DEC::SaveCursor) => {
					self.saved = Some(self.cursor.clone());
				}

				Control::C1(C1::ControlSequence(CSI::RestoreCursor)) |
				Control::DEC(DEC::RestoreCursor) => {
					if let Some(cursor) = self.saved.take() {
						self.cursor = cursor;
					}
				}

				// Charset.
				Control::DEC(DEC::SelectCharset(i, charset)) => {
					if self.cursor.charsets.len() >= i as usize {
						self.cursor.charsets[i as usize] = charset;
					}
				}

				Control::C0(C0::ShiftIn) => {
					self.cursor.charset = 0;
				}

				Control::C0(C0::ShiftOut) => {
					self.cursor.charset = 1;
				}

				// Movement functions.
				Control::C0(C0::CarriageReturn) => {
					term!(self; cursor Position(Some(0), None));
				}

				Control::C0(C0::LineFeed) => {
					if term!(self; cursor Down(1)).is_some() {
						term!(self; scroll! up 1);
					}
				}

				Control::C0(C0::Backspace) => {
					term!(self; cursor Left(1));
				}

				Control::C1(C1::ControlSequence(CSI::CursorPosition { x, y })) => {
					term!(self; cursor Position(Some(x), Some(y)));
				}

				Control::C1(C1::ControlSequence(CSI::CursorVerticalPosition(n))) => {
					term!(self; cursor Position(None, Some(n)));
				}

				Control::C1(C1::ControlSequence(CSI::CursorHorizontalPosition(n))) => {
					term!(self; cursor Position(Some(n), None));
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

				Control::C1(C1::Index) => {
					if term!(self; cursor Down(1)).is_some() {
						term!(self; scroll up 1);
					}
				}

				Control::C1(C1::ReverseIndex) => {
					if term!(self; cursor Up(1)).is_some() {
						term!(self; scroll down 1);
					}
				}

				Control::C1(C1::ControlSequence(CSI::ScrollUp(n))) => {
					term!(self; scroll up n);
				}

				Control::C1(C1::ControlSequence(CSI::ScrollDown(n))) => {
					term!(self; scroll down n);
				}

				Control::DEC(DEC::BackIndex) => {
					if self.cursor.x() == 0 {
						let row = term!(self; row for 0);

						for row in row .. self.area.height as usize {
							self.rows[row].pop_back();
							self.rows[row].push_front(Cell::empty(term!(self; style)));
						}

						for y in 0 .. self.area.height {
							term!(self; clean references (0, y));
						}
					}
					else {
						term!(self; cursor Left(1));
					}
				}

				Control::DEC(DEC::ForwardIndex) => {
					if self.cursor.x() == self.area.width - 1 {
						let row = term!(self; row for 0);

						for row in row .. self.area.height as usize {
							self.rows[row].pop_front();
							self.rows[row].push_back(Cell::empty(term!(self; style)));
						}

						for y in 0 .. self.area.height {
							term!(self; clean references (0, y));
						}
					}
					else {
						term!(self; cursor Right(1));
					}
				}

				Control::C1(C1::NextLine) => {
					if term!(self; cursor Down(1)).is_some() {
						term!(self; scroll up 1);
					}

					term!(self; cursor Position(Some(0), None));
				}

				// Erase functions.
				Control::C1(C1::ControlSequence(CSI::EraseDisplay(CSI::Erase::ToEnd))) => {
					let (x, y) = self.cursor.position();

					for x in x .. self.area.width {
						term!(self; touched (x, y));
						term!(self; mut cell (x, y)).into_empty(term!(self; style));
					}

					for y in y .. self.area.height {
						term!(self; touched line y);

						for x in 0 .. self.area.width {
							term!(self; mut cell (x, y)).into_empty(term!(self; style));
						}
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseDisplay(CSI::Erase::ToStart))) => {
					let (x, y) = self.cursor.position();

					for x in 0 ... x {
						term!(self; touched (x, y));
						term!(self; mut cell (x, y)).into_empty(term!(self; style));
					}

					for y in 0 .. y {
						term!(self; touched line y);

						for x in 0 .. self.area.width {
							term!(self; mut cell (x, y)).into_empty(term!(self; style));
						}
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseDisplay(CSI::Erase::All))) => {
					term!(self; touched all);

					for y in 0 .. self.area.height {
						for x in 0 .. self.area.width {
							term!(self; mut cell (x, y)).into_empty(term!(self; style));
						}
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseLine(CSI::Erase::ToEnd))) => {
					let (x, y) = self.cursor.position();

					for x in x .. self.area.width {
						term!(self; touched (x, y));
						term!(self; mut cell (x, y)).into_empty(term!(self; style));
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseLine(CSI::Erase::ToStart))) => {
					let (x, y) = self.cursor.position();

					for x in 0 ... x {
						term!(self; touched (x, y));
						term!(self; mut cell (x, y)).into_empty(term!(self; style));
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseLine(CSI::Erase::All))) => {
					let y = self.cursor.y();

					term!(self; touched line y);

					for x in 0 .. self.area.width {
						term!(self; mut cell (x, y)).into_empty(term!(self; style));
					}
				}

				Control::C1(C1::ControlSequence(CSI::EraseCharacter(n))) => {
					let (x, y) = term!(self; cursor);

					for x in x .. x + n {
						term!(self; mut cell (x, y)).into_empty(term!(self; style));
						term!(self; touched (x, y));
					}

					term!(self; clean references (x + n, y));
				}

				Control::C1(C1::ControlSequence(CSI::DeleteLine(n))) => {
					term!(self; scroll up n from self.cursor.y());
				}

				Control::C1(C1::ControlSequence(CSI::DeleteCharacter(n))) => {
					let y   = self.cursor.y();
					let x   = self.cursor.x();
					let n   = cmp::max(0, cmp::min(self.area.width - x, n));
					let row = term!(self; row for y);
					let row = &mut self.rows[row as usize];

					row.drain(x as usize .. x as usize + n as usize);
					row.append(&mut vec_deque![Cell::empty(term!(self; style)); n as usize]);

					for x in x .. self.area.width {
						term!(self; touched (x, y));
					}
				}

				// Insertion functions.
				Control::DEC(DEC::AlignmentTest) => {
					for (x, y) in self.area.absolute() {
						term!(self; mut cell (x, y)).into_occupied("E", term!(self; style));
					}

					term!(self; touched all);
				}

				Control::C1(C1::ControlSequence(CSI::InsertLine(n))) => {
					term!(self; scroll down n from self.cursor.y());
				}

				Control::C1(C1::ControlSequence(CSI::InsertCharacter(n))) => {
					let y   = self.cursor.y();
					let x   = self.cursor.x();
					let n   = cmp::max(0, cmp::min(self.area.width, n));
					let row = term!(self; row for y);
					let row = &mut self.rows[row as usize];

					for _ in x .. x + n {
						row.insert(x as usize, Cell::empty(term!(self; style)));
					}

					row.drain(self.area.width as usize ..);

					for x in x .. self.area.width {
						term!(self; touched (x, y));
					}
				}

				Control::C0(C0::HorizontalTabulation) |
				Control::C1(C1::HorizontalTabulationSet) => {
					let x   = self.cursor.x();
					let y   = self.cursor.y();

					for x in x .. x + (8 - (x % 8)) {
						term!(self; mut cell (x, y)).into_empty(self.cursor.style.clone());
						term!(self; cursor Right(1));
					}
				}

				Control::None(string) => {
					for mut ch in string.graphemes(true) {
						if term!(self; charset) == DEC::Charset::DEC(DEC::charset::DEC::Graphic) {
							ch = match ch {
								"A" => "↑",
								"B" => "↓",
								"C" => "→",
								"D" => "←",
								"E" => "█",
								"F" => "▚",
								"G" => "☃",
								"_" => " ",
								"`" => "◆",
								"a" => "▒",
								"b" => "␉",
								"c" => "␌",
								"d" => "␍",
								"e" => "␊",
								"f" => "°",
								"g" => "±",
								"h" => "␤",
								"i" => "␋",
								"j" => "┘",
								"k" => "┐",
								"l" => "┌",
								"m" => "└",
								"n" => "┼",
								"o" => "⎺",
								"p" => "⎻",
								"q" => "─",
								"r" => "⎼",
								"s" => "⎽",
								"t" => "├",
								"u" => "┤",
								"v" => "┴",
								"w" => "┬",
								"x" => "│",
								"y" => "≤",
								"z" => "≥",
								"{" => "π",
								"|" => "≠",
								"}" => "£",
								"~" => "·",
								_   => ch,
							};
						}

						let width = ch.width() as u32;

						if self.mode.contains(mode::WRAP) && self.cursor.wrap() {
							if term!(self; cursor Down(1)).is_some() {
								term!(self; scroll! up 1);
							}

							term!(self; cursor Position(Some(0), None));
						}

						// Change the cells appropriately.
						//
						// TODO: make the logic here a little cleaner, there's a bunch of
						//       code repetition.
						{
							let (x, y) = term!(self; cursor);

							// If it's all white-space, make the cells empty, otherwise make
							// them occupied.
							if ch.chars().all(char::is_whitespace) {
								if !term!(self; cell (x, y)).is_empty() || term!(self; cell (x, y)).style() != term!(self; style!) {
									for x in x .. x + width {
										term!(self; mut cell (x, y)).into_empty(term!(self; style));
										term!(self; touched (x, y));
									}

									term!(self; clean references (x + width, y));
								}
							}
							else {
								let changed = match term!(self; cell (x, y)) {
									&Cell::Empty { .. } =>
										true,

									&Cell::Occupied { ref style, ref value, .. } =>
										value != ch || style != term!(self; style!),

									&Cell::Reference(..) =>
										unreachable!()
								};

								if changed {
									term!(self; mut cell (x, y)).into_occupied(ch, term!(self; style));
									term!(self; touched (x, y));

									for (i, x) in (x + 1 .. x + width).enumerate() {
										term!(self; mut cell (x, y)).into_reference(i as u8 + 1);
									}

									term!(self; clean references (x + width, y));
								}
							}
						}

						// If the character overflows the area, mark it for wrapping.
						if self.cursor.x() + width >= self.area.width {
							self.cursor.state.insert(cursor::WRAP);
						}
						else {
							term!(self; cursor Right(width));
						}
					}
				}

				// Style functions.
				Control::C1(C1::ControlSequence(CSI::SelectGraphicalRendition(attrs))) => {
					let mut style = **term!(self; style!);

					for attr in &attrs {
						match *attr {
							SGR::Reset =>
								style = Style::default(),

							SGR::Font(SGR::Weight::Normal) =>
								style.attributes.remove(style::BOLD | style::FAINT),

							SGR::Font(SGR::Weight::Bold) => {
								style.attributes.remove(style::FAINT);
								style.attributes.insert(style::BOLD);
							}

							SGR::Font(SGR::Weight::Faint) => {
								style.attributes.remove(style::BOLD);
								style.attributes.insert(style::FAINT);
							}

							SGR::Italic(true) =>
								style.attributes.insert(style::ITALIC),
							SGR::Italic(false) =>
								style.attributes.remove(style::ITALIC),

							SGR::Underline(true) =>
								style.attributes.insert(style::UNDERLINE),
							SGR::Underline(false) =>
								style.attributes.remove(style::UNDERLINE),

							SGR::Blink(true) =>
								style.attributes.insert(style::BLINK),
							SGR::Blink(false) =>
								style.attributes.remove(style::BLINK),

							SGR::Reverse(true) =>
								style.attributes.insert(style::REVERSE),
							SGR::Reverse(false) =>
								style.attributes.remove(style::REVERSE),

							SGR::Invisible(true) =>
								style.attributes.insert(style::INVISIBLE),
							SGR::Invisible(false) =>
								style.attributes.remove(style::INVISIBLE),

							SGR::Struck(true) =>
								style.attributes.insert(style::STRUCK),
							SGR::Struck(false) =>
								style.attributes.remove(style::STRUCK),

							SGR::Foreground(ref color) =>
								style.foreground = Some(to_rgba(color, &self.config, true)),

							SGR::Background(ref color) =>
								style.background = Some(to_rgba(color, &self.config, false)),
						}
					}

					self.cursor.update(style);
				}

				Control::DEC(DEC::CursorStyle(n)) => {
					match n {
						0 => {
							if self.config.style().cursor().blink() {
								self.cursor.state.insert(cursor::BLINK);
							}

							self.cursor.shape = self.config.style().cursor().shape();
						}

						1 => {
							self.cursor.state.insert(cursor::BLINK);
							self.cursor.shape = Shape::Block;
						}

						2 => {
							self.cursor.state.remove(cursor::BLINK);
							self.cursor.shape = Shape::Block;
						}

						3 => {
							self.cursor.state.insert(cursor::BLINK);
							self.cursor.shape = Shape::Line;
						}

						4 => {
							self.cursor.state.remove(cursor::BLINK);
							self.cursor.shape = Shape::Line;
						}

						5 => {
							self.cursor.state.insert(cursor::BLINK);
							self.cursor.shape = Shape::Beam;
						}

						6 => {
							self.cursor.state.remove(cursor::BLINK);
							self.cursor.shape = Shape::Beam;
						}

						_ => ()
					}
				}

				Control::C1(C1::OperatingSystemCommand(cmd)) if cmd.starts_with("cursor:") => {
					let mut parts = cmd.split(':').skip(1);

					match parts.next() {
						Some("show") => {
							self.cursor.state.insert(cursor::VISIBLE);
						}

						Some("hide") => {
							self.cursor.state.remove(cursor::VISIBLE);
						}

						Some("background") => {
							let     desc  = parts.next().unwrap_or("-");
							let mut color = *self.config.style().cursor().background();

							if let Some(c) = config::to_color(desc) {
								color = c;
							}

							self.cursor.background = color;
						}

						Some(_) | None => ()
					}
				}

				Control::C1(C1::OperatingSystemCommand(cmd)) if cmd.starts_with("0;") || cmd.starts_with("k;") => {
					actions.push(Action::Title(String::from(&cmd[2..])));
				}

				code => {
					debug!(target: "cancer::terminal::unhandled", "unhandled control code: {:?}", code);
				}
			}
		}

		Ok((actions.into_iter(), self.touched.iter(self.area)))
	}
}

fn to_rgba(color: &SGR::Color, config: &Config, foreground: bool) -> Rgba<f64> {
	match *color {
		SGR::Color::Default => {
			if foreground {
				*config.style().color().foreground()
			}
			else {
				*config.style().color().background()
			}
		}

		SGR::Color::Transparent =>
			Rgba::new(0.0, 0.0, 0.0, 0.0),

		SGR::Color::Index(index) =>
			*config.color().get(index),

		SGR::Color::Rgb(r, g, b) =>
			Rgba::new_u8(r, g, b, 255),

		SGR::Color::Cmy(c, m, y) => {
			let c = c as f64 / 255.0;
			let m = m as f64 / 255.0;
			let y = y as f64 / 255.0;

			Rgba::new(1.0 - c, 1.0 - m, 1.0 - y, 1.0)
		}

		SGR::Color::Cmyk(c, m, y, k) => {
			let c = c as f64 / 255.0;
			let m = m as f64 / 255.0;
			let y = y as f64 / 255.0;
			let k = k as f64 / 255.0;

			Rgba::new(
				1.0 - (c * (1.0 - k) + k),
				1.0 - (m * (1.0 - k) + k),
				1.0 - (y * (1.0 - k) + k),
				1.0)
		}
	}
}
