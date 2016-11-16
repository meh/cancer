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
use std::mem;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::vec;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

use error;
use style::Style;
use platform::key::{self, Key};
use platform::mouse::{self, Mouse};
use terminal::{Access, Terminal, Cursor, Iter, Row};
use terminal::touched::{self, Touched};
use terminal::cell::{self, Cell};
use terminal::cursor;
use overlay::Status;
use overlay::command::{self, Command};
use interface::Action;

#[derive(Debug)]
pub struct Overlay {
	inner:   Terminal,
	cache:   Vec<u8>,
	touched: Touched,

	scroll: u32,
	cursor: Cursor,
	view:   HashMap<(u32, u32), Cell, BuildHasherDefault<FnvHasher>>,
	status: Option<Status>,

	prefix:    Option<u8>,
	times:     Option<u32>,
	selection: Option<Selection>,
	selected:  Rc<Style>,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Selection {
	Normal {
		start: (u32, u32),
		end:   (u32, u32)
	},

	Block {
		start: (u32, u32),
		end:   (u32, u32),
	},

	Line {
		start: u32,
		end:   u32,
	},
}

macro_rules! overlay {
	($term:ident; cursor) => ({
		let x = $term.cursor.x();
		let y = $term.cursor.y();

		if let Cell::Reference(offset) = *$term.get(x, y) {
			(x - offset as u32, y)
		}
		else {
			(x, y)
		}
	});

	($term:ident; cursor absolute) => ({
		let     (x, y) = overlay!($term; cursor);
		let mut offset = ($term.inner.rows() as u32 - 1 - y) + $term.scroll;

		if $term.status.is_some() {
			offset -= 1;
		}

		(x, offset)
	});

	($term:ident; cursor $($travel:tt)*) => ({
		$term.touched.push($term.cursor.position());
		let r = $term.cursor.travel(cursor::$($travel)*);
		$term.touched.push($term.cursor.position());

		r
	});

	($term:ident; while $condition:expr => $block:block) => ({
		let mut before = (overlay!($term; cursor), $term.scroll);

		while $condition {
			$block;

			let after = (overlay!($term; cursor), $term.scroll);

			if before == after {
				break;
			}
			else {
				before = after;
			}
		}
	});

	($term:ident; status mode $name:expr) => ({
		if let Some(status) = $term.status.as_mut() {
			$term.touched.line($term.inner.rows() - 1);
			status.mode($name);
		}
	});
}

impl Overlay {
	/// Create a new `Overlay` for the given `Terminal`.
	pub fn new(inner: Terminal) -> Self {
		let mut cursor = inner.cursor().clone();
		mem::swap(&mut cursor.foreground, &mut cursor.background);

		let status = inner.config().style().status().map(|c| {
			cursor.travel(cursor::Up(1));
			cursor.scroll = (0, inner.rows() - 2);

			let mut status = Status::new(c, inner.columns());
			status.mode("NORMAL");

			let (x, y) = cursor.position();
			let y      = inner.grid().back().len() as u32 + y + 2;
			status.position((x, y));

			status
		});

		let mut style = Style::default();
		style.foreground = Some(*inner.config().style().selection().foreground());
		style.background = Some(*inner.config().style().selection().background());
		style.attributes = inner.config().style().selection().attributes();

		Overlay {
			inner:   inner,
			touched: Touched::default(),
			cache:   Vec::new(),

			scroll: 0,
			cursor: cursor,
			view:   Default::default(),
			status: status,

			prefix: None,
			times:  None,

			selection: None,
			selected:  Rc::new(style),
		}
	}

	/// Convert the `Overlay` into its wrapped `Terminal`, writing any cached
	/// input.
	pub fn into_inner<W: Write>(mut self, output: W) -> error::Result<Terminal> {
		try!(self.inner.input(self.cache, output));
		Ok(self.inner)
	}

