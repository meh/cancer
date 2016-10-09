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

use std::collections::HashSet;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

use picto;
use picto::iter::Coordinates;
use terminal::{Terminal, Cell, cell};

#[derive(Debug)]
pub struct Area<'a> {
	iter:  Coordinates,
	inner: &'a Terminal,
	seen:  HashSet<(u32, u32), BuildHasherDefault<FnvHasher>>,
}

impl<'a> Area<'a> {
	pub fn new(inner: &Terminal, area: picto::Area) -> Area {
		Area {
			iter:  area.relative(),
			inner: inner,
			seen:  Default::default(),
		}
	}
}

impl<'a> Iterator for Area<'a> {
	type Item = &'a Cell;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let (x, y) = if let Some((x, y)) = self.iter.next() {
				(x, y)
			}
			else {
				return None;
			};

			let cell = self.inner.get(x, y);

			if let &cell::State::Reference { x, y, .. } = cell.state() {
				let cell = self.inner.get(x, y);

				if !self.seen.contains(&(x, y)) {
					self.seen.insert((x, y));
					return Some(cell);
				}
			}
			else if cell.is_wide() {
				self.seen.insert((x, y));
				return Some(cell);
			}
			else {
				return Some(cell);
			}
		}
	}
}
