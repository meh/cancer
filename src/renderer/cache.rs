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
use std::sync::Arc;

use sys::pango;
use style::{self, Style};
use terminal::cell;
use font::Font;
use renderer::{option, Options};

#[derive(Debug)]
pub struct Cache {
	width:  u32,
	height: u32,
	inner:  Vec<Cell>,
}

#[derive(Clone, Default, Debug)]
pub struct Cell {
	style:   Rc<Style>,
	value:   Option<String>,
	valid:   bool,
	options: Options,
}

impl Cell {
	/// Create an empty cache cell.
	pub fn empty(style: Rc<Style>) -> Self {
		Cell {
			style:   style,
			value:   None,
			valid:   true,
			options: Options::empty(),
		}
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

		let style  = Rc::new(Style::default());
		self.inner = vec![Cell::empty(style.clone()); (width * height) as usize];
	}

	/// Invalidate the given cell.
	pub fn invalidate(&mut self, cell: &cell::Position) {
		debug_assert!(!cell.is_reference());

		self.inner[(cell.y() * self.width + cell.x()) as usize].valid = false;
	}

	/// Update the cache, returns `false` if nothing was changed.
	pub fn update(&mut self, cell: &cell::Position, mut options: Options) -> bool {
		debug_assert!(!cell.is_reference());

		let cache = &mut self.inner[(cell.y() * self.width + cell.x()) as usize];

		// Avoid invalidating the cache because of blinking when the cell is not
		// blinking.
		if !cache.style.attributes().contains(style::BLINK) &&
		   !cell.style().attributes().contains(style::BLINK)
		{
			options.remove(option::BLINKING);
		}

		// Check if the cache is up to date.
		if cache.valid &&
		   cache.options == options &&
		   cell.style() == &cache.style &&
		   ((cell.is_empty() && cache.value.is_none()) ||
		    (cell.is_occupied() && cache.value.as_ref().map(AsRef::as_ref) == Some(cell.value())))
		{
			return false;
		}

		*cache = Cell {
			style:   cell.style().clone(),
			value:   if cell.is_empty() { None } else { Some(cell.value().into()) },
			options: options,
			valid:   true,
		};

		true
	}
}
