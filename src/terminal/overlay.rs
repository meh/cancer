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

	($term:ident; cursor $($travel:tt)*) => ({
		$term.touched.push($term.cursor.position());
		let r = $term.cursor.travel(cursor::$($travel)*);
		$term.touched.push($term.cursor.position());
		r
	});
}

#[derive(Debug)]
pub struct Overlay {
	inner:   Terminal,
	cache:   Vec<u8>,
	touched: Touched,

	scroll:  u32,
	cursor:  Cursor,
	changed: HashMap<(u32, u32), Cell, BuildHasherDefault<FnvHasher>>,

	times:  Option<u32>,
	select: Option<Region>,
}

impl Overlay {
	pub fn new(inner: Terminal) -> Self {
		let mut cursor = inner.cursor.clone();
		{
			let tmp = cursor.foreground;
			cursor.foreground = cursor.background;
			cursor.background = tmp;
		}

		Overlay {
			inner:   inner,
			touched: Touched::default(),
			cache:   Vec::new(),

			scroll:  0,
			cursor:  cursor,
			changed: Default::default(),

			times:  None,
			select: None,
		}
	}

	pub fn into_inner<W: Write>(mut self, output: W) -> error::Result<Terminal> {
		try!(self.inner.handle(self.cache, output));
		Ok(self.inner)
	}

	pub fn get(&self, x: u32, y: u32) -> &Cell {
		let back   = self.inner.grid.back();
		let view   = self.inner.grid.view();
		let offset = (view.len() as u32 - 1 - y) + self.scroll;

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

	pub fn get_mut(&mut self, x: u32, y: u32) -> &mut Cell {
		let view   = self.inner.grid.view();
		let offset = (view.len() as u32 - 1 - y) + self.scroll;

		if !self.changed.contains_key(&(x, offset)) {
			let cell = self.get(x, y).clone();
			self.changed.insert((x, offset), cell);
		}

		self.changed.get_mut(&(x, offset)).unwrap()
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
		use platform::key::{Value, Button};

		let mut actions = Vec::new();

		if let Value::Char(ref ch) = *key.value() {
			if let Ok(number) = ch.parse::<u32>() {
				if let Some(times) = self.times.take() {
					self.times = Some(times * 10 + number);
				}
				else {
					self.times = Some(number);
				}

				return (actions.into_iter(), self.touched.iter(self.inner.region));
			}
		}

		for _ in 0 .. self.times.unwrap_or(1) {
			match *key.value() {
				Value::Char(ref ch) => match &**ch {
					// C-y
					"\x19" => {
						if self.scroll < self.inner.grid.back().len() as u32 {
							self.touched.all();
							self.scroll += 1;
						}
					}
	
					// C-e
					"\x05" => {
						if self.scroll > 0 {
							self.touched.all();
							self.scroll -= 1;
						}
					}
	
					"h" => {
						if overlay!(self; cursor Left(1)).is_some() {
							overlay!(self; cursor Up(1));
							overlay!(self; cursor Position(Some(self.inner.region.width - 1), None));
						}
					}
	
					"j" => {
						overlay!(self; cursor Down(1));
					}
	
					"k" => {
						overlay!(self; cursor Up(1));
					}
	
					"l" => {
						if overlay!(self; cursor Right(1)).is_some() {
							overlay!(self; cursor Down(1));
							overlay!(self; cursor Position(Some(0), None));
						}
					}
	
					_ => ()
				},

				Value::Button(ref button) => match button {
					_ => ()
				},

				Value::Keypad(ref button) => match button {
					_ => ()
				},
			}
		}

		self.times = None;
		(actions.into_iter(), self.touched.iter(self.inner.region()))
	}

	pub fn handle<I: AsRef<[u8]>>(&mut self, input: I) {
		self.cache.extend(input.as_ref());
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
