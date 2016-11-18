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
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::vec;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

use unicode_segmentation::UnicodeSegmentation;
use error;
use style::{self, Style};
use platform::key::{self, Key};
use platform::mouse::{self, Mouse};
use terminal::{Access, Terminal, Cursor, Iter, Row};
use terminal::touched::{self, Touched};
use terminal::cell::{self, Cell};
use terminal::cursor;
use overlay::Status;
use overlay::command::{self, Command};
use overlay::hints::{Hint, Hints};
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

	prefix: Option<u8>,
	times:  Option<u32>,

	selector: Selector,
	hinter:   Hinter,
}

#[derive(PartialEq, Clone, Debug)]
struct Selector {
	current: Option<Selection>,
	style:   Rc<Style>,
}

#[derive(PartialEq, Clone, Debug)]
struct Hinter {
	selected: Option<String>,
	current:  Option<String>,
	level:    usize,
	hints:    Option<Hints>,

	label:      Rc<Style>,
	underlined: Rc<Style>,
	hinted:     Rc<Style>,
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

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Highlight<'a> {
	Selection(&'a Selection),
	Hint(&'a Hint, usize, bool),
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
		{
			let config = inner.config().style().overlay().cursor();

			cursor.foreground = *config.foreground();
			cursor.background = *config.background();
			cursor.shape      = config.shape();

			if config.blink() {
				cursor.state.insert(cursor::BLINK);
			}
			else {
				cursor.state.remove(cursor::BLINK);
			}
		}

		let status = inner.config().style().overlay().status().map(|c| {
			cursor.travel(cursor::Up(1));
			cursor.scroll = (0, inner.rows() - 2);

			let mut status = Status::new(*c, inner.columns());
			status.mode("NORMAL");

			let (x, y) = cursor.position();
			let y      = inner.grid().back().len() as u32 + y + 2;
			status.position((x, y));

			status
		});

		let selector = Selector {
			current: None,
			style:   Rc::new(*inner.config().style().overlay().selection()),
		};

		let hinter = Hinter {
			selected: None,
			current:  None,
			level:    0,
			hints:    None,

			label:      Rc::new(*inner.config().style().overlay().hint()),
			underlined: Rc::new(Style { attributes: style::UNDERLINE, .. Default::default() }),
			hinted:     Rc::new(Style {
				foreground: inner.config().style().overlay().hint().foreground,
				background: inner.config().style().overlay().hint().background,
				attributes: inner.config().style().overlay().hint().attributes ^ style::REVERSE,
			}),
		};

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

			selector: selector,
			hinter:   hinter,
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

	/// Check if the cursor is at the beginning.
	fn at_start(&self) -> bool {
		let (x, y) = overlay!(self; cursor);
		let back   = self.inner.grid().back().len() as u32 +
			if self.status.is_some() { 1 } else { 0 };

		self.scroll == back &&
		x == 0 &&
		y == 0
	}

	/// Check if the cursor is at the end.
	fn at_end(&self) -> bool {
		let (x, y) = overlay!(self; cursor);

		self.scroll == 0 &&
		x == self.inner.columns() - 1 &&
		y == self.inner.rows() - 1 - if self.status.is_some() { 1 } else { 0 }
	}

	/// Handle key input.
	pub fn key(&mut self, key: Key) -> (vec::IntoIter<Action>, touched::Iter) {
		use platform::key::{Value, Button, Keypad};

		debug!(target: "cancer::overlay::input", "key {:?}", key);

		// Check if the key is a number that makes operations run N times, if so
		// bail out early.
		//
		// The check is not done while in HINT mode.
		if self.hinter.hints.is_none() || self.hinter.selected.is_some() {
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
		}

		let times  = self.times.take();
		let prefix = self.prefix.take();

		let command = match *key.value() {
			Value::Char(ref ch) => match &**ch {
				"u" if key.modifier().is_empty() && self.hinter.hints.is_none() =>
					Command::Hint(command::Hint::Start),

				"o" if key.modifier().is_empty() && self.hinter.selected.is_some() =>
					Command::Hint(command::Hint::Open),

				"y" if key.modifier().is_empty() && self.hinter.selected.is_some() =>
					Command::Hint(command::Hint::Copy(match times {
						Some(1) => "PRIMARY",
						Some(2) => "SECONDARY",
						_       => "CLIPBOARD",
					}.into())),

				ch if key.modifier().is_empty() && self.hinter.hints.is_some() && self.hinter.selected.is_none() =>
					Command::Hint(command::Hint::Pick(ch.chars().next().unwrap())),

				"e" if key.modifier().is_empty() && prefix == Some(b'g') =>
					Command::Move(command::Move::Word(command::Word::PreviousEnd(times.unwrap_or(1)))),

				"g" if key.modifier().is_empty() && prefix == Some(b'g') =>
					Command::Scroll(command::Scroll::Begin),

				ch if prefix == Some(b'f') =>
					Command::Move(command::Move::Until(command::Until::Next(times.unwrap_or(1), ch.into()))),

				ch if prefix == Some(b'F') =>
					Command::Move(command::Move::Until(command::Until::Previous(times.unwrap_or(1), ch.into()))),

				ch if prefix == Some(b't') =>
					Command::Move(command::Move::Until(command::Until::NextBefore(times.unwrap_or(1), ch.into()))),

				ch if prefix == Some(b'T') =>
					Command::Move(command::Move::Until(command::Until::PreviousBefore(times.unwrap_or(1), ch.into()))),

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

				"e" if key.modifier().is_empty() =>
					Command::Move(command::Move::Word(command::Word::NextEnd(times.unwrap_or(1)))),

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

				"g" if key.modifier().is_empty() => {
					self.prefix = Some(b'g');
					Command::None
				}

				"f" if key.modifier().is_empty() => {
					self.prefix = Some(b'f');
					Command::None
				}

				"F" if key.modifier() == key::SHIFT => {
					self.prefix = Some(b'F');
					Command::None
				}

				"t" if key.modifier().is_empty() => {
					self.prefix = Some(b't');
					Command::None
				}

				"T" if key.modifier() == key::SHIFT => {
					self.prefix = Some(b'T');
					Command::None
				}

				_ => {
					debug!(target: "cancer::overlay::unhandled", "key {:?}", key);
					Command::None
				}
			},

			Value::Button(button) => match button {
				Button::Escape if key.modifier().is_empty() =>
					Command::Exit,

				_ if self.hinter.hints.is_some() =>
					Command::None,

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
				_ if self.hinter.hints.is_some() =>
					Command::None,

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

		let actions = self.handle(command);
		(actions.into_iter(), self.touched.iter(self.inner.region()))
	}

	/// Handle mouse events.
	pub fn mouse(&mut self, mouse: Mouse) -> (vec::IntoIter<Action>, touched::Iter) {
		debug!(target: "cancer::overlay::input", "mouse {:?}", mouse);

		let command = match mouse {
			_ if self.hinter.hints.is_some() =>
				Command::None,

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
		let     before  = overlay!(self; cursor absolute);
		let mut actions = self.command(command);
		let     after   = overlay!(self; cursor absolute);

		if after != before {
			if self.selector.current.is_some() {
				let s = self.selector.current.unwrap();
				self.highlight(Highlight::Selection(&s), false);
				self.select(before, after);
				let s = self.selector.current.unwrap();
				self.highlight(Highlight::Selection(&s), true);
				self.touched.all();
			}

			if let Some(status) = self.status.as_mut() {
				let x = after.0 + 1;
				let y = self.inner.grid().back().len() as u32 + self.inner.grid().view().len() as u32 - after.1;

				self.touched.line(self.inner.rows() - 1);
				status.position((x, y));
			}

			if let Some(selection) = self.selector.current {
				debug!(target: "cancer::overlay::selection", "selection: {:?}", self.selection(&selection));
				actions.push(Action::Copy("PRIMARY".into(), self.selection(&selection)));
			}
		}

		actions
	}

	fn command(&mut self, command: Command) -> Vec<Action> {
		fn is_boundary<T: AsRef<str>>(ch: T) -> bool {
			!ch.as_ref().chars().any(|c| c.is_alphabetic() || c.is_numeric())
		}

		debug!(target: "cancer::overlay::command", "command: {:?}", command);

		let mut actions = Vec::new();

		match command {
			Command::None => (),
			Command::Exit => {
				overlay!(self; status mode "NORMAL");

				if let Some(selection) = self.selector.current.take() {
					self.highlight(Highlight::Selection(&selection), false);
				}
				else if let Some(hints) = self.hinter.hints.take() {
					for hint in hints.values() {
						self.highlight(Highlight::Hint(hint, 0, false), false);
					}

					self.hinter.current.take();
					self.hinter.selected.take();
					self.hinter.level = 0;
				}
				else {
					actions.push(Action::Overlay(false));
				}

				self.touched.all();
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
						if overlay!(self; cursor Down(1)).is_some() {
							self.command(Command::Scroll(command::Scroll::Down(1)));
						}

						if !self.at_end() {
							overlay!(self; cursor Position(Some(0), None));
						}
					}
				}
			}

			Command::Move(command::Move::Word(command::Word::Next(times))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if !is_boundary(self.get(c.0, c.1).value()) {
						while !is_boundary(self.get(c.0, c.1).value()) && !self.at_end() {
							self.command(Command::Move(command::Move::Right(1)));
							c = overlay!(self; cursor);
						}
					}

					while is_boundary(self.get(c.0, c.1).value()) && !self.at_end() {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}
				}
			}

			Command::Move(command::Move::Word(command::Word::Previous(times))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if !is_boundary(self.get(c.0, c.1).value()) {
						while !is_boundary(self.get(c.0, c.1).value()) && !self.at_start() {
							self.command(Command::Move(command::Move::Left(1)));
							c = overlay!(self; cursor);
						}
					}

					while is_boundary(self.get(c.0, c.1).value()) && !self.at_start() {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}

					while !is_boundary(self.get(c.0, c.1).value()) && !self.at_start() {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}

					if is_boundary(self.get(c.0, c.1).value()) && !self.at_start() {
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
						while is_boundary(self.get(c.0, c.1).value()) && !self.at_end() {
							self.command(Command::Move(command::Move::Right(1)));
							c = overlay!(self; cursor);
						}
					}

					while !is_boundary(self.get(c.0, c.1).value()) && !self.at_end() {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}

					if is_boundary(self.get(c.0, c.1).value()) && !self.at_end() {
						self.command(Command::Move(command::Move::Left(1)));
					}
				}
			}

			Command::Move(command::Move::Word(command::Word::PreviousEnd(times))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if !is_boundary(self.get(c.0, c.1).value()) {
						while !is_boundary(self.get(c.0, c.1).value()) && !self.at_start() {
							self.command(Command::Move(command::Move::Left(1)));
							c = overlay!(self; cursor);
						}
					}

					while is_boundary(self.get(c.0, c.1).value()) && !self.at_start() {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}
				}
			}

			Command::Move(command::Move::Until(command::Until::Next(times, ch))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if self.get(c.0, c.1).value() == ch {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}

					while self.get(c.0, c.1).value() != ch && !self.at_end() {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}
				}
			}

			Command::Move(command::Move::Until(command::Until::Previous(times, ch))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if self.get(c.0, c.1).value() == ch {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}

					while self.get(c.0, c.1).value() != ch && !self.at_start() {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}
				}
			}

