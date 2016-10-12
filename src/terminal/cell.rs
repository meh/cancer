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
use std::cell::{RefCell, Ref, RefMut};
use std::any::Any;
use std::mem;
use std::rc::Rc;
use unicode_width::UnicodeWidthStr;

use style::Style;

#[derive(Debug)]
pub enum Cell {
	Empty {
		style: Rc<Style>,
	},

	Occupied {
		style: Rc<Style>,
		value: String,
		data:  RefCell<Box<Any>>,
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
			data:  RefCell::new(Box::new(())),
		}
	}

	/// Create a referencing cell.
	pub fn reference(offset: u8) -> Self {
		Cell::Reference(offset)
	}

	/// Check if the cell is empty.
	pub fn is_empty(&self) -> bool {
		if let &Cell::Empty { .. } = self {
			true
		}
		else {
			false
		}
	}

	/// Check if the cell is occupied.
	pub fn is_occupied(&self) -> bool {
		if let &Cell::Occupied { .. } = self {
			true
		}
		else {
			false
		}
	}

	/// Check if the cell is a reference.
	pub fn is_reference(&self) -> bool {
		if let &Cell::Reference(..) = self {
			true
		}
		else {
			false
		}
	}

	/// Check if the cell is wide.
	pub fn is_wide(&self) -> bool {
		match self {
			&Cell::Empty { .. } =>
				false,

			&Cell::Occupied { ref value, .. } =>
				value.width() > 1,

			&Cell::Reference(..) =>
				unreachable!()
		}
	}

	/// Make the cell empty.
	pub fn into_empty(&mut self, style: Rc<Style>) {
		mem::replace(self, Cell::Empty {
			style: style,
		});
	}

	/// Make the cell occupied.
	pub fn into_occupied(&mut self, value: String, style: Rc<Style>) {
		mem::replace(self, Cell::Occupied {
			value: value,
			style: style,
			data:  RefCell::new(Box::new(())),
		});
	}

	/// Make the cell into a reference.
	pub fn into_reference(&mut self, offset: u8) {
		mem::replace(self, Cell::Reference(offset));
	}

	/// Get the cell style.
	pub fn style(&self) -> &Rc<Style> {
		match self {
			&Cell::Empty { ref style, .. } =>
				style,

			&Cell::Occupied { ref style, .. } =>
				style,

			&Cell::Reference(..) =>
				unreachable!(),
		}
	}

	/// Get the value if any.
	pub fn value(&self) -> &str {
		match self {
			&Cell::Empty { .. } =>
				" ",

			&Cell::Occupied { ref value, .. } =>
				value,

			_ =>
				unreachable!()
		}
	}

	/// Get the data as immutable.
	pub fn data(&self) -> Ref<Box<Any>> {
		match self {
			&Cell::Occupied { ref data, .. } =>
				data.borrow(),

			_ =>
				unreachable!()
		}
	}

	/// Get the data as mutable.
	pub fn data_mut(&self) -> RefMut<Box<Any>> {
		match self {
			&Cell::Occupied { ref data, .. } =>
				data.borrow_mut(),

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
		match self {
			&Cell::Reference(offset) =>
				offset as u32,

			_ =>
				unreachable!()
		}
	}
}

impl Clone for Cell {
	fn clone(&self) -> Self {
		match self {
			&Cell::Empty { ref style } => {
				Cell::Empty {
					style: style.clone()
				}
			}

			&Cell::Reference(offset) => {
				Cell::Reference(offset)
			}

			&Cell::Occupied { ref style, ref value, .. } => {
				Cell::Occupied {
					style: style.clone(),
					value: value.clone(),
					data:  RefCell::new(Box::new(())),
				}
			}
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
