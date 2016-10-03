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
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel};

use picto::Area;
use error;
use config::Config;
use style::{self, Style};
use terminal::{Cell, iter};

use picto::color::Rgba;

#[derive(Debug)]
pub struct Terminal {
	config: Arc<Config>,
	output: Option<Receiver<Vec<u8>>>,

	area:     Area,
	cells:    Vec<Cell>,
	cursor:   (u32, u32),
	blinking: bool,
}

impl Terminal {
	pub fn open(config: Arc<Config>, width: u32, height: u32) -> error::Result<Self> {
		let (sender, receiver) = sync_channel(1);
		thread::spawn(move || {
			loop {
				sender.send(vec![]).unwrap();
				thread::sleep(Duration::from_secs(10));
			}
		});

		let area  = Area::from(0, 0, width, height);
		let style = Rc::new(Style::default());
		let cells = area.absolute().map(|(x, y)|
			Cell::Empty { x: x, y: y, style: style.clone() });

		Ok(Terminal {
			config: config,
			output: Some(receiver),

			area:     area,
			cells:    cells.collect(),
			cursor:   (2, 2),
			blinking: false,
		})
	}

	pub fn output(&mut self) -> Receiver<Vec<u8>> {
		self.output.take().unwrap()
	}

	pub fn resize(&mut self, width: u32, height: u32) {

	}

	pub fn columns(&self) -> u32 {
		self.area.width
	}

	pub fn rows(&self) -> u32 {
		self.area.height
	}

	pub fn blinking<'a>(&'a mut self, value: bool) -> impl Iterator<Item = &'a Cell> {
		self.blinking = value;
		self.iter().filter(|c| c.style().attributes().contains(style::BLINK))
	}

	pub fn is_blinking(&self) -> bool {
		self.blinking
	}

	pub fn cursor(&self) -> &Cell {
		self.get(self.cursor.0, self.cursor.1)
	}

	pub fn area(&self, area: Area) -> iter::Area {
		iter::Area::new(&self, area)
	}

	pub fn iter(&self) -> iter::Area {
		iter::Area::new(&self, self.area)
	}

	pub fn get(&self, x: u32, y: u32) -> &Cell {
		&self.cells[(y * self.area.width + x) as usize]
	}

	pub fn get_mut(&mut self, x: u32, y: u32) -> &mut Cell {
		&mut self.cells[(y * self.area.width + x) as usize]
	}
}
