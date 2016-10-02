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

use std::vec;
use terminal::{Terminal, Cell};

#[derive(Debug)]
pub struct Filter<'a> {
	iter:  vec::IntoIter<(u32, u32)>,
	inner: &'a Terminal,
}

impl<'a> Filter<'a> {
	pub fn new(inner: &Terminal, list: Vec<(u32, u32)>) -> Filter {
		Filter {
			iter:  list.into_iter(),
			inner: inner,
		}
	}
}

impl<'a> Iterator for Filter<'a> {
	type Item = &'a Cell;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some((x, y)) = self.iter.next() {
			Some(match self.inner.get(x, y) {
				&Cell::Reference { x, y } =>
					self.inner.get(x, y),

				cell =>
					cell
			})
		}
		else {
			None
		}
	}
}
