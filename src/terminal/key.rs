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

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Key {
	modifier: Modifier,
	value:    Value,
}

bitflags! {
	pub flags Modifier: u8 {
		const ALT   = 1 << 0,
		const CAPS  = 1 << 1,
		const CTRL  = 1 << 2,
		const LOGO  = 1 << 4,
		const NUM   = 1 << 5,
		const SHIFT = 1 << 6,
	}
}

impl Default for Modifier {
	fn default() -> Self {
		Modifier::empty()
	}
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Value {
	String(String),
	Button(Button),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Button {
	Enter,
	Escape,

	Up,
	Down,
	Right,
	Left,
}

impl From<Button> for Value {
	fn from(value: Button) -> Value {
		Value::Button(value)
	}
}

impl From<String> for Value {
	fn from(value: String) -> Value {
		Value::String(value)
	}
}

impl Key {
	pub fn new(value: Value, modifier: Modifier) -> Self {
		Key {
			modifier: modifier,
			value:    value,
		}
	}

	pub fn write<W: Write>(&self, mut output: W) -> io::Result<()> {
		macro_rules! write {
			(raw $raw:expr) => (
				try!(output.write_all($raw));
			);

			($item:expr) => (
				($item.into(): Control).fmt(output.by_ref(), false)?;
			);
		}

		match self.value {
			Value::String(ref string) => {
				try!(output.write_all(string.as_bytes()));
			}

			Value::Button(Button::Enter) => {
				write!(C0::CarriageReturn);
			}

			Value::Button(Button::Escape) => {
				write!(C0::Escape);
			}

			Value::Button(Button::Up) => {
				write!(raw b"\x1BOA");
			}

			Value::Button(Button::Down) => {
				write!(raw b"\x1BOB");
			}

			Value::Button(Button::Right) => {
				write!(raw b"\x1BOC");
			}

			Value::Button(Button::Left) => {
				write!(raw b"\x1BOD");
			}
		}

		Ok(())
	}
}