	/// Get the cell at the given coordinates taking scrolling and status bar
	/// into consideration.
	fn get(&self, x: u32, y: u32) -> &Cell {
		if let Some(status) = self.status.as_ref() {
			if y == self.inner.rows() - 1 {
				return &status[x as usize];
			}
		}

		let     back   = self.inner.grid().back();
		let     view   = self.inner.grid().view();
		let mut offset = (view.len() as u32 - 1 - y) + self.scroll;

		if self.status.is_some() {
			offset -= 1;
		}

		if let Some(cell) = self.view.get(&(x, offset)) {
			return cell;
		}

		if offset as usize >= view.len() {
			&back[back.len() - 1 - (offset as usize - view.len())][x as usize]
		}
		else {
			&view[view.len() - 1 - offset as usize][x as usize]
		}
	}

	/// Fetch the underlying row at the given index based on the current
	/// scrolling.
	fn row(&self, y: u32) -> &Row {
		let back = self.inner.grid().back();
		let view = self.inner.grid().view();

		if y as usize >= view.len() {
			&back[back.len() - 1 - (y as usize - view.len())]
		}
		else {
			&view[view.len() - 1 - y as usize]
		}
	}

	/// Get the current cursor position.
	pub fn cursor(&self) -> cursor::Cell {
		let (x, y) = overlay!(self; cursor);
		cursor::Cell::new(&self.cursor, cell::Position::new(x, y, self.get(x, y)))
	}

	/// Get an iterator over positioned cells.
	pub fn iter<T: Iterator<Item = (u32, u32)>>(&self, iter: T) -> Iter<Self, T> {
		Iter::new(self, iter)
	}

	/// Handle key input.
	pub fn key(&mut self, key: Key) -> (vec::IntoIter<Action>, touched::Iter) {
		use platform::key::{Value, Button, Keypad};

		debug!(target: "cancer::overlay::input", "key {:?}", key);

		// Check if the key is a number that makes operations run N times, if so
		// bail out early.
		if let Value::Char(ref ch) = *key.value() {
			if let Ok(number) = ch.parse::<u32>() {
				if self.times.is_some() || number != 0 {
					if let Some(times) = self.times.take() {
						self.times = Some(times * 10 + number);
					}
					else {
						self.times = Some(number);
					}

					return (Vec::new().into_iter(), self.touched.iter(self.inner.region()));
				}
			}
		}

		let new    = self.prefix.is_none();
		let times  = self.times.take();
		let prefix = self.prefix.take();

		let command = match *key.value() {
			Value::Char(ref ch) => match &**ch {
				"c" if key.modifier() == key::CTRL =>
					Command::Exit,

				"i" if key.modifier().is_empty() =>
					Command::Exit,

				"q" if key.modifier().is_empty() =>
					Command::Exit,

				"y" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::Up(times.unwrap_or(1))),

				"e" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::Down(times.unwrap_or(1))),

				"u" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::PageUp(times.unwrap_or(1))),

				"d" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::PageDown(times.unwrap_or(1))),

				"$" =>
					Command::Move(command::Move::End),

				"^" | "0" =>
					Command::Move(command::Move::Start),

				"h" if key.modifier().is_empty() =>
					Command::Move(command::Move::Left(times.unwrap_or(1))),

				"j" if key.modifier().is_empty() =>
					Command::Move(command::Move::Down(times.unwrap_or(1))),

				"k" if key.modifier().is_empty() =>
					Command::Move(command::Move::Up(times.unwrap_or(1))),

				"l" if key.modifier().is_empty() =>
					Command::Move(command::Move::Right(times.unwrap_or(1))),

				"w" if key.modifier().is_empty() =>
					Command::Move(command::Move::Word(command::Word::Next(times.unwrap_or(1)))),

				"b" if key.modifier().is_empty() =>
					Command::Move(command::Move::Word(command::Word::Previous(times.unwrap_or(1)))),

				"e" if key.modifier().is_empty() && prefix == Some(b'g') =>
					Command::Move(command::Move::Word(command::Word::PreviousEnd(times.unwrap_or(1)))),

				"e" if key.modifier().is_empty() =>
					Command::Move(command::Move::Word(command::Word::NextEnd(times.unwrap_or(1)))),

				"g" if key.modifier().is_empty() && prefix.is_none() => {
					self.prefix = Some(b'g');
					Command::None
				}

				"g" if key.modifier().is_empty() && prefix == Some(b'g') =>
					Command::Scroll(command::Scroll::Begin),

