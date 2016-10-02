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
pub enum Cell {
	Empty {
		x: u32,
		y: u32,

		style: Rc<Style>,
	},

	Char {
		x: u32,
		y: u32,

		value: String,
		style: Rc<Style>,
	},

	Reference {
		x: u32,
		y: u32,
	},
}

impl Cell {
	pub fn x(&self) -> u32 {
		match self {
			&Cell::Empty { x, .. } |
			&Cell::Char { x, .. } |
			&Cell::Reference { x, .. } =>
				x
		}
	}

	pub fn y(&self) -> u32 {
		match self {
			&Cell::Empty { y, .. } |
			&Cell::Char { y, .. } |
			&Cell::Reference { y, .. } =>
				y
		}
	}

	pub fn width(&self) -> u32 {
		match self {
			&Cell::Empty { .. } =>
				1,

			&Cell::Char { ref value, .. } =>
				value.width() as u32,

			&Cell::Reference { .. } =>
				unreachable!(),
		}
	}

	pub fn style(&self) -> &Style {
		match self {
			&Cell::Empty { ref style, .. } |
			&Cell::Char { ref style, .. } =>
				style,

			&Cell::Reference { .. } =>
				unreachable!(),
		}
	}
}
