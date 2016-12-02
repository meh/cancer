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
use std::ops::{Index, Deref, DerefMut};
use std::vec;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

use unicode_segmentation::UnicodeSegmentation;
use error;
use config::overlay as config;
use style::{self, Style};
use platform::Clipboard;
use platform::key::{self, Key};
use platform::mouse::{self, Mouse};
use terminal::{Terminal, Cursor, Iter, Row};
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

unsafe impl Send for Overlay { }

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

	config: config::Hinter,
	label:  Rc<Style>,
	hinted: Rc<Style>,
}

impl Hinter {
	pub fn get(&self) -> Option<&str> {
		if let (Some(hints), Some(selected)) = (self.hints.as_ref(), self.selected.as_ref()) {
			hints.get(selected).map(|v| &*v.content)
		}
		else {
			None
		}
	}
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
	Hint(&'a Hint, usize),
}

macro_rules! overlay {
	($term:ident; cursor) => ({
		let x = $term.cursor.x();
		let y = $term.cursor.y();

		if let Cell::Reference(offset) = $term[(x, y)] {
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
			let config = inner.config().overlay().cursor();

			cursor.foreground = *config.foreground();
			cursor.background = *config.background();
			cursor.shape      = config.shape();
			cursor.state.insert(cursor::VISIBLE);

			if config.blink() {
				cursor.state.insert(cursor::BLINK);
			}
			else {
				cursor.state.remove(cursor::BLINK);
			}
		}

		let status = inner.config().overlay().status().map(|c| {
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
			style:   Rc::new(*inner.config().overlay().selection()),
		};

		let hinter = Hinter {
			selected: None,
			current:  None,
			level:    0,
			hints:    None,

			config:  inner.config().overlay().hinter(0).clone(),
			label:   Rc::new(Style::default()),
			hinted:  Rc::new(Style::default()),
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

	/// Get the current cursor position.
	pub fn cursor(&self) -> cursor::Cell {
		let (x, y) = overlay!(self; cursor);
		cursor::Cell::new(&self.cursor, cell::Position::new(x, y, &self[(x, y)]))
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

		fn is_boundary(ch: &str) -> bool {
			!ch.chars().any(|c| c.is_alphabetic() || c.is_numeric())
		}

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
				// Hint handling.
				"u" if key.modifier().is_empty() && self.hinter.hints.is_none() =>
					Command::Hint(command::Hint::Start(times.unwrap_or(0))),

				"o" if key.modifier().is_empty() && self.hinter.selected.is_some() =>
					Command::Hint(command::Hint::Open),

				"y" if key.modifier().is_empty() && self.hinter.selected.is_some() =>
					Command::Hint(command::Hint::Copy(match times {
						Some(1) => Clipboard::Primary,
						Some(2) => Clipboard::Secondary,
						_       => Clipboard::default(),
					})),

				ch if key.modifier().is_empty() && self.hinter.hints.is_some() && self.hinter.selected.is_none() =>
					Command::Hint(command::Hint::Pick(ch.chars().next().unwrap())),

				// Prefix based operations.
				"e" if key.modifier().is_empty() && prefix == Some(b'g') =>
					Command::Move(command::Move::Previous(times.unwrap_or(1),
						command::Previous::Word(command::Word::End(box is_boundary)))),

				"g" if key.modifier().is_empty() && prefix == Some(b'g') =>
					Command::Scroll(command::Scroll::Begin),

				ch if prefix == Some(b'f') =>
					Command::Move(command::Move::Next(times.unwrap_or(1),
						command::Next::Match(command::Match::After(ch.into())))),

				ch if prefix == Some(b'F') =>
					Command::Move(command::Move::Previous(times.unwrap_or(1),
						command::Previous::Match(command::Match::After(ch.into())))),

				ch if prefix == Some(b't') =>
					Command::Move(command::Move::Next(times.unwrap_or(1),
						command::Next::Match(command::Match::Before(ch.into())))),

				ch if prefix == Some(b'T') =>
					Command::Move(command::Move::Previous(times.unwrap_or(1),
						command::Previous::Match(command::Match::Before(ch.into())))),

				// Keys for exits.
				"c" if key.modifier() == key::CTRL =>
					Command::Exit,

				"i" if key.modifier().is_empty() =>
					Command::Exit,

				"q" if key.modifier().is_empty() =>
					Command::Exit,

				// Scrolling commands.
				"y" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::Up(times.unwrap_or(1))),

				"e" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::Down(times.unwrap_or(1))),

				"u" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::PageUp(times.unwrap_or(1))),

				"d" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::PageDown(times.unwrap_or(1))),

				"G" if key.modifier() == key::SHIFT => {
					if let Some(times) = times {
						Command::Scroll(command::Scroll::To(times))
					}
					else {
						Command::Scroll(command::Scroll::End)
					}
				}

				// Cursor movement commands.
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
					Command::Move(command::Move::Next(times.unwrap_or(1),
						command::Next::Word(command::Word::Start(box is_boundary)))),

				"b" if key.modifier().is_empty() =>
					Command::Move(command::Move::Previous(times.unwrap_or(1),
						command::Previous::Word(command::Word::Start(box is_boundary)))),

				"e" if key.modifier().is_empty() =>
					Command::Move(command::Move::Next(times.unwrap_or(1),
						command::Next::Word(command::Word::End(box is_boundary)))),

				// Selection commands.
				"v" if key.modifier().is_empty() =>
					Command::Select(command::Select::Normal),

				"v" if key.modifier() == key::CTRL =>
					Command::Select(command::Select::Block),

				"V" if key.modifier() == key::SHIFT =>
					Command::Select(command::Select::Line),

				"y" if key.modifier().is_empty() =>
					Command::Copy(match times {
						Some(1) => Clipboard::Primary,
						Some(2) => Clipboard::Secondary,
						_       => Clipboard::default(),
					}.into()),

				"p" if key.modifier().is_empty() =>
					Command::Paste(match times {
						Some(1) => Clipboard::Primary,
						Some(2) => Clipboard::Secondary,
						_       => Clipboard::default(),
					}.into()),

				// Prefix setters.
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
				// Exit commands.
				Button::Escape if key.modifier().is_empty() =>
					Command::Exit,

				// Ignore keys while in hint mode.
				_ if self.hinter.hints.is_some() =>
					Command::None,

				// Scrolling commands.
				Button::PageUp =>
					Command::Scroll(command::Scroll::PageUp(times.unwrap_or(1))),

				Button::PageDown =>
					Command::Scroll(command::Scroll::PageDown(times.unwrap_or(1))),

				// Cursor movement commands.
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

				// Selection commands.
				Button::Insert if key.modifier() == key::SHIFT =>
					Command::Paste(Clipboard::Primary),

				_ => {
					debug!(target: "cancer::overlay::unhandled", "key {:?}", key);
					Command::None
				}
			},

			Value::Keypad(button) => match button {
				// Ignore keys while in hint mode.
				_ if self.hinter.hints.is_some() =>
					Command::None,

				// Cursor movement commands.
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
			// Ignore mouse events while in hint mode.
			_ if self.hinter.hints.is_some() =>
				Command::None,

			// Scrolling commands.
			Mouse::Click(mouse::Click { button: mouse::Button::Up, .. }) =>
				Command::Scroll(command::Scroll::Up(1)),

			Mouse::Click(mouse::Click { button: mouse::Button::Down, .. }) =>
				Command::Scroll(command::Scroll::Down(1)),

			// Cursor movement commands.
			Mouse::Click(mouse::Click { button: mouse::Button::Left, press: false, position, .. }) =>
				Command::Move(command::Move::To(position.x, position.y)),

			// Selection commands.
			Mouse::Click(mouse::Click { button: mouse::Button::Middle, press: false, .. }) =>
				Command::Paste(Clipboard::Primary),

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
				actions.push(Action::Copy(Clipboard::Primary, self.selection(&selection)));
			}
		}

