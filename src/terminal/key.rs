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
use terminal::{mode, Mode};

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
	Keypad(Keypad),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Button {
	Escape,
	Backspace,
	Enter,
	Delete,
	Insert,
	Home,
	End,

	PageUp,
	PageDown,

	Up,
	Down,
	Right,
	Left,

	F1,
	F2,
	F3,
	F4,
	F5,
	F6,
	F7,
	F8,
	F9,
	F10,
	F11,
	F12,
	F13,
	F14,
	F15,
	F16,
	F17,
	F18,
	F19,
	F20,
	F21,
	F22,
	F23,
	F24,
	F25,
	F26,
	F27,
	F28,
	F29,
	F30,
	F31,
	F32,
	F33,
	F34,
	F35,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Keypad {
	Enter,
	Home,
	Begin,
	End,
	Insert,

	Multiply,
	Add,
	Subtract,
	Divide,
	Decimal,

	PageUp,
	PageDown,

	Up,
	Down,
	Right,
	Left,

	Number(u8),
}

impl From<String> for Value {
	fn from(value: String) -> Value {
		Value::String(value)
	}
}

impl From<Button> for Value {
	fn from(value: Button) -> Value {
		Value::Button(value)
	}
}

impl From<Keypad> for Value {
	fn from(value: Keypad) -> Value {
		Value::Keypad(value)
	}
}

impl Key {
	pub fn new(value: Value, modifier: Modifier) -> Self {
		Key {
			modifier: modifier,
			value:    value,
		}
	}

	pub fn write<W: Write>(&self, mode: Mode, mut output: W) -> io::Result<()> {
		macro_rules! write {
			() => ();

			(_ # $($modes:ident)|+ => $string:expr, $($rest:tt)*) => ({
				if mode.contains($(mode::$modes)|*) {
					return output.write_all($string);
				}

				write!($($rest)*)
			});

			(_ => $string:expr,) => ({
				output.write_all($string)
			});

			($($modifier:ident)|+ # $($modes:ident)|+ => $string:expr, $($rest:tt)*) => ({
				if self.modifier.contains($($modifier)|*) && mode.contains($(mode::$modes)|*) {
					return output.write_all($string);
				}

				write!($($rest)*)
			});

			($($modifier:ident)|+ => $string:expr, $($rest:tt)*) => ({
				if self.modifier.contains($($modifier)|*) {
					return output.write_all($string);
				}

				write!($($rest)*)
			});
		}

		match self.value {
			Value::String(ref string) => {
				if self.modifier.contains(ALT) {
					try!(output.write_all(b"\x1B"));
				}

				output.write_all(string.as_bytes())
			}

			Value::Button(Button::Escape) => write! {
				_ => b"\x1B",
			},

			Value::Button(Button::Backspace) => write! {
				ALT => b"\x1B\x7F",
				_   => b"\x7F",
			},

			Value::Button(Button::Enter) => write! {
				ALT # CRLF => b"\x1B\r\n",
				ALT        => b"\x1B\r",

				_ # CRLF => b"\r\n",
				_        => b"\r",
			},

			Value::Button(Button::Delete) => write! {
				CTRL # APPLICATION_KEYPAD => b"\x1B[3;5~",
				CTRL                      => b"\x1B[M",

				SHIFT # APPLICATION_KEYPAD => b"\x1B[3;2~",
				SHIFT                      => b"\x1B[2K",

				_ # APPLICATION_KEYPAD => b"\x1B[3~",
				_                      => b"\x1B[P",
			},

			Value::Button(Button::Insert) => write! {
				CTRL # APPLICATION_KEYPAD => b"\x1B[2;5~",
				CTRL                      => b"\x1B[L",

				SHIFT # APPLICATION_KEYPAD => b"\x1B[2;2~",
				SHIFT                      => b"\x1B[4l",

				_ # APPLICATION_KEYPAD => b"\x1B[2~",
				_                      => b"\x1B[M",
			},

			Value::Button(Button::Home) => write! {
				SHIFT # APPLICATION_CURSOR => b"\x1B[1;2H",
				SHIFT                      => b"\x1B[2J",

				_ # APPLICATION_CURSOR => b"\x1B[7~",
				_                      => b"\x1B[H",
			},

			Value::Button(Button::End) => write! {
				CTRL # APPLICATION_KEYPAD => b"\x1B[1;5F",
				CTRL                      => b"\x1B[J",

				SHIFT # APPLICATION_KEYPAD => b"\x1B[1;2F",
				SHIFT                      => b"\x1B[K",

				_ => b"\x1B[8~",
			},

			Value::Button(Button::PageUp) => write! {
				CTRL  => b"\x1B[5;5~",
				SHIFT => b"\x1B[5;2~",
				_     => b"\x1B[5~",
			},

			Value::Button(Button::PageDown) => write! {
				CTRL  => b"\x1B[6;5~",
				SHIFT => b"\x1B[6;2~",
				_     => b"\x1B[6~",
			},

			Value::Button(Button::Up) => write! {
				CTRL  => b"\x1B[1;5A",
				ALT   => b"\x1B[1;3A",
				SHIFT => b"\x1B[1;2A",

				_ # APPLICATION_CURSOR => b"\x1BOA",
				_                      => b"\x1B[A",
			},

			Value::Button(Button::Down) => write! {
				CTRL  => b"\x1B[1;5B",
				ALT   => b"\x1B[1;3B",
				SHIFT => b"\x1B[1;2B",

				_ # APPLICATION_CURSOR => b"\x1BOB",
				_                      => b"\x1B[B",
			},

			Value::Button(Button::Right) => write! {
				CTRL  => b"\x1B[1;5C",
				ALT   => b"\x1B[1;3C",
				SHIFT => b"\x1B[1;2C",

				_ # APPLICATION_CURSOR => b"\x1BOC",
				_                      => b"\x1B[C",
			},

			Value::Button(Button::Left) => write! {
				CTRL  => b"\x1B[1;5D",
				ALT   => b"\x1B[1;3D",
				SHIFT => b"\x1B[1;2D",

				_ # APPLICATION_CURSOR => b"\x1BOD",
				_                      => b"\x1B[D",
			},

			Value::Button(Button::F1) => write! {
				CTRL  => b"\x1B[1;5P",
				ALT   => b"\x1B[1;3P",
				LOGO  => b"\x1B[1;6P",
				SHIFT => b"\x1B[1;2P",
				_     => b"\x1BOP",
			},

			Value::Button(Button::F2) => write! {
				CTRL  => b"\x1B[1;5Q",
				ALT   => b"\x1B[1;3Q",
				LOGO  => b"\x1B[1;6Q",
				SHIFT => b"\x1B[1;2Q",
				_     => b"\x1BOQ",
			},

			Value::Button(Button::F3) => write! {
				CTRL  => b"\x1B[1;5R",
				ALT   => b"\x1B[1;3R",
				LOGO  => b"\x1B[1;6R",
				SHIFT => b"\x1B[1;2R",
				_     => b"\x1BOR",
			},

			Value::Button(Button::F4) => write! {
				CTRL  => b"\x1B[1;5S",
				ALT   => b"\x1B[1;3S",
				LOGO  => b"\x1B[1;6S",
				SHIFT => b"\x1B[1;2S",
				_     => b"\x1BOS",
			},

			Value::Button(Button::F5) => write! {
				CTRL  => b"\x1B[15;5~",
				ALT   => b"\x1B[15;3~",
				LOGO  => b"\x1B[15;6~",
				SHIFT => b"\x1B[15;2~",
				_     => b"\x1B[15~",
			},

			Value::Button(Button::F6) => write! {
				CTRL  => b"\x1B[17;5~",
				ALT   => b"\x1B[17;3~",
				LOGO  => b"\x1B[17;6~",
				SHIFT => b"\x1B[17;2~",
				_     => b"\x1B[17~",
			},

			Value::Button(Button::F7) => write! {
				CTRL  => b"\x1B[18;5~",
				ALT   => b"\x1B[18;3~",
				LOGO  => b"\x1B[18;6~",
				SHIFT => b"\x1B[18;2~",
				_     => b"\x1B[18~",
			},

			Value::Button(Button::F8) => write! {
				CTRL  => b"\x1B[19;5~",
				ALT   => b"\x1B[19;3~",
				LOGO  => b"\x1B[19;6~",
				SHIFT => b"\x1B[19;2~",
				_     => b"\x1B[19~",
			},

			Value::Button(Button::F9) => write! {
				CTRL  => b"\x1B[20;5~",
				ALT   => b"\x1B[20;3~",
				LOGO  => b"\x1B[20;6~",
				SHIFT => b"\x1B[20;2~",
				_     => b"\x1B[20~",
			},

			Value::Button(Button::F10) => write! {
				CTRL  => b"\x1B[21;5~",
				ALT   => b"\x1B[21;3~",
				LOGO  => b"\x1B[21;6~",
				SHIFT => b"\x1B[21;2~",
				_     => b"\x1B[21~",
			},

			Value::Button(Button::F11) => write! {
				CTRL  => b"\x1B[23;5~",
				ALT   => b"\x1B[23;3~",
				LOGO  => b"\x1B[23;6~",
				SHIFT => b"\x1B[23;2~",
				_     => b"\x1B[23~",
			},

			Value::Button(Button::F12) => write! {
				CTRL  => b"\x1B[24;5~",
				ALT   => b"\x1B[24;3~",
				LOGO  => b"\x1B[24;6~",
				SHIFT => b"\x1B[24;2~",
				_     => b"\x1B[24~",
			},

			Value::Button(Button::F13) => write! {
				_ => b"\x1B[1;2P",
			},

			Value::Button(Button::F14) => write! {
				_ => b"\x1B[1;2Q",
			},

			Value::Button(Button::F15) => write! {
				_ => b"\x1B[1;2R",
			},

			Value::Button(Button::F16) => write! {
				_ => b"\x1B[1;2S",
			},

			Value::Button(Button::F17) => write! {
				_ => b"\x1B[15;2~",
			},

			Value::Button(Button::F18) => write! {
				_ => b"\x1B[17;2~",
			},

			Value::Button(Button::F19) => write! {
				_ => b"\x1B[18;2~",
			},

			Value::Button(Button::F20) => write! {
				_ => b"\x1B[19;2~",
			},

			Value::Button(Button::F21) => write! {
				_ => b"\x1B[20;2~",
			},

			Value::Button(Button::F22) => write! {
				_ => b"\x1B[21;2~",
			},

			Value::Button(Button::F23) => write! {
				_ => b"\x1B[23;2~",
			},

			Value::Button(Button::F24) => write! {
				_ => b"\x1B[24;2~",
			},

			Value::Button(Button::F25) => write! {
				_ => b"\x1B[1;5P",
			},

			Value::Button(Button::F26) => write! {
				_ => b"\x1B[1;5Q",
			},

			Value::Button(Button::F27) => write! {
				_ => b"\x1B[1;5R",
			},

			Value::Button(Button::F28) => write! {
				_ => b"\x1B[1;5S",
			},

			Value::Button(Button::F29) => write! {
				_ => b"\x1B[15;5~",
			},

			Value::Button(Button::F30) => write! {
				_ => b"\x1B[17;5~",
			},

			Value::Button(Button::F31) => write! {
				_ => b"\x1B[18;5~",
			},

			Value::Button(Button::F32) => write! {
				_ => b"\x1B[19;5~",
			},

			Value::Button(Button::F33) => write! {
				_ => b"\x1B[20;5~",
			},

			Value::Button(Button::F34) => write! {
				_ => b"\x1B[21;5~",
			},

			Value::Button(Button::F35) => write! {
				_ => b"\x1B[23;5~",
			},

			Value::Keypad(..) =>
				unimplemented!(),
		}
	}
}
