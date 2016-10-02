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
use style::Style;
use terminal::{Cell, Iter};

#[derive(Debug)]
pub struct Terminal {
	config: Arc<Config>,
	output: Option<Receiver<Vec<u8>>>,

	area:   Area,
	cells:  Vec<Cell>,
	cursor: (u32, u32),
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

			area:   area,
			cells:  cells.collect(),
			cursor: (0, 0),
		})
	}

	pub fn output(&mut self) -> Receiver<Vec<u8>> {
		self.output.take().unwrap()
	}

	pub fn area(&self, area: Area) -> Iter {
		Iter::new(&self, area)
	}

	pub fn get(&self, x: u32, y: u32) -> &Cell {
		&self.cells[(y * self.area.width + x) as usize]
	}
}
