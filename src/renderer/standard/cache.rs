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

use style::{self, Style};
use terminal::cell;
use renderer::Options;

/// Cache for cells to avoid rendering a cell multiple times when it's not
/// needed.
#[derive(Debug)]
pub struct Cache {
	width:  u32,
	height: u32,
	inner:  Vec<Cell>,
}

/// A cell in the cache.
#[derive(Clone, Default, Debug)]
pub struct Cell {
	style: Rc<Style>,
	value: Option<String>,
	flags: Flags,
}

impl Cell {
	/// Create an empty cache cell.
	pub fn empty(style: Rc<Style>) -> Self {
		Cell {
			style: style,
			value: None,
			flags: Flags::empty(),
		}
	}
}

bitflags! {
	flags Flags: u8 {
		const NONE     = 0,
		const VALID    = 1 << 0,
		const BLINKING = 1 << 1,
		const REVERSE  = 1 << 2,
	}
}

impl Default for Flags {
	fn default() -> Self {
		VALID
	}
}

impl Cache {
	/// Create a new cache of the given size.
	pub fn new(width: u32, height: u32) -> Self {
		let style = Rc::new(Style::default());

		Cache {
			width:  width,
			height: height,
			inner:  vec![Cell::empty(style.clone()); (width * height) as usize],
		}
	}

	/// Resize the cache, and invalidate it.
	pub fn resize(&mut self, width: u32, height: u32) {
		self.width  = width;
		self.height = height;

		let style = Rc::new(Style::default());
		self.inner.resize((width * height) as usize, Cell::empty(style.clone()));

		for cell in &mut self.inner {
			cell.flags.remove(VALID);
		}
	}

	/// Invalidate the given cell.
	pub fn invalidate(&mut self, cell: &cell::Position) {
		debug_assert!(!cell.is_reference());

		self.inner[(cell.y() * self.width + cell.x()) as usize].flags.remove(VALID);
	}

	/// Update the cache, returns `false` if the cache is valid.
	///
	/// The cell is seen as unchanged if it's valid, the style and content match
	/// and the rendering options match.
	pub fn update(&mut self, cell: &cell::Position, options: Options) -> bool {
		debug_assert!(!cell.is_reference());

		let index = (cell.y() * self.width + cell.x()) as usize;

		// Check if the cache is up to date, this is awful logic but alas.
		{
			let cache = &self.inner[index];

			if cache.flags.contains(VALID) &&
			   cache.flags.contains(REVERSE) == options.reverse() &&
			   (!cache.style.attributes().contains(style::BLINK) ||
				   cache.flags.contains(BLINKING) == options.blinking()) &&
			   cell.style() == &cache.style &&
			   ((cell.is_empty() && cache.value.is_none()) ||
			    (cell.is_occupied() && cache.value.as_ref().map(AsRef::as_ref) == Some(cell.value())))
			{
				return false;
			}
		}

		// Update the cache.
		self.inner[index] = Cell {
			style: cell.style().clone(),
			value: if cell.is_empty() { None } else { Some(cell.value().into()) },
			flags: VALID
				| if options.blinking() { BLINKING } else { NONE }
				| if options.reverse() { REVERSE } else { NONE }
		};

		// Invalidate reference cells.
		for cell in &mut self.inner[index + 1 .. index + cell.width() as usize] {
			cell.flags.remove(VALID);
		}

		true
	}
}
