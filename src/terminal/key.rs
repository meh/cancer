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

use std::io::{self, Write};
use control::{Control, C0, C1, CSI, Format};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Key {
	Enter,
	Escape,

	Up,
	Down,
	Right,
	Left,
}

impl Key {
	pub fn write<W: Write>(&self, mut output: W) -> io::Result<()> {
		macro_rules! write {
			(raw $raw:expr) => (
				try!(output.write_all($raw));
			);

			($item:expr) => (
				try!(($item.into(): Control).fmt(output.by_ref(), false));
			);
		}

		match *self {
			Key::Enter =>
				write!(C0::LineFeed),

			Key::Escape =>
				write!(C0::Escape),

			Key::Up =>
				write!(raw b"\x1BOA"),

			Key::Down =>
				write!(raw b"\x1BOB"),

			Key::Right =>
				write!(raw b"\x1BOC"),

			Key::Left =>
				write!(raw b"\x1BOD"),
		}

		Ok(())
	}
}
