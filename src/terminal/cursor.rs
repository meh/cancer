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
use std::rc::Rc;
use std::sync::Arc;

use picto::color::Rgba;
use style::Style;
use terminal::{cell, Touched};
use config::Config;
use config::style::Shape;

#[derive(PartialEq, Clone, Default, Debug)]
pub struct Cursor {
	position: (u32, u32),
	limits:   (u32, u32),
	style:    Rc<Style>,

	pub(super) foreground: Rgba<f64>,
	pub(super) background: Rgba<f64>,
	pub(super) shape:      Shape,
	pub(super) state:      State,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Travel {
	Position(Option<u32>, Option<u32>),

	Up(u32),
	Down(u32),
	Left(u32),
	Right(u32),
}

bitflags! {
	pub flags State: u8 {
		const BLINK   = 1 << 0,
		const VISIBLE = 1 << 1,
		const WRAP    = 1 << 2,
	}
}

impl Default for State {
	fn default() -> Self {
		VISIBLE
	}
}

pub use self::Travel::*;

impl Cursor {
	pub fn new(config: Arc<Config>, width: u32, height: u32) -> Self {
		let mut state = State::default();
		if config.style().cursor().blink() {
			state.insert(BLINK);
		}

		Cursor {
			position: (0, 0),
			limits:   (width, height),
			style:    Default::default(),

			foreground: *config.style().cursor().foreground(),
			background: *config.style().cursor().background(),
			shape:      config.style().cursor().shape(),
			state:      state,
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

	pub fn foreground(&self) -> &Rgba<f64> {
		&self.foreground
	}

	pub fn background(&self) -> &Rgba<f64> {
		&self.background
	}

	pub fn shape(&self) -> Shape {
		self.shape
	}

	pub fn blink(&self) -> bool {
		self.state.contains(BLINK)
	}

	pub fn is_visible(&self) -> bool {
		self.state.contains(VISIBLE)
	}

	pub fn wrap(&self) -> bool {
		self.state.contains(WRAP)
	}

	pub fn update(&mut self, style: Style) {
		if &*self.style != &style {
			self.style = Rc::new(style);
		}
	}

	pub fn travel(&mut self, value: Travel, touched: &mut Touched) -> Option<i32> {
		self.state.remove(WRAP);
		touched.push(self.position);

		let mut overflow = None;

		match value {
			Position(x, y) => {
				if let Some(x) = x {
					if x < self.limits.0 {
						self.position.0 = x;
					}
					else {
						self.position.0 = self.limits.0 - 1;
					}
				}

				if let Some(y) = y {
					if y < self.limits.1 {
						self.position.1 = y;
					}
					else {
						self.position.1 = self.limits.1 - 1;
					}
				}
			}

			Up(n) => {
				let new = self.position.1 as i32 - n as i32;

				if new < 0 {
					self.position.1 = 0;
					overflow = Some(new);
				}
				else {
					self.position.1 = new as u32;
				}
			}

			Down(n) => {
				let new = self.position.1 as i32 + n as i32;

				if new >= self.limits.1 as i32 {
					self.position.1 = self.limits.1 - 1;
					overflow = Some(new - (self.limits.1 as i32 - 1));
				}
				else {
					self.position.1 = new as u32;
				}
			}

			Left(n) => {
				let new = self.position.0 as i32 - n as i32;

				if new < 0 {
					self.position.0 = 0;
					overflow = Some(new);
				}
				else {
					self.position.0 = new as u32;
				}
			}

			Right(n) => {
				let new = self.position.0 as i32 + n as i32;

				if new >= self.limits.0 as i32 {
					self.position.0 = self.limits.0 - 1;
					overflow = Some(new - (self.limits.0 as i32 - 1));
				}
				else {
					self.position.0 = new as u32;
				}
			}
		}

		touched.push(self.position);
		overflow
	}
}

/// A wrapper for a cursor and the cell it's on.
pub struct Cell<'a> {
	cursor: &'a Cursor,
	cell:   cell::Position<'a>,
}

impl<'a> Cell<'a> {
	pub fn new(cursor: &'a Cursor, cell: cell::Position<'a>) -> Cell<'a> {
		Cell {
			cursor: cursor,
			cell:   cell,
		}
	}

	pub fn cell(&self) -> cell::Position {
		self.cell
	}
}

impl<'a> Deref for Cell<'a> {
	type Target = Cursor;

	fn deref(&self) -> &Self::Target {
		self.cursor
	}
}
