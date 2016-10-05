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
use unicode_width::UnicodeWidthStr;

use style::Style;

#[derive(PartialEq, Clone, Debug)]
pub struct Cell {
	x: u32,
	y: u32,

	style: Rc<Style>,
	state: State,
}

#[derive(PartialEq, Clone, Debug)]
pub enum State {
	Empty,

	Reference {
		x: u32,
		y: u32,
	},

	Char {
		value: String,
		wrap:  bool,
	}
}

impl Cell {
	pub fn new(x: u32, y: u32, style: Rc<Style>) -> Self {
		Cell {
			x: x,
			y: y,

			style: style,
			state: State::Empty
		}
	}

	pub fn x(&self) -> u32 {
		self.x
	}

	pub fn y(&self) -> u32 {
		self.y
	}

	pub fn wrap(&self) -> bool {
		match &self.state {
			&State::Char { wrap, .. } =>
				wrap,

			_ =>
				false
		}
	}

	pub fn is_empty(&self) -> bool {
		if let &State::Empty = &self.state {
			true
		}
		else {
			false
		}
	}

	pub fn is_reference(&self) -> bool {
		if let &State::Reference { .. } = &self.state {
			true
		}
		else {
			false
		}
	}

	pub fn is_character(&self) -> bool {
		if let &State::Char { .. } = &self.state {
			true
		}
		else {
			false
		}
	}

	pub fn make_char(&mut self, ch: String, wrap: bool) {
		self.state = State::Char {
			value: ch,
			wrap:  wrap,
		}
	}

	pub fn make_reference(&mut self, x: u32, y: u32) {
		self.state = State::Reference {
			x: x,
			y: y,
		}
	}

	pub fn style(&self) -> &Style {
		&self.style
	}
	
	pub fn state(&self) -> &State {
		&self.state
	}

	pub fn char(&self) -> Option<&str> {
		match &self.state {
			&State::Char { ref value, .. } =>
				Some(value),

			_ =>
				None
		}
	}

	pub fn width(&self) -> u32 {
		match &self.state {
			&State::Empty =>
				1,

			&State::Char { ref value, .. } =>
				value.width() as u32,

			&State::Reference { .. } =>
				0,
		}
	}
}
