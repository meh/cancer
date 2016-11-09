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

use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::collections::HashMap;
use std::vec;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;
use picto::Region;

use error;
use platform::{Key, key};
use terminal::{Access, Action, Terminal, Cursor, Iter};
use terminal::touched::{self, Touched};
use terminal::cell::{self, Cell};
use terminal::cursor;
use terminal::grid;
use overlay::{Status, Selection};
use overlay::command::{self, Command};

pub type Changed = HashMap<(u32, u32), Cell, BuildHasherDefault<FnvHasher>>;

#[derive(Debug)]
pub struct Overlay {
	inner:   Terminal,
	cache:   Vec<u8>,
	touched: Touched,

	scroll:  u32,
	cursor:  Cursor,
	changed: Changed,
	status:  Option<Status>,

	prefix: Option<u8>,
	times:  Option<u32>,
	select: Option<Selection>,
}

macro_rules! overlay {
	($term:ident; cursor) => ({
		let x = $term.cursor.x();
		let y = $term.cursor.y();

		if let Cell::Reference(offset) = *overlay!($term; cell (x, y)) {
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
		let before = overlay!($term; cursor absolute);
		$term.touched.push($term.cursor.position());

		let r = $term.cursor.travel(cursor::$($travel)*);

		let after = overlay!($term; cursor absolute);
		$term.touched.push($term.cursor.position());

		overlay!($term; select before, after);

		r
	});

	($term:ident; select $before:expr, $after:expr) => ({
		if let Some(select) = $term.select.as_mut() {
			Overlay::select(select, &mut $term.changed, $before, $after);
			overlay!($term; touched all);
		}
	});

	($term:ident; unselect) => ({
		Overlay::unselect($term.select.take().unwrap(), &mut $term.changed);
		overlay!($term; touched all);
	});

	($term:ident; status mode $name:expr) => ({
		if let Some(status) = $term.status.as_mut() {
			overlay!($term; touched line ($term.inner.rows()) - 1);
			status.mode($name);
		}
	});

	($term:ident; status position!) => ({
		if $term.status.is_some() {
			let (x, y) = overlay!($term; cursor absolute);
			let x      = x + 1;
			let y      = $term.inner.grid().back().len() as u32 + $term.inner.grid().view().len() as u32 - y;
			overlay!($term; status position (x, y));
		}
	});

	($term:ident; status position $pair:expr) => ({
		if let Some(status) = $term.status.as_mut() {
			overlay!($term; touched line ($term.inner.rows()) - 1);
			status.position($pair);
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

	($term:ident; cell ($x:expr, $y:expr)) => ({
		$term.get($x, $y)
	});

	($term:ident; mut cell ($x:expr, $y:expr)) => ({
		let view   = $term.inner.grid().view();
		let offset = (view.len() as u32 - 1 - y) + $term.scroll;

		if !$term.changed.contains_key(&(x, offset)) {
			let cell = $term.get(x, y).clone();
			$term.changed.insert((x, offset), cell);
		}

		$term.changed.get_mut(&(x, offset)).unwrap()
	});
}

impl Overlay {
	pub fn new(inner: Terminal) -> Self {
		let mut cursor = inner.cursor().clone();
		{
			let tmp = cursor.foreground;
			cursor.foreground = cursor.background;
			cursor.background = tmp;
		}

		let status = inner.config().style().status().map(|c| {
			cursor.travel(cursor::Up(1));
			cursor.scroll = (0, inner.rows() - 2);

			let mut status = Status::new(c, inner.columns());
			status.mode("NORMAL");

			let (x, y) = cursor.position();
			let y      = inner.grid().back().len() as u32 + y + 2;
			status.position((x, y));

			status
		});;

		Overlay {
			inner:   inner,
			touched: Touched::default(),
			cache:   Vec::new(),

			scroll:  0,
			cursor:  cursor,
			changed: Default::default(),
			status:  status,

			prefix: None,
			times:  None,
			select: None,
		}
	}

	pub fn into_inner<W: Write>(mut self, output: W) -> error::Result<Terminal> {
		try!(self.inner.input(self.cache, output));
		Ok(self.inner)
	}

	pub fn get(&self, x: u32, y: u32) -> &Cell {
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

		if let Some(cell) = self.changed.get(&(x, offset)) {
			return cell;
		}

		if offset as usize >= view.len() {
			&back[back.len() - 1 - (offset as usize - view.len())][x as usize]
		}
		else {
			&view[view.len() - 1 - offset as usize][x as usize]
		}
	}

	pub fn row(&self, y: u32) -> &grid::Row {
		let back = self.inner.grid().back();
		let view = self.inner.grid().view();

		if y as usize >= view.len() {
			&back[back.len() - 1 - (y as usize - view.len())]
		}
		else {
			&view[view.len() - 1 - y as usize]
		}
	}

	pub fn cursor(&self) -> cursor::Cell {
		let (x, y) = overlay!(self; cursor);
		cursor::Cell::new(&self.cursor, cell::Position::new(x, y, self.get(x, y)))
	}

	/// Get an iterator over positioned cells.
	pub fn iter<'a, T: Iterator<Item = (u32, u32)>>(&'a self, iter: T) -> Iter<Self, T> {
		Iter::new(self, iter)
	}

	pub fn key(&mut self, key: Key) -> (vec::IntoIter<Action>, touched::Iter) {
		let command = self.command(key);
		let actions = self.handle(command);

		(actions.into_iter(), self.touched.iter(self.inner.region()))
	}

	pub fn input<I: AsRef<[u8]>>(&mut self, input: I) {
		self.cache.extend(input.as_ref());
	}

	fn handle(&mut self, command: Command) -> Vec<Action> {
		let mut actions = Vec::new();

		match command {
			Command::None => (),

			Command::Scroll(command::Scroll::Up(times)) => {
				for _ in 0 .. times {
					let offset = if self.status.is_some() { 1 } else { 0 };

					if self.scroll < self.inner.grid().back().len() as u32 + offset {
						self.scroll += 1;
					}
				}

				overlay!(self; touched all);
				overlay!(self; status position!);
			}

			Command::Scroll(command::Scroll::Down(times)) => {
				for _ in 0 .. times {
					if self.scroll > 0 {
						self.scroll -= 1;
					}
				}

				overlay!(self; touched all);
				overlay!(self; status position!);
			}

			Command::Scroll(command::Scroll::PageUp(times)) => {
				for _ in 0 .. times {
					self.scroll += self.inner.rows().saturating_sub(3);
			
					if self.scroll > self.inner.grid().back().len() as u32 {
						self.scroll = self.inner.grid().back().len().saturating_sub(1) as u32;
					}
				}

				overlay!(self; touched all);
				overlay!(self; status position!);
			}

			Command::Scroll(command::Scroll::PageDown(times)) => {
				for _ in 0 .. times {
					self.scroll = self.scroll.saturating_sub(self.inner.rows() - 3);
				}
	
				overlay!(self; touched all);
				overlay!(self; status position!);
			}

			Command::Move(command::Move::End) => {
				overlay!(self; cursor Position(Some(self.inner.columns() - 1), None));
				overlay!(self; status position!);
			}

			Command::Move(command::Move::Start) => {
				overlay!(self; cursor Position(Some(0), None));
				overlay!(self; status position!);
			}

			Command::Move(command::Move::Left(times)) => {
				for _ in 0 ..times {
					if overlay!(self; cursor Left(1)).is_some() {
						if let Some(&Selection::Normal { .. }) = self.select.as_ref() {
							if overlay!(self; cursor Up(1)).is_some() {
								self.handle(Command::Scroll(command::Scroll::Up(1)));
							}

							overlay!(self; cursor Position(Some(self.inner.columns() - 1), None));
						}
					}
				}

				overlay!(self; status position!);
			}

			Command::Move(command::Move::Down(times)) => {
				for _ in 0 .. times {
					if overlay!(self; cursor Down(1)).is_some() {
						self.handle(Command::Scroll(command::Scroll::Down(1)));
					}
				}

				overlay!(self; status position!);
			}

			Command::Move(command::Move::Up(times)) => {
				for _ in 0 .. times {
					if overlay!(self; cursor Up(1)).is_some() {
						self.handle(Command::Scroll(command::Scroll::Up(1)));
					}
				}

				overlay!(self; status position!);
			}

			Command::Move(command::Move::Right(times)) => {
				for _ in 0 .. times {
					if overlay!(self; cursor Right(1)).is_some() {
						if let Some(&Selection::Normal { .. }) = self.select.as_ref() {
							if overlay!(self; cursor Down(1)).is_some() {
								self.handle(Command::Scroll(command::Scroll::Down(1)));
							}

							overlay!(self; cursor Position(Some(0), None));
						}
					}
				}

				overlay!(self; status position!);
			}

			Command::Scroll(command::Scroll::Begin) => {
				self.scroll = self.inner.grid().back().len() as u32;

				overlay!(self; touched all);
				overlay!(self; status position!);
			}

			Command::Scroll(command::Scroll::End) => {
				self.scroll = 0;

				overlay!(self; touched all);
				overlay!(self; status position!);
			}

			Command::Scroll(command::Scroll::To(n)) => {
				self.scroll = (self.inner.grid().back().len() as u32).saturating_sub(n - 1);

				if self.status.is_some() {
					self.scroll += 1;
				}

				overlay!(self; touched all);
				overlay!(self; status position!);
			}

			Command::Select(command::Select::Normal) => {
				match self.select.take() {
					Some(Selection::Normal { .. }) => {
						overlay!(self; status mode "NORMAL");
					}

					Some(Selection::Block(region)) => {
						overlay!(self; status mode "VISUAL BLOCK");

						// TODO: convert from `Block` to `Normal`.
					}

					None => {
						overlay!(self; status mode "VISUAL");

						let (x, y) = overlay!(self; cursor absolute);
						self.select = Some(Selection::Normal { start: (x, y), end: (x + 1, y) });
					}
				}
			}

			Command::Select(command::Select::Block) => {
				match self.select.take() {
					Some(Selection::Block(..)) => {
						overlay!(self; status mode "NORMAL");
					}

					Some(Selection::Normal { start, end }) => {
						overlay!(self; status mode "VISUAL");

						// TODO: convert from `Normal` to `Block`.
					}

					None => {
						overlay!(self; status mode "VISUAL BLOCK");

						let (x, y) = overlay!(self; cursor absolute);
						self.select = Some(Selection::Block(Region::from(x, y, 1, 1)));
					}
				}
			}

			Command::Copy => {
				if let Some(selection) = self.selection() {
					overlay!(self; status mode "NORMAL");
					actions.push(Action::Clipboard("CLIPBOARD".into(), selection));
					overlay!(self; unselect);
				}
			}
		}

		if let Some(selection) = self.selection() {
			actions.push(Action::Clipboard("PRIMARY".into(), selection));
		}

		actions
	}

	fn command(&mut self, key: Key) -> Command {
		use platform::key::{Value, Button, Keypad};

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

					return Command::None;
				}
			}
		}

		let new    = self.prefix.is_none();
		let times  = self.times.take();
		let prefix = self.prefix.take();

		let command = match *key.value() {
			Value::Char(ref ch) => match &**ch {
				"\x19" | "e" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::Up(times.unwrap_or(1))),

				"\x05" | "e" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::Down(times.unwrap_or(1))),

				"\x15" | "u" if key.modifier() == key::CTRL =>
					Command::Scroll(command::Scroll::PageUp(times.unwrap_or(1))),

				"\x04" | "d" if key.modifier() == key::CTRL =>
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

				// Scroll to the top.
				"g" if key.modifier().is_empty() && prefix.is_none() => {
					self.prefix = Some(b'g');
					Command::None
				}

				"g" if key.modifier().is_empty() && prefix == Some(b'g') => {
					Command::Scroll(command::Scroll::Begin)
				}

				// Scroll to the end.
				"G" if key.modifier() == key::SHIFT => {
					if let Some(times) = times {
						Command::Scroll(command::Scroll::To(times))
					}
					else {
						Command::Scroll(command::Scroll::End)
					}
				}

				// Region selection.
				"v" if key.modifier().is_empty() => {
					Command::Select(command::Select::Normal)
				}

				// Block selection.
				"\x16" | "v" if key.modifier() == key::CTRL => {
					Command::Select(command::Select::Block)
				}

				"y" if key.modifier().is_empty() => {
					Command::Copy
				}

				_ => {
					debug!(target: "cancer::terminal::overlay::unhandled", "key {:?}", key);
					Command::None
				}
			},

			Value::Button(ref button) => match button {
				&Button::PageUp => 
					Command::Scroll(command::Scroll::PageUp(times.unwrap_or(1))),

				&Button::PageDown =>
					Command::Scroll(command::Scroll::PageDown(times.unwrap_or(1))),

				&Button::Left =>
					Command::Move(command::Move::Left(times.unwrap_or(1))),

				&Button::Down =>
					Command::Move(command::Move::Down(times.unwrap_or(1))),

				&Button::Up =>
					Command::Move(command::Move::Up(times.unwrap_or(1))),

				&Button::Right =>
					Command::Move(command::Move::Right(times.unwrap_or(1))),

				_ => {
					debug!(target: "cancer::terminal::overlay::unhandled", "key {:?}", key);
					Command::None
				}
			},

			Value::Keypad(ref button) => match button {
				&Keypad::Left =>
					Command::Move(command::Move::Left(times.unwrap_or(1))),

				&Keypad::Down =>
					Command::Move(command::Move::Down(times.unwrap_or(1))),

				&Keypad::Up =>
					Command::Move(command::Move::Up(times.unwrap_or(1))),

				&Keypad::Right =>
					Command::Move(command::Move::Right(times.unwrap_or(1))),

				_ => {
					debug!(target: "cancer::terminal::overlay::unhandled", "key {:?}", key);
					Command::None
				}
			},
		};

		// Only remove the prefix if it hadn't just been set.
		if self.prefix.is_some() && !new {
			self.prefix = None;
		}

		command
	}

	fn selection(&self) -> Option<String> {
		/// Find the index of the first non-empty cell followed by only empty
		/// cells.
		fn edge(row: &grid::Row, start: u32, end: u32) -> u32 {
			let mut found = None;

			for x in start .. end {
				let cell = &row[x as usize];

				if cell.is_empty() && found.is_none() {
					found = Some(x)
				}
				else if cell.is_occupied() && found.is_some() {
					found = None;
				}
			}

			found.unwrap_or(end)
		}

		match self.select {
			Some(Selection::Normal { start, end }) => {
				let mut lines  = vec![];
				let mut unwrap = None::<Vec<String>>;

				for y in (start.1 ... end.1).rev() {
					let (start, end) = if start.1 == end.1 {
						(start.0, end.0)
					}
					else if y == start.1 {
						(start.0, self.inner.columns())
					}
					else if y == end.1 {
						(0, end.0)
					}
					else {
						(0, self.inner.columns())
					};

					let     row  = self.row(y);
					let mut line = String::new();

					for x in start .. edge(row, start, end) {
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
				for lines in lines {
					for line in lines.into_iter().rev() {
						result.push_str(&line);
					}

					result.push('\n');
				}
				result.pop();

				Some(result)
			}

			Some(Selection::Block(region)) => {
				let mut result = String::new();
				for y in region.y .. region.y + region.height {
					let row = self.row(y);

					for x in region.x .. edge(row, region.x, region.x + region.width) {
						result.push_str(row[x as usize].value());
					}

					result.push('\n');
				}
				result.pop();

				Some(result)
			}

			None =>
				None
		}
	}

	fn select(selection: &mut Selection, changed: &mut Changed, before: (u32, u32), after: (u32, u32)) {
		match *selection {
			Selection::Normal { ref mut start, ref mut end } => {
				// TODO: it
			}

			Selection::Block(ref mut region) => {
				// TODO: it
			}
		}
	}

	fn unselect(selection: Selection, changed: &mut Changed) {
		match selection {
			Selection::Normal { start, end } => {
				// TODO: it
			}

			Selection::Block(region) => {
				// TODO: it
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
