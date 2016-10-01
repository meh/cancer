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

use picto::Area;
use terminal::{Terminal, Cell};

pub struct Iter<'a> {
	x: u32,
	y: u32,

	area:  Area,
	inner: &'a Terminal,
}

impl<'a> Iter<'a> {
	pub fn new(inner: &Terminal, area: Area) -> Iter {
		Iter {
			x: 0,
			y: 0,

			area:  area,
			inner: inner,
		}
	}
}

impl<'a> Iterator for Iter<'a> {
	type Item = &'a Cell;

	fn next(&mut self) -> Option<Self::Item> {
		None
	}
}
