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

use std::ops::Deref;
use std::mem;
use std::rc::Rc;
use unicode_width::UnicodeWidthStr;

use style::Style;

#[derive(Clone, Debug)]
pub enum Cell {
	Empty {
		style: Rc<Style>,
	},

	Occupied {
		style: Rc<Style>,
		value: String,
	},

	Reference(u8),
}

#[derive(Copy, Clone, Debug)]
pub struct Position<'a> {
	x: u32,
	y: u32,

	inner: &'a Cell,
}

impl Cell {
	/// Create an empty cell.
	pub fn empty(style: Rc<Style>) -> Self {
		Cell::Empty {
			style: style
		}
	}

	/// Create an occupied cell.
	pub fn occupied(value: String, style: Rc<Style>) -> Self {
		Cell::Occupied {
			value: value,
			style: style,
		}
	}

	/// Create a referencing cell.
	pub fn reference(offset: u8) -> Self {
		Cell::Reference(offset)
	}

	/// Check if the cell is empty.
	pub fn is_empty(&self) -> bool {
		if let Cell::Empty { .. } = *self {
			true
		}
		else {
			false
		}
	}

	/// Check if the cell is occupied.
	pub fn is_occupied(&self) -> bool {
		if let Cell::Occupied { .. } = *self {
			true
		}
		else {
			false
		}
	}

	/// Check if the cell is a reference.
	pub fn is_reference(&self) -> bool {
		if let Cell::Reference(..) = *self {
			true
		}
		else {
			false
		}
	}

	/// Check if the cell is wide.
	pub fn is_wide(&self) -> bool {
		match *self {
			Cell::Empty { .. } =>
				false,

			Cell::Occupied { ref value, .. } =>
				value.width() > 1,

			Cell::Reference(..) =>
				unreachable!()
		}
	}

	/// Make the cell empty.
	pub fn make_empty(&mut self, style: Rc<Style>) {
		mem::replace(self, Cell::Empty {
			style: style,
		});
	}

	/// Make the cell occupied.
	pub fn make_occupied<T: Into<String>>(&mut self, value: T, style: Rc<Style>) {
		mem::replace(self, Cell::Occupied {
			value: value.into(),
			style: style,
		});
	}

	/// Make the cell into a reference.
	pub fn make_reference(&mut self, offset: u8) {
		mem::replace(self, Cell::Reference(offset));
	}

	/// Get the cell style.
	pub fn style(&self) -> &Rc<Style> {
		match *self {
			Cell::Empty { ref style, .. } |
			Cell::Occupied { ref style, .. } =>
				style,

			Cell::Reference(..) =>
				unreachable!(),
		}
	}

	/// Get the value if any.
	pub fn value(&self) -> &str {
		match *self {
			Cell::Empty { .. } =>
				" ",

			Cell::Occupied { ref value, .. } =>
				value,

			_ =>
				unreachable!()
		}
	}

	/// Get the cell width.
	pub fn width(&self) -> u32 {
		self.value().width() as u32
	}

	/// Get the reference offset.
	pub fn offset(&self) -> u32 {
		match *self {
			Cell::Reference(offset) =>
				offset as u32,

			_ =>
				unreachable!()
		}
	}
}

impl<'a> Position<'a> {
	pub fn new(x: u32, y: u32, inner: &Cell) -> Position {
		Position {
			x: x,
			y: y,

			inner: inner
		}
	}

	pub fn x(&self) -> u32 {
		self.x
	}

	pub fn y(&self) -> u32 {
		self.y
	}
}

impl<'a> Deref for Position<'a> {
	type Target = Cell;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}