			Command::Move(command::Move::Until(command::Until::NextBefore(times, ch))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if self.get(c.0, c.1).value() == ch {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}

					while self.get(c.0, c.1).value() != ch && !self.at_end() {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}

					if !self.at_end() {
						self.command(Command::Move(command::Move::Left(1)));
					}
				}
			}

			Command::Move(command::Move::Until(command::Until::PreviousBefore(times, ch))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if self.get(c.0, c.1).value() == ch {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}

					while self.get(c.0, c.1).value() != ch && !self.at_start() {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}

					if !self.at_start() {
						self.command(Command::Move(command::Move::Right(1)));
					}
				}
			}

			Command::Select(mode) => {
				let (name, old, new) = match (mode, self.selector.current.take()) {
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
					self.highlight(Highlight::Selection(&old), false);
				}

				if let Some(new) = new {
					self.selector.current = Some(new);
					self.highlight(Highlight::Selection(&new), true);
				}

				self.touched.all();
			}

			Command::Copy(name) => {
				if let Some(selection) = self.selector.current.take() {
					actions.push(Action::Overlay(false));
					actions.push(Action::Copy(name, self.selection(&selection)));
				}
			}

			Command::Paste(name) => {
				actions.push(Action::Overlay(false));
				actions.push(Action::Paste(name));
			}

			Command::Hint(command::Hint::Start) => {
				let top     = self.inner.rows() - 1 - if self.status.is_some() { 1 } else { 0 };
				let content = self.selection(&Selection::Line { start: top, end: 0 });
				let urls    = self.inner.config().environment().hinter().matcher()
					.find_iter(&content).collect::<Vec<_>>();

				if !urls.is_empty() {
					overlay!(self; status mode "HINT");

					self.hinter.hints = Some(Hints::new(self.inner.config().environment().hinter().label().to_vec(), urls.len()));

					for url in urls {
						self.hint(url, &content);
					}

					self.touched.all();
				}
				else {
					overlay!(self; status mode "NORMAL");
				}
			}

			Command::Hint(command::Hint::Pick(code)) => {
				let mut selected = self.hinter.current.clone().unwrap_or("".into());
				selected.push(code);

				// If no hints match, ignore the pick.
				if self.hinter.hints.as_ref().unwrap().iter().any(|(name, _)| name.starts_with(&selected)) {
					self.hinter.current  = Some(selected.clone());
					self.hinter.level   += 1;

					// If the wanted hint has been found.
					if self.hinter.hints.as_ref().unwrap().contains_key(&selected) {
						let level = self.hinter.level;

						// De-highlight every other hint and select the matching one.
						for (name, hint) in self.hinter.hints.clone().unwrap().into_inner() {
							if selected == name {
								self.highlight(Highlight::Hint(&hint, level, true), true);
							}
							else {
								self.highlight(Highlight::Hint(&hint, level, false), false);
							}
						}

						self.hinter.selected = Some(selected);
					}
					else {
						// De-highlight non-matching hints, and highlight matching ones.
						for (name, hint) in self.hinter.hints.clone().unwrap().into_inner() {
							let level = self.hinter.level;

							if name.starts_with(&selected) {
								self.highlight(Highlight::Hint(&hint, level, false), true);
							}
							else {
								self.highlight(Highlight::Hint(&hint, level, false), false);
							}
						}
					}

					self.touched.all();
				}
			}

			Command::Hint(command::Hint::Open) => {
				actions.push(Action::Overlay(false));

				if let Some(hint) = self.hinter.hints.as_ref().unwrap().get(self.hinter.selected.as_ref().unwrap()) {
					actions.push(Action::Open(hint.content.clone()));
				}
			}

			Command::Hint(command::Hint::Copy(name)) => {
				actions.push(Action::Overlay(false));

				if let Some(hint) = self.hinter.hints.as_ref().unwrap().get(self.hinter.selected.as_ref().unwrap()) {
					actions.push(Action::Copy(name, hint.content.clone()));
				}
			}
		}

		actions
	}

	/// Turn the current selection to its text representation.
	fn selection(&self, selection: &Selection) -> String {
		/// Find the index of the first non-empty cell followed by only empty
		/// cells.
		fn edge(row: &Row, start: u32, end: u32) -> u32 {
			let mut found = None;

			for x in start ... end {
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
		match *try!(return option self.selector.current.as_mut()) {
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

	/// Mark cells in a terminal as a hint.
	fn hint<T: AsRef<str>>(&mut self, (start, end): (usize, usize), content: T) {
		let content = content.as_ref();
		let url     = &content[start .. end];
		let hint    = if let Some(hints) = self.hinter.hints.as_mut() {
			let mut position = (None::<(u32, u32)>, None::<(u32, u32)>);
			let mut offset   = 0;
			let mut x        = 0;
			let mut y        = self.inner.rows() - 1 - if self.status.is_some() { 1 } else { 0 };

			for ch in content.graphemes(true) {
				if position.0.is_none() && offset == start {
					position.0 = Some((x, y));
				}

				if position.1.is_none() && offset == end {
					position.1 = Some((x, y));
					break;
				}

				offset += ch.len();
				x      += 1;

				if x >= self.inner.columns() || ch == "\n" {
					x  = 0;
					y -= 1;
				}
			}

			Some(hints.put((position.0.unwrap(), position.1.unwrap()), url).clone())
		}
		else {
			None
		};

		if let Some(hint) = hint {
			self.highlight(Highlight::Hint(&hint, 0, false), true);
		}
	}

	/// Enable or disable highlighting of the given selection.
	fn highlight(&mut self, what: Highlight, flag: bool) {
		match what {
			Highlight::Selection(&Selection::Normal { start, end }) => {
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
							cell.set_style(self.selector.style.clone());
							self.view.insert((x, y), cell);
						}
						else {
							self.view.remove(&(x, y));
						}
					}
				}
			}

			Highlight::Selection(&Selection::Block { start, end }) => {
				for y in end.1 ... start.1 {
					for x in start.0 ... end.0 {
						if flag {
							let mut cell = self.row(y)[x as usize].clone();
							cell.set_style(self.selector.style.clone());
							self.view.insert((x, y), cell);
						}
						else {
							self.view.remove(&(x, y));
						}
					}
				}
			}

			Highlight::Selection(&Selection::Line { start, end }) => {
				for y in end ... start {
					for x in 0 .. self.inner.columns() {
						if flag {
							let mut cell = self.row(y)[x as usize].clone();
							cell.set_style(self.selector.style.clone());
							self.view.insert((x, y), cell);
						}
						else {
							self.view.remove(&(x, y));
						}
					}
				}
			}

			Highlight::Hint(hint, level, selected) => {
				let (mut x, mut y) = hint.position.0;

				// Add the label, if the level permits.
				for ch in hint.name.graphemes(true).skip(level) {
					if flag {
						self.view.insert((x, y), Cell::occupied(ch.into(), self.hinter.label.clone()));
					}
					else {
						self.view.remove(&(x, y));
					}

					x += 1;
					if x >= self.inner.columns() {
						x  = 0;
						y -= 1;
					}
				}

				// Make the URL underscored or selected.
				while (x, y) != hint.position.1 {
					if flag {
						let mut cell = self.row(y)[x as usize].clone();

						if selected {
							cell.set_style(self.hinter.hinted.clone());
						}
						else {
							cell.set_style(self.hinter.underlined.clone());
						}

						self.view.insert((x, y), cell);
					}
					else {
						self.view.remove(&(x, y));
					}

					x += 1;
					if x >= self.inner.columns() {
						x  = 0;
						y -= 1;
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
