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

use std::ops::Index;
use std::io::Write;
use std::vec;

use picto::Region;
use error;
use config::Config;
use platform::{Key, Mouse};
use terminal::{Terminal, Mode, Iter, Cell};
use terminal::{cursor, touched};
use overlay::Overlay;

#[derive(Debug)]
pub enum Interface {
	Terminal(Terminal),
	Overlay(Overlay),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Action {
	Urgent,
	Overlay(bool),
	Title(String),
	Resize(u32, u32),
	Copy(String, String),
	Paste(String),
	Open(String),
}

impl Interface {
	pub fn config(&self) -> &Config {
		match *self {
			Interface::Terminal(ref terminal) =>
				terminal.config(),

			Interface::Overlay(ref overlay) =>
				overlay.config(),
		}
	}

	pub fn overlay(&self) -> bool {
		if let Interface::Terminal(..) = *self {
			false
		}
		else {
			true
		}
	}

	pub fn into_inner<W: Write>(self, output: W) -> error::Result<Terminal> {
		match self {
			Interface::Terminal(terminal) =>
				Ok(terminal),

			Interface::Overlay(overlay) =>
				overlay.into_inner(output)
		}
	}

	pub fn cursor(&self) -> cursor::Cell {
		match *self {
			Interface::Terminal(ref terminal) =>
				terminal.cursor(),

			Interface::Overlay(ref overlay) =>
				overlay.cursor(),
		}
	}

	pub fn columns(&self) -> u32 {
		match *self {
			Interface::Terminal(ref terminal) =>
				terminal.columns(),

			Interface::Overlay(ref overlay) =>
				overlay.columns(),
		}
	}

	pub fn rows(&self) -> u32 {
		match *self {
			Interface::Terminal(ref terminal) =>
				terminal.rows(),

			Interface::Overlay(ref overlay) =>
				overlay.rows(),
		}
	}

	pub fn mode(&self) -> Mode {
		match *self {
			Interface::Terminal(ref terminal) =>
				terminal.mode(),

			Interface::Overlay(ref overlay) =>
				overlay.mode(),
		}
	}

	pub fn region(&self) -> Region {
		match *self {
			Interface::Terminal(ref terminal) =>
				terminal.region(),

			Interface::Overlay(ref overlay) =>
				overlay.region(),
		}
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		match *self {
			Interface::Terminal(ref mut terminal) =>
				terminal.resize(width, height),

			Interface::Overlay(_) =>
				unreachable!()
		}
	}

	pub fn blinking(&mut self, value: bool) -> touched::Iter {
		match *self {
			Interface::Terminal(ref mut terminal) =>
				terminal.blinking(value),

			Interface::Overlay(ref mut overlay) =>
				overlay.blinking(value),
		}
	}

	pub fn iter<T: Iterator<Item = (u32, u32)>>(&self, iter: T) -> Iter<Self, T> {
		Iter::new(self, iter)
	}

	pub fn focus<O: Write>(&mut self, value: bool, output: O) -> error::Result<()> {
		if let Interface::Terminal(ref mut terminal) = *self {
			try!(terminal.focus(value, output));
		}

		Ok(())
	}

	pub fn paste<O: Write>(&mut self, value: &[u8], output: O) -> error::Result<()> {
		match *self {
			Interface::Terminal(ref mut terminal) =>
				try!(terminal.paste(value, output)),

			Interface::Overlay(ref mut overlay) =>
				try!(overlay.paste(value, output)),
		}

		Ok(())
	}

	pub fn key<O: Write>(&mut self, key: Key, output: O) -> error::Result<(vec::IntoIter<Action>, touched::Iter)> {
		if &key == self.config().input().prefix() {
			return Ok((vec![Action::Overlay(!self.overlay())].into_iter(), touched::Iter::empty()));
		}

		match *self {
			Interface::Terminal(ref mut terminal) => {
				try!(terminal.key(key, output));
				Ok((Vec::new().into_iter(), touched::Iter::empty()))
			}

			Interface::Overlay(ref mut overlay) => {
				Ok(overlay.key(key))
			}
		}
	}

	pub fn mouse<O: Write>(&mut self, mouse: Mouse, output: O) -> error::Result<(vec::IntoIter<Action>, touched::Iter)> {
		if !self.config().input().mouse() {
			return Ok((Vec::new().into_iter(), touched::Iter::empty()));
		}

		match *self {
			Interface::Terminal(ref mut terminal) => {
				try!(terminal.mouse(mouse, output));
				Ok((Vec::new().into_iter(), touched::Iter::empty()))
			}

			Interface::Overlay(ref mut overlay) => {
				Ok(overlay.mouse(mouse))
			}
		}
	}

	pub fn input<I: AsRef<[u8]>, O: Write>(&mut self, input: I, output: O) -> error::Result<(vec::IntoIter<Action>, touched::Iter)> {
		match *self {
			Interface::Terminal(ref mut terminal) => {
				terminal.input(input, output)
			}

			Interface::Overlay(ref mut overlay) => {
				overlay.input(input);
				Ok((Vec::new().into_iter(), touched::Iter::empty()))
			}
		}
	}
}

impl Index<(u32, u32)> for Interface {
	type Output = Cell;

	fn index(&self, (x, y): (u32, u32)) -> &Cell {
		match *self {
			Interface::Terminal(ref terminal) =>
				&terminal[(x, y)],

			Interface::Overlay(ref overlay) =>
				&overlay[(x, y)],
		}
	}
}

impl From<Terminal> for Interface {
	fn from(value: Terminal) -> Self {
		Interface::Terminal(value)
	}
}

impl From<Overlay> for Interface {
	fn from(value: Overlay) -> Self {
		Interface::Overlay(value)
	}
}
