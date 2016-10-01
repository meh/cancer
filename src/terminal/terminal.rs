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
use font::Font;
use style::Style;
use terminal::{Cell, Iter};

pub struct Terminal {
	config: Arc<Config>,

	area:  Area,
	lines: Vec<Vec<Cell>>,
	input: Option<Receiver<Vec<u8>>>,
}

impl Terminal {
	pub fn open(config: Arc<Config>, width: u32, height: u32) -> error::Result<Self> {
		let style = Rc::new(Style::default());
		let lines = (0 .. height).map(|y| vec![Cell::new(0, y, " ".into(), style.clone())]).collect();

		let (sender, receiver) = sync_channel(1);
		thread::spawn(move || {
			loop {
				sender.send(vec![]).unwrap();
				thread::sleep(Duration::from_secs(10));
			}
		});

		Ok(Terminal {
			config: config,

			area:  Area::from(0, 0, width, height),
			lines: lines,
			input: Some(receiver),
		})
	}

	pub fn input(&mut self) -> Receiver<Vec<u8>> {
		self.input.take().unwrap()
	}

	pub fn area(&self, area: Area) -> Iter {
		Iter::new(&self, area)
	}
}
