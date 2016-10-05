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
use std::io::Write;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use picto::Area;
use error::{self, Error};
use config::Config;
use style::{self, Style};
use terminal::{Cell, cell, iter};

#[derive(Debug)]
pub struct Terminal {
	config: Arc<Config>,

	area:     Area,
	cells:    Vec<Cell>,
	cursor:   (u32, u32),
	blinking: bool,
}

impl Terminal {
	pub fn open(config: Arc<Config>, width: u32, height: u32) -> error::Result<Self> {
		let area  = Area::from(0, 0, width, height);
		let style = Rc::new(Style::default());
		let cells = area.absolute().map(|(x, y)| Cell::new(x, y, style.clone()));

		Ok(Terminal {
			config: config,

			area:     area,
			cells:    cells.collect(),
			cursor:   (0, 0),
			blinking: false,
		})
	}

	pub fn columns(&self) -> u32 {
		self.area.width
	}

	pub fn rows(&self) -> u32 {
		self.area.height
	}

	pub fn is_blinking(&self) -> bool {
		self.blinking
	}

	pub fn cursor(&self) -> &Cell {
		self.get(self.cursor.0, self.cursor.1)
	}

	pub fn get(&self, x: u32, y: u32) -> &Cell {
		&self.cells[(y * self.area.width + x) as usize]
	}

	pub fn area(&self, area: Area) -> iter::Area {
		iter::Area::new(&self, area)
	}

	pub fn iter(&self) -> iter::Area {
		iter::Area::new(&self, self.area)
	}

	pub fn resize<'a>(&'a mut self, width: u32, height: u32) -> impl Iterator<Item = &'a Cell> {
		self.iter()
	}

	pub fn blinking<'a>(&'a mut self, value: bool) -> impl Iterator<Item = &'a Cell> {
		self.blinking = value;
		self.iter().filter(|c| c.style().attributes().contains(style::BLINK))
	}

	pub fn handle<'a, I: AsRef<[u8]>, O: Write>(&'a mut self, input: I, output: O) -> error::Result<impl Iterator<Item = &'a Cell>> {
		Ok(self.iter())
	}

	// TODO(meh): handle wrapping.
	// TODO(meh): collapse references
	fn insert(&mut self, ch: String) -> u32 {
		let width = ch.width() as u32;
		let index = self.cursor.1 * self.area.width + self.cursor.0;
		let cells = &mut self.cells[index as usize .. (index + width) as usize];

		let (cell, rest) = cells.split_at_mut(1);
		cell[0].make_char(ch, false);

		for c in rest {
			c.make_reference(cell[0].x(), cell[0].y());
		}

		self.cursor.0 += width;
		width
	}

	fn delete(&mut self) -> u32 {
		0
	}
}