		actions
	}

	fn command(&mut self, command: Command) -> Vec<Action> {
		let mut actions = Vec::new();

		match command {
			Command::None => (),

			// Handle exit, exit the overlay if there's no current mode, otherwise
			// exit the mode.
			Command::Exit => {
				overlay!(self; status mode "NORMAL");

				if let Some(selection) = self.selector.current.take() {
					self.highlight(Highlight::Selection(&selection), false);
				}
				else if let Some(hints) = self.hinter.hints.take() {
					for hint in hints.values() {
						self.highlight(Highlight::Hint(hint, 0), false);
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

			// Scrolling commands.
			Command::Scroll(command::Scroll::Begin) => {
				self.scroll = self.inner.grid().back().len() as u32
					+ if self.status.is_some() { 1 } else { 0 };

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

			// Cursor movement commands.
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

			Command::Move(command::Move::Next(times, command::Next::Word(command::Word::Start(boundary)))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if !boundary(self[(c.0, c.1)].value()) {
						while !boundary(self[(c.0, c.1)].value()) && !self.at_end() {
							self.command(Command::Move(command::Move::Right(1)));
							c = overlay!(self; cursor);
						}
					}

					while boundary(self[(c.0, c.1)].value()) && !self.at_end() {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}
				}
			}

			Command::Move(command::Move::Previous(times, command::Previous::Word(command::Word::Start(boundary)))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if !boundary(self[(c.0, c.1)].value()) {
						while !boundary(self[(c.0, c.1)].value()) && !self.at_start() {
							self.command(Command::Move(command::Move::Left(1)));
							c = overlay!(self; cursor);
						}
					}

					while boundary(self[(c.0, c.1)].value()) && !self.at_start() {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}

					while !boundary(self[(c.0, c.1)].value()) && !self.at_start() {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}

					if boundary(self[(c.0, c.1)].value()) && !self.at_start() {
						self.command(Command::Move(command::Move::Right(1)));
					}
				}
			}

			Command::Move(command::Move::Next(times, command::Next::Word(command::Word::End(boundary)))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if !boundary(self[(c.0, c.1)].value()) {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}

					if boundary(self[(c.0, c.1)].value()) {
						while boundary(self[(c.0, c.1)].value()) && !self.at_end() {
							self.command(Command::Move(command::Move::Right(1)));
							c = overlay!(self; cursor);
						}
					}

					while !boundary(self[(c.0, c.1)].value()) && !self.at_end() {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}

					if boundary(self[(c.0, c.1)].value()) && !self.at_end() {
						self.command(Command::Move(command::Move::Left(1)));
					}
				}
			}

			Command::Move(command::Move::Previous(times, command::Previous::Word(command::Word::End(boundary)))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if !boundary(self[(c.0, c.1)].value()) {
						while !boundary(self[(c.0, c.1)].value()) && !self.at_start() {
							self.command(Command::Move(command::Move::Left(1)));
							c = overlay!(self; cursor);
						}
					}

					while boundary(self[(c.0, c.1)].value()) && !self.at_start() {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}
				}
			}

			Command::Move(command::Move::Next(times, command::Next::Match(command::Match::After(ch)))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if self[(c.0, c.1)].value() == ch {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}

					while self[(c.0, c.1)].value() != ch && !self.at_end() {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}
				}
			}

			Command::Move(command::Move::Previous(times, command::Previous::Match(command::Match::After(ch)))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if self[(c.0, c.1)].value() == ch {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}

					while self[(c.0, c.1)].value() != ch && !self.at_start() {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}
				}
			}

			Command::Move(command::Move::Next(times, command::Next::Match(command::Match::Before(ch)))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if self[(c.0, c.1)].value() == ch {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}

					while self[(c.0, c.1)].value() != ch && !self.at_end() {
						self.command(Command::Move(command::Move::Right(1)));
						c = overlay!(self; cursor);
					}

					if !self.at_end() {
						self.command(Command::Move(command::Move::Left(1)));
					}
				}
			}

			Command::Move(command::Move::Previous(times, command::Previous::Match(command::Match::Before(ch)))) => {
				for _ in 0 .. times {
					let mut c = overlay!(self; cursor);

					if self[(c.0, c.1)].value() == ch {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}

					while self[(c.0, c.1)].value() != ch && !self.at_start() {
						self.command(Command::Move(command::Move::Left(1)));
						c = overlay!(self; cursor);
					}

					if !self.at_start() {
						self.command(Command::Move(command::Move::Right(1)));
					}
				}
			}

			// Selection commands.
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

			// Hint handling.
			Command::Hint(command::Hint::Start(id)) => {
				let bottom = self.scroll;
				let top    = self.inner.rows() - 1
					- if self.status.is_some() { 1 } else { 0 }
					+ self.scroll;

				let config  = self.inner.config().overlay().hinter(id).clone();
				let content = self.selection(&Selection::Line { start: top, end: bottom });
				let urls    = config.matcher().find_iter(&content).collect::<Vec<_>>();

				if !urls.is_empty() {
					overlay!(self; status mode "HINT");

					self.hinter.label  = Rc::new(*config.style());
					self.hinter.hinted = Rc::new(Style {
						foreground: config.style().foreground,
						background: config.style().background,
						attributes: config.style().attributes ^ style::REVERSE,
					});

					self.hinter.hints  = Some(Hints::new(config.label().to_vec(), urls.len()));
					self.hinter.config = config;

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
								self.highlight(Highlight::Hint(&hint, level), true);
							}
							else {
								self.highlight(Highlight::Hint(&hint, level), false);
							}
						}

						self.hinter.selected = Some(selected);
						actions.push(Action::Copy(Clipboard::Primary, self.hinter.get().unwrap().into()));
					}
					else {
						// De-highlight non-matching hints, and highlight matching ones.
						for (name, hint) in self.hinter.hints.clone().unwrap().into_inner() {
							let level = self.hinter.level;

							if name.starts_with(&selected) {
								self.highlight(Highlight::Hint(&hint, level), true);
							}
							else {
								self.highlight(Highlight::Hint(&hint, level), false);
							}
						}
					}

					self.touched.all();
				}
			}

			Command::Hint(command::Hint::Open) => {
				actions.push(Action::Overlay(false));

				if let Some(hint) = self.hinter.get() {
					actions.push(Action::Open(self.hinter.config.opener().map(String::from), hint.into()));
				}
			}

			Command::Hint(command::Hint::Copy(name)) => {
				actions.push(Action::Overlay(false));

				if let Some(hint) = self.hinter.get() {
					actions.push(Action::Copy(name, hint.into()));
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

				// Iterate in reverse on the rows, so wrapped lines can be unwrapped.
				for y in end.1 ... start.1 {
					// Adapt the horizontal edges based on the vertical position.
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

					let     row  = &self[y];
					let mut line = String::new();

					// Fill the current line.
					for x in start ... edge(row, start, end) {
						line.push_str(row[x as usize].value());
					}

					// If the row is wrapped, push it up.
					if row.wrap() {
						if let Some(mut unwrapped) = unwrap.take() {
							unwrapped.push(line);
							unwrap = Some(unwrapped);
						}
						else {
							unwrap = Some(vec![line]);
						}
					}
					// If the row is not wrapped and we have unwrapped rows, it means
					// it's the original row which had been wrapped.
					else if let Some(mut unwrapped) = unwrap.take() {
						unwrapped.push(line);
						lines.push(unwrapped);
					}
					// Otherwise it's just a line.
					else {
						lines.push(vec![line]);
					}
				}

				let mut result = String::new();

				// Collect up the lines in reverse order, which happens to be the
				// original order.
				for lines in lines.into_iter().rev() {
					// Collect up any wrapped lines, in reverse order, which again is the
					// original order.
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

				// Iterate in proper order.
				for y in (end.1 ... start.1).rev() {
					let row = &self[y];

					// Collect up from edge to edge.
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
	///
	/// TODO(meh): simplify this, if at all possible.
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
			let mut graphemes = content.graphemes(true).peekable();
			let mut position  = (None::<(u32, u32)>, None::<(u32, u32)>);
			let mut offset    = 0;
			let mut x         = 0;
			let mut y         = self.inner.rows() - 1
				- if self.status.is_some() { 1 } else { 0 }
				+ self.scroll;

			while let Some(ch) = graphemes.next() {
				if position.0.is_none() && offset >= start {
					position.0 = Some((x, y));
				}

				if position.1.is_none() && offset >= end {
					position.1 = Some((x, y));
					break;
				}

				offset  += ch.len();
				x       += 1;

				// Avoid offsetting multiple times because of a completely filled row.
				if x >= self.inner.columns() && ch != "\n" && graphemes.peek() == Some(&"\n") {
					offset += 1;
					graphemes.next();
				}

				// If it's a newline or we're beyond a wrapped line.
				if ch == "\n" || x >= self.inner.columns() {
					x  = 0;
					y -= 1;
				}
			}

			// If no final position was reached, it means it goes to the end of the
			// input.
			Some(hints.put((position.0.unwrap(), position.1.unwrap_or((x, y))),
				url.replace('\n', "")).clone())
		}
		else {
			None
		};

		if let Some(hint) = hint {
			self.highlight(Highlight::Hint(&hint, 0), true);
		}
	}

	/// Enable or disable highlighting of the given selection.
	fn highlight(&mut self, what: Highlight, flag: bool) {
		match what {
			Highlight::Selection(&Selection::Normal { start, end }) => {
				// Adapt the horizontal edges based on the vertical position.
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
							let mut cell = self[y][x as usize].clone();
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
							let mut cell = self[y][x as usize].clone();
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
							let mut cell = self[y][x as usize].clone();
							cell.set_style(self.selector.style.clone());
							self.view.insert((x, y), cell);
						}
						else {
							self.view.remove(&(x, y));
						}
					}
				}
			}

			Highlight::Hint(hint, level) => {
				let (mut x, mut y) = hint.position.0;

				// Add the label, if the level permits.
				for ch in hint.label.graphemes(true).skip(level) {
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
						let mut cell = self[y][x as usize].clone();
						cell.set_style(self.hinter.hinted.clone());
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

impl Index<(u32, u32)> for Overlay {
	type Output = Cell;

	fn index(&self, (x, y): (u32, u32)) -> &Cell {
		// If there's a status bar and the requested row is the last one, return
		// the cell from the status.
		if let Some(status) = self.status.as_ref() {
			if y == self.inner.rows() - 1 {
				return &status[x as usize];
			}
		}

		let back = self.inner.grid().back();
		let view = self.inner.grid().view();

		// The internal Y coordinate starts from the bottom, as 0, and increases
		// going up.
		let mut offset = (view.len() as u32 - 1 - y) + self.scroll;

		// If there's a status bar, the actual offset has to be adapted to ignore
		// the last line.
		if self.status.is_some() {
			offset -= 1;
		}

		// If the cell was changed within the overlay, return that.
		if let Some(cell) = self.view.get(&(x, offset)) {
			return cell;
		}

		// Get the proper cell between the current view and the scroll back.
		if offset as usize >= view.len() {
			&back[back.len() - 1 - (offset as usize - view.len())][x as usize]
		}
		else {
			&view[view.len() - 1 - offset as usize][x as usize]
		}
	}
}

impl Index<u32> for Overlay {
	type Output = Row;

	fn index(&self, y: u32) -> &Row {
		let back = self.inner.grid().back();
		let view = self.inner.grid().view();

		if y as usize >= view.len() {
			&back[back.len() - 1 - (y as usize - view.len())]
		}
		else {
			&view[view.len() - 1 - y as usize]
		}
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
