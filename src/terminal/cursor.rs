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
use std::ops::Deref;

use style::{self, Style};
use terminal::{Cell, Dirty};

#[derive(PartialEq, Clone, Default, Debug)]
pub struct Cursor {
	position: (u32, u32),
	limits:   (u32, u32),
	style:    Rc<Style>,
}

pub enum Travel {
	Position(Option<u32>, Option<u32>),

	Up(u32),
	Down(u32),
	Left(u32),
	Right(u32),
}

pub use self::Travel::*;

impl Cursor {
	pub fn new(width: u32, height: u32) -> Self {
		Cursor {
			position: (0, 0),
			limits:   (width, height),
			style:    Default::default(),
		}
	}

	pub fn position(&self) -> (u32, u32) {
		self.position
	}

	pub fn x(&self) -> u32 {
		self.position.0
	}

	pub fn y(&self) -> u32 {
		self.position.1
	}

	pub fn style(&self) -> &Rc<Style> {
		&self.style
	}

	pub fn update(&mut self, style: Style) {
		if &*self.style != &style {
			self.style = Rc::new(style);
		}
	}

	pub fn travel(&mut self, value: Travel, dirty: &mut Dirty) {
		dirty.push(self.position);

		match value {
			Position(x, y) => {
				if let Some(x) = x {
					if x <= self.limits.0 {
						self.position.0 = x;
					}
				}

				if let Some(y) = y {
					if y <= self.limits.1 {
						self.position.1 = y;
					}
				}
			}

			Up(n) => {
				if self.position.1 > 0 {
					self.position.1 -= n;
				}
			}

			Down(n) => {
				if self.position.1 < self.limits.1 {
					self.position.1 += n;
				}
			}

			Left(n) => {
				if self.position.0 > 0 {
					self.position.0 -= n;
				}
				else if self.position.1 > 0 {
					self.position.0  = self.limits.0 - 1;
					self.position.1 -= 1;
				}
			}

			Right(n) => {
				if self.position.0 < self.limits.0 {
					self.position.0 += n;
				}
				else if self.position.1 < self.limits.1 {
					self.position.0  = 0;
					self.position.1 += 1;
				}
			}
		}

		dirty.push(self.position);
	}
}

/// A wrapper for a cursor and the cell it's on.
pub struct CursorCell<'a> {
	cursor: &'a Cursor,
	cell:   &'a Cell,
}

impl<'a> CursorCell<'a> {
	pub fn new(cursor: &'a Cursor, cell: &'a Cell) -> CursorCell<'a> {
		CursorCell {
			cursor: cursor,
			cell:   cell,
		}
	}

	pub fn cell(&self) -> &Cell {
		self.cell
	}
}

impl<'a> Deref for CursorCell<'a> {
	type Target = Cursor;

	fn deref(&self) -> &Self::Target {
		self.cursor
	}
}
