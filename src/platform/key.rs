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

#[derive(Eq, Clone, Debug)]
pub struct Key {
	value:    Value,
	modifier: Modifier,
	lock:     Lock,
}

/// Implementation to ignore locks, they're just informational.
impl PartialEq for Key {
	fn eq(&self, other: &Key) -> bool {
		self.modifier == other.modifier && self.value == other.value
	}
}

bitflags! {
	pub flags Modifier: u8 {
		const ALT   = 1 << 0,
		const CTRL  = 1 << 2,
		const LOGO  = 1 << 4,
		const SHIFT = 1 << 6,
	}
}

impl Default for Modifier {
	fn default() -> Self {
		Modifier::empty()
	}
}

bitflags! {
	pub flags Lock: u8 {
		const CAPS = 1 << 0,
		const NUM  = 1 << 1,
	}
}

impl Default for Lock {
	fn default() -> Self {
		Lock::empty()
	}
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Value {
	Char(String),
	Button(Button),
	Keypad(Keypad),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Button {
	Tab,
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

	F(u8),
	Menu,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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
	pub fn new(value: Value, modifier: Modifier, lock: Lock) -> Self {
		Key {
			value:    value,
			modifier: modifier,
			lock:     lock,
		}
	}

	/// Get the value.
	pub fn value(&self) -> &Value {
		&self.value
	}

	/// Get the active modifiers.
	pub fn modifier(&self) -> Modifier {
		self.modifier
	}

	/// Get the active locks.
	pub fn lock(&self) -> Lock {
		self.lock
	}
}
