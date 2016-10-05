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

use picto;
use picto::iter::Coordinates;
use terminal::{Terminal, Cell, cell};

#[derive(Debug)]
pub struct Area<'a> {
	iter:  Coordinates,
	inner: &'a Terminal,
}

impl<'a> Area<'a> {
	pub fn new(inner: &Terminal, area: picto::Area) -> Area {
		Area {
			iter:  area.relative(),
			inner: inner,
		}
	}
}

impl<'a> Iterator for Area<'a> {
	type Item = &'a Cell;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some((x, y)) = self.iter.next() {
			let cell = self.inner.get(x, y);

			match cell.state() {
				&cell::State::Reference { x, y, .. } =>
					Some(self.inner.get(x, y)),

				_ =>
					Some(cell)
			}
		}
		else {
			None
		}
	}
}
