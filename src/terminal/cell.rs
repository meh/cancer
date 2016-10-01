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

	inner: String,
	style: Rc<Style>,
}

impl Cell {
	pub fn new(x: u32, y: u32, inner: String, style: Rc<Style>) -> Self {
		Cell {
			x: x,
			y: y,

			inner: inner,
			style: style,
		}
	}

	pub fn x(&self) -> u32 {
		self.x
	}

	pub fn y(&self) -> u32 {
		self.y
	}

	pub fn width(&self) -> u32 {
		self.inner.width() as u32
	}

	pub fn style(&self) -> &Style {
		&self.style
	}
}

impl AsRef<str> for Cell {
	fn as_ref(&self) -> &str {
		&self.inner
	}
}
