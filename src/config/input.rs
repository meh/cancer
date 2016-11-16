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

use toml;
use platform::{Key, key};

#[derive(PartialEq, Clone, Debug)]
pub struct Input {
	prefix: Key,
	mouse:  bool,
}

impl Default for Input {
	fn default() -> Self {
		Input {
			prefix: Key::new("a".to_string().into(), key::LOGO, Default::default()),
			mouse:  true,
		}
	}
}

impl Input {
	pub fn load(&mut self, table: &toml::Table) {
		if let Some(value) = table.get("prefix").and_then(|v| v.as_str()) {
			self.prefix = to_key(value);
		}

		if let Some(value) = table.get("mouse").and_then(|v| v.as_bool()) {
			self.mouse = value;
		}
	}

	pub fn prefix(&self) -> &Key {
		&self.prefix
	}

	pub fn mouse(&self) -> bool {
		self.mouse
	}
}

fn to_key<T: AsRef<str>>(value: T) -> Key {
	let     value     = value.as_ref();
	let mut modifiers = value.split('-').collect::<Vec<&str>>();
	let     button    = modifiers.pop().unwrap().to_lowercase();

	let modifiers = modifiers.iter().fold(Default::default(), |acc, modifier|
		match *modifier {
			"C" => acc | key::CTRL,
			"A" => acc | key::ALT,
			"S" => acc | key::SHIFT,
			"L" => acc | key::LOGO,
			_   => acc,
		});

	let key = match &*button {
		"esc" =>
			key::Button::Escape.into(),

		"backspace" | "bs" =>
			key::Button::Backspace.into(),

		"enter" | "return" =>
			key::Button::Enter.into(),

		"delete" | "del" =>
			key::Button::Delete.into(),

		"insert" | "ins" =>
			key::Button::Insert.into(),

		"home" =>
			key::Button::Home.into(),

		"end" =>
			key::Button::End.into(),

		"pageup" | "pagup" | "pup" | "previous" | "prev" | "prior" =>
			key::Button::PageUp.into(),

		"pagedown" | "pagdown" | "pdown" | "next" =>
			key::Button::PageDown.into(),

		"up" =>
			key::Button::Up.into(),

		"down" =>
			key::Button::Down.into(),

		"right" =>
			key::Button::Right.into(),

		"left" =>
			key::Button::Left.into(),

		"menu" =>
			key::Button::Menu.into(),

		_ =>
			button.into()
	};

	Key::new(key, modifiers, Default::default())
}