				"G" if key.modifier() == key::SHIFT => {
					if let Some(times) = times {
						Command::Scroll(command::Scroll::To(times))
					}
					else {
						Command::Scroll(command::Scroll::End)
					}
				}

				"v" if key.modifier().is_empty() =>
					Command::Select(command::Select::Normal),

				"v" if key.modifier() == key::CTRL =>
					Command::Select(command::Select::Block),

				"V" if key.modifier() == key::SHIFT =>
					Command::Select(command::Select::Line),

				"y" if key.modifier().is_empty() =>
					Command::Copy(match times {
						Some(1) => "PRIMARY",
						Some(2) => "SECONDARY",
						_       => "CLIPBOARD",
					}.into()),

				"p" if key.modifier().is_empty() =>
					Command::Paste(match times {
						Some(1) => "PRIMARY",
						Some(2) => "SECONDARY",
						_       => "CLIPBOARD",
					}.into()),

				_ => {
					debug!(target: "cancer::overlay::unhandled", "key {:?}", key);
					Command::None
				}
			},

			Value::Button(button) => match button {
				Button::Escape if key.modifier().is_empty() =>
					Command::Exit,

				Button::PageUp =>
					Command::Scroll(command::Scroll::PageUp(times.unwrap_or(1))),

				Button::PageDown =>
					Command::Scroll(command::Scroll::PageDown(times.unwrap_or(1))),

				Button::Left =>
					Command::Move(command::Move::Left(times.unwrap_or(1))),

				Button::Down =>
					Command::Move(command::Move::Down(times.unwrap_or(1))),

				Button::Up =>
					Command::Move(command::Move::Up(times.unwrap_or(1))),

				Button::Right =>
					Command::Move(command::Move::Right(times.unwrap_or(1))),

				Button::Home =>
					Command::Move(command::Move::Start),

				Button::End =>
					Command::Move(command::Move::End),

				Button::Insert if key.modifier() == key::SHIFT =>
					Command::Paste("PRIMARY".into()),

				_ => {
					debug!(target: "cancer::overlay::unhandled", "key {:?}", key);
					Command::None
				}
			},

			Value::Keypad(button) => match button {
				Keypad::Home =>
					Command::Move(command::Move::Start),

				Keypad::End =>
					Command::Move(command::Move::End),

				Keypad::Left =>
					Command::Move(command::Move::Left(times.unwrap_or(1))),

				Keypad::Down =>
					Command::Move(command::Move::Down(times.unwrap_or(1))),

				Keypad::Up =>
					Command::Move(command::Move::Up(times.unwrap_or(1))),

				Keypad::Right =>
					Command::Move(command::Move::Right(times.unwrap_or(1))),

				_ => {
					debug!(target: "cancer::overlay::unhandled", "key {:?}", key);
					Command::None
				}
			},
		};

		// Only remove the prefix if it hadn't just been set.
		if self.prefix.is_some() && !new {
			self.prefix = None;
		}

		let actions = self.handle(command);
		(actions.into_iter(), self.touched.iter(self.inner.region()))
	}

	/// Handle mouse events.
	pub fn mouse(&mut self, mouse: Mouse) -> (vec::IntoIter<Action>, touched::Iter) {
		debug!(target: "cancer::overlay::input", "mouse {:?}", mouse);

		let command = match mouse {
			Mouse::Click(mouse::Click { button: mouse::Button::Left, press: false, position, .. }) =>
				Command::Move(command::Move::To(position.x, position.y)),

			Mouse::Click(mouse::Click { button: mouse::Button::Middle, press: false, .. }) =>
				Command::Paste("PRIMARY".into()),

			Mouse::Click(mouse::Click { button: mouse::Button::Up, .. }) =>
				Command::Scroll(command::Scroll::Up(1)),

			Mouse::Click(mouse::Click { button: mouse::Button::Down, .. }) =>
				Command::Scroll(command::Scroll::Down(1)),

			_ =>
				Command::None,
		};

		let actions = self.handle(command);
		(actions.into_iter(), self.touched.iter(self.inner.region()))
	}

	/// Handle terminal input, effectively caching it until the overlay is
	/// closed.
	pub fn input<I: AsRef<[u8]>>(&mut self, input: I) {
		self.cache.extend(input.as_ref());
	}

	/// Handle a command.
	fn handle(&mut self, command: Command) -> Vec<Action> {
		let before  = overlay!(self; cursor absolute);
		let actions = self.command(command);
		let after   = overlay!(self; cursor absolute);

		if after != before {
			if self.selection.is_some() {
				let s = self.selection.unwrap();
				self.highlight(&s, false);
				self.select(before, after);
				let s = self.selection.unwrap();
				self.highlight(&s, true);
				self.touched.all();
			}

			if let Some(status) = self.status.as_mut() {
				let x = after.0 + 1;
				let y = self.inner.grid().back().len() as u32 + self.inner.grid().view().len() as u32 - after.1;

				self.touched.line(self.inner.rows() - 1);
				status.position((x, y));
			}
		}

		actions
	}

	fn command(&mut self, command: Command) -> Vec<Action> {
		fn is_boundary<T: AsRef<str>>(ch: T) -> bool {
			!ch.as_ref().chars().any(|c| c.is_alphabetic() || c.is_numeric())
		}

		let mut actions = Vec::new();

		match command {
			Command::None => (),
			Command::Exit => {
				if let Some(selection) = self.selection.take() {
					overlay!(self; status mode "NORMAL");

					self.highlight(&selection, false);
					self.touched.all();
				}
				else {
					actions.push(Action::Overlay(false));
				}
			}

			Command::Scroll(command::Scroll::Begin) => {
				self.scroll = self.inner.grid().back().len() as u32;
				self.touched.all();
			}

			Command::Scroll(command::Scroll::End) => {
				self.scroll = 0;
				self.touched.all();
			}

			Command::Scroll(command::Scroll::To(n)) => {
				self.scroll = (self.inner.grid().back().len() as u32).saturating_sub(n - 1);

				if self.status.is_some() {
					self.scroll += 1;
				}

				self.touched.all();
			}

			Command::Scroll(command::Scroll::Up(times)) => {
				for _ in 0 .. times {
					let offset = if self.status.is_some() { 1 } else { 0 };

					if self.scroll < self.inner.grid().back().len() as u32 + offset {
						self.scroll += 1;
					}
				}

				self.touched.all();
			}

			Command::Scroll(command::Scroll::Down(times)) => {
				for _ in 0 .. times {
					if self.scroll > 0 {
						self.scroll -= 1;
					}
				}

				self.touched.all();
			}

			Command::Scroll(command::Scroll::PageUp(times)) => {
				for _ in 0 .. times {
					self.scroll += self.inner.rows().saturating_sub(3);

					if self.scroll > self.inner.grid().back().len() as u32 {
						self.scroll = self.inner.grid().back().len().saturating_sub(1) as u32;
					}
				}

				self.touched.all();
			}

			Command::Scroll(command::Scroll::PageDown(times)) => {
				for _ in 0 .. times {
					self.scroll = self.scroll.saturating_sub(self.inner.rows() - 3);
				}

				self.touched.all();
			}

			Command::Move(command::Move::To(x, y)) => {
				if self.status.is_none() || y != self.inner.rows() - 1 {
					overlay!(self; cursor Position(Some(x), Some(y)));
				}
			}

			Command::Move(command::Move::End) => {
				overlay!(self; cursor Position(Some(self.inner.columns() - 1), None));
			}

			Command::Move(command::Move::Start) => {
				overlay!(self; cursor Position(Some(0), None));
			}

			Command::Move(command::Move::Left(times)) => {
				for _ in 0 ..times {
					if overlay!(self; cursor Left(1)).is_some() {
						if overlay!(self; cursor Up(1)).is_some() {
							self.command(Command::Scroll(command::Scroll::Up(1)));
						}

						overlay!(self; cursor Position(Some(self.inner.columns() - 1), None));
					}
				}
			}

			Command::Move(command::Move::Down(times)) => {
				for _ in 0 .. times {
					if overlay!(self; cursor Down(1)).is_some() {
						self.command(Command::Scroll(command::Scroll::Down(1)));
					}
				}
			}

			Command::Move(command::Move::Up(times)) => {
				for _ in 0 .. times {
					if overlay!(self; cursor Up(1)).is_some() {
						self.command(Command::Scroll(command::Scroll::Up(1)));
					}
				}
			}

			Command::Move(command::Move::Right(times)) => {
				for _ in 0 .. times {
					if overlay!(self; cursor Right(1)).is_some() {
						if overlay!(self; cursor Down(1)).is_some() && self.scroll != 0 {
							self.command(Command::Scroll(command::Scroll::Down(1)));
							overlay!(self; cursor Position(Some(0), None));
						}
					}
				}
			}

			Command::Move(command::Move::Word(command::Word::Next(times))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if !is_boundary(self.get(c.0, c.1).value()) {
						overlay!(self; while !is_boundary(self.get(c.0, c.1).value()) => {
							self.command(Command::Move(command::Move::Right(1)));
							c = overlay!(self; cursor);
						});
					}

					overlay!(self; while is_boundary(self.get(c.0, c.1).value()) => {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					});
				}
			}

			Command::Move(command::Move::Word(command::Word::Previous(times))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if !is_boundary(self.get(c.0, c.1).value()) {
						overlay!(self; while !is_boundary(self.get(c.0, c.1).value()) => {
							self.command(Command::Move(command::Move::Left(1)));
							c = overlay!(self; cursor);
						});
					}

					overlay!(self; while is_boundary(self.get(c.0, c.1).value()) => {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					});

					overlay!(self; while !is_boundary(self.get(c.0, c.1).value()) => {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					});

					if is_boundary(self.get(c.0, c.1).value()) {
						self.command(Command::Move(command::Move::Right(1)));
					}
				}
			}

			Command::Move(command::Move::Word(command::Word::NextEnd(times))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if !is_boundary(self.get(c.0, c.1).value()) {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}

					if is_boundary(self.get(c.0, c.1).value()) {
						overlay!(self; while is_boundary(self.get(c.0, c.1).value()) => {
							self.command(Command::Move(command::Move::Right(1)));
							c = overlay!(self; cursor);
						});
					}

					overlay!(self; while !is_boundary(self.get(c.0, c.1).value()) => {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					});

					if is_boundary(self.get(c.0, c.1).value()) {
						self.command(Command::Move(command::Move::Left(1)));
					}
				}
			}

			Command::Move(command::Move::Word(command::Word::PreviousEnd(times))) => {
				for _ in 0 .. times {
					let (mut x, mut y) = overlay!(self; cursor);

					if !is_boundary(self.get(x, y).value()) {
						while !is_boundary(self.get(x, y).value()) {
							self.command(Command::Move(command::Move::Left(1)));

							let (nx, ny) = overlay!(self; cursor);
							x = nx; y = ny;
						}
					}

					while is_boundary(self.get(x, y).value()) {
						self.command(Command::Move(command::Move::Left(1)));

						let (nx, ny) = overlay!(self; cursor);
						x = nx; y = ny;
					}
				}
			}

			Command::Select(mode) => {
				let (name, old, new) = match (mode, self.selection.take()) {
					(command::Select::Normal, Some(Selection::Normal { start, end })) => {
						("NORMAL",
							Some(Selection::Normal { start: start, end: end }),
							None)
					}

					(command::Select::Normal, Some(Selection::Block { start, end })) => {
						("VISUAL",
							Some(Selection::Block { start: start, end: end }),
							Some(Selection::Normal { start: start, end: end }))
					}

					(command::Select::Normal, Some(Selection::Line { start, end })) => {
						let columns = self.inner.columns();

						("VISUAL",
							Some(Selection::Line { start: start, end: end }),
							Some(Selection::Normal { start: (0, start), end: (columns - 1, end) }))
					}

					(command::Select::Normal, None) => {
						let (x, y) = overlay!(self; cursor absolute);

						("VISUAL",
							None,
							Some(Selection::Normal { start: (x, y), end: (x, y) }))
					}

					(command::Select::Block, Some(Selection::Block { start, end })) => {
						("NORMAL",
							Some(Selection::Block { start: start, end: end }),
							None)
					}

					(command::Select::Block, Some(Selection::Normal { start, end })) => {
						("VISUAL BLOCK",
							Some(Selection::Normal { start: start, end: end }),
							Some(Selection::Block { start: start, end: end }))
					}

					(command::Select::Block, Some(Selection::Line { start, end })) => {
						let columns = self.inner.columns();

						("VISUAL BLOCK",
							Some(Selection::Line { start: start, end: end }),
							Some(Selection::Block { start: (0, start), end: (columns - 1, end) }))
					}

					(command::Select::Block, None) => {
						let (x, y) = overlay!(self; cursor absolute);

						("VISUAL BLOCK",
							None,
							Some(Selection::Block { start: (x, y), end: (x, y) }))
					}

					(command::Select::Line, Some(Selection::Line { start, end })) => {
						("NORMAL",
							Some(Selection::Line { start: start, end: end }),
							None)
					}

					(command::Select::Line, Some(Selection::Normal { start, end })) => {
						("VISUAL LINE",
							Some(Selection::Normal { start: start, end: end }),
							Some(Selection::Line { start: start.1, end: end.1 }))
					}

					(command::Select::Line, Some(Selection::Block { start, end })) => {
						("VISUAL LINE",
							Some(Selection::Block { start: start, end: end }),
							Some(Selection::Line { start: start.1, end: end.1 }))
					}

					(command::Select::Line, None) => {
						let (_, y) = overlay!(self; cursor absolute);

						("VISUAL LINE",
							None,
							Some(Selection::Line { start: y, end: y }))
					}
				};

				overlay!(self; status mode name);

				if let Some(old) = old {
					self.highlight(&old, false);
				}

				if let Some(new) = new {
					self.selection = Some(new);
					self.highlight(&new, true);
				}

				self.touched.all();
			}

			Command::Copy(name) => {
				if let Some(selection) = self.selection.take() {
					overlay!(self; status mode "NORMAL");

					self.highlight(&selection, false);
					self.touched.all();

					actions.push(Action::Overlay(false));
					actions.push(Action::Copy(name, self.selection(&selection)));
				}
			}

			Command::Paste(name) => {
				actions.push(Action::Overlay(false));
				actions.push(Action::Paste(name));
			}
		}

		if let Some(selection) = self.selection {
			actions.push(Action::Copy("PRIMARY".into(), self.selection(&selection)));
		}

		actions
	}

	/// Turn the current selection to its text representation.
	fn selection(&self, selection: &Selection) -> String {
		/// Find the index of the first non-empty cell followed by only empty
		/// cells.
		fn edge(row: &Row, start: u32, end: u32) -> u32 {
			let mut found = None;

			for x in start .. end {
				let cell = &row[x as usize];

				if cell.is_empty() && found.is_none() {
					found = Some(x.saturating_sub(1));
				}
				else if cell.is_occupied() && found.is_some() {
					found = None;
				}
			}

			found.unwrap_or(end)
		}

		match *selection {
			Selection::Normal { start, end } => {
				let mut lines  = vec![];
				let mut unwrap = None::<Vec<String>>;

				for y in end.1 ... start.1 {
					let (start, end) = if start.1 == end.1 {
						(start.0, end.0)
					}
					else if y == start.1 {
						(start.0, self.inner.columns() - 1)
					}
					else if y == end.1 {
						(0, end.0)
					}
					else {
						(0, self.inner.columns() - 1)
					};

					let     row  = self.row(y);
					let mut line = String::new();

					for x in start ... edge(row, start, end) {
						line.push_str(row[x as usize].value());
					}

					if row.wrap() {
						if let Some(mut unwrapped) = unwrap.take() {
							unwrapped.push(line);
							unwrap = Some(unwrapped);
						}
						else {
							unwrap = Some(vec![line]);
						}
					}
					else if let Some(mut unwrapped) = unwrap.take() {
						unwrapped.push(line);
						lines.push(unwrapped);
					}
					else {
						lines.push(vec![line]);
					}
				}

				let mut result = String::new();

				for lines in lines.into_iter().rev() {
					for line in lines.into_iter().rev() {
						result.push_str(&line);
					}

					result.push('\n');
				}

				result.pop();
				result
			}

			Selection::Block { start, end } => {
				let mut result = String::new();

				for y in (end.1 ... start.1).rev() {
					let row = self.row(y);

					for x in start.0 ... edge(row, start.0, end.0) {
						result.push_str(row[x as usize].value());
					}

					result.push('\n');
				}

				result.pop();
				result
			}

			Selection::Line { start, end } => {
				self.selection(&Selection::Normal {
					start: (0, start),
					end:   (self.inner.columns() - 1, end)
				})
			}
		}
	}

	/// Update the current selection based on the cursor movement.
	fn select(&mut self, before: (u32, u32), after: (u32, u32)) {
		match *try!(return option self.selection.as_mut()) {
			Selection::Normal { ref mut start, ref mut end } => {
				// Cursor went down.
				if before.1 > after.1 {
					if after.1 <= start.1 && after.1 >= end.1 {
						*start = after;
					}
					else {
						*end = after;
					}
				}
				// Cursor went up.
				else if before.1 < after.1 {
					if after.1 > start.1 && after.1 >= start.1 {
						*start = after;
					}
					else {
						*end = after;
					}
				}
				// Cursor went right.
				else if after.0 > before.0 {
					if (start.1 == end.1 && after.0 >= end.0) ||
					   (start.1 != end.1 && end.1 == after.1)
					{
						end.0 = after.0;
					}
					else {
						start.0 = after.0;
					}
				}
				// Cursor went left.
				else if after.0 < before.0 {
					if (start.1 == end.1 && after.0 >= start.0) ||
					   (start.1 != end.1 && end.1 == after.1)
					{
						end.0 = after.0;
					}
					else {
						start.0 = after.0;
					}
				}
			}

			Selection::Block { ref mut start, ref mut end } => {
				// Cursor went down.
				if before.1 > after.1 {
					if after.1 < end.1 {
						end.1 = after.1;
					}
					else {
						start.1 = after.1;
					}
				}
				// Cursor went up.
				else if before.1 < after.1 {
					if after.1 > start.1 {
						start.1 = after.1;
					}
					else {
						end.1 = after.1;
					}
				}

				// Cursor went right.
				if after.0 > before.0 {
					if after.0 > end.0 {
						end.0 = after.0;
					}
					else {
						start.0 = after.0;
					}
				}
				// Cursor went left.
				else if after.0 < before.0 {
					if after.0 < start.0 {
						start.0 = after.0;
					}
					else {
						end.0 = after.0;
					}
				}
			}

			Selection::Line { ref mut start, ref mut end } => {
				// Cursor went down.
				if before.1 > after.1 {
					if after.1 < *end {
						*end = after.1;
					}
					else {
						*start = after.1;
					}
				}
				// Cursor went up.
				else if before.1 < after.1 {
					if after.1 > *start {
						*start = after.1;
					}
					else {
						*end = after.1;
					}
				}
			}
		}
	}

	/// Enable or disable highlighting of the given selection.
	fn highlight(&mut self, selection: &Selection, flag: bool) {
		match *selection {
			Selection::Normal { start, end } => {
				for y in end.1 ... start.1 {
					let (start, end) = if start.1 == end.1 {
						(start.0, end.0)
					}
					else if y == start.1 {
						(start.0, self.inner.columns() - 1)
					}
					else if y == end.1 {
						(0, end.0)
					}
					else {
						(0, self.inner.columns() - 1)
					};

					for x in start ... end {
						if flag {
							let mut cell = self.row(y)[x as usize].clone();
							cell.set_style(self.selected.clone());
							self.view.insert((x, y), cell);
						}
						else {
							self.view.remove(&(x, y));
						}
					}
				}
			}

			Selection::Block { start, end } => {
				for y in end.1 ... start.1 {
					for x in start.0 ... end.0 {
						if flag {
							let mut cell = self.row(y)[x as usize].clone();
							cell.set_style(self.selected.clone());
							self.view.insert((x, y), cell);
						}
						else {
							self.view.remove(&(x, y));
						}
					}
				}
			}

			Selection::Line { start, end } => {
				for y in end ... start {
					for x in 0 .. self.inner.columns() {
						if flag {
							let mut cell = self.row(y)[x as usize].clone();
							cell.set_style(self.selected.clone());
							self.view.insert((x, y), cell);
						}
						else {
							self.view.remove(&(x, y));
						}
					}
				}
			}
		}
	}
}

impl Access for Overlay {
	fn access(&self, x: u32, y: u32) -> &Cell {
		self.get(x, y)
	}
}

impl Deref for Overlay {
	type Target = Terminal;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for Overlay {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}
