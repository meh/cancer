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
	Char(String),
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
		Value::Char(value)
	}
}

impl From<char> for Value {
	fn from(value: char) -> Value {
		let mut string = String::new();
		string.push(value);

		Value::Char(string)
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

	pub fn modifier(&self) -> Modifier {
		self.modifier
	}

	pub fn value(&self) -> &Value {
		&self.value
	}
}
