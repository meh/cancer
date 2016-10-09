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
use terminal::{Terminal, Cell, cell};

#[derive(Debug)]
pub struct Indexed<'a, T>
	where T: Iterator<Item = (u32, u32)>
{
	iter:  T,
	inner: &'a Terminal,
}

impl<'a, T> Indexed<'a, T>
	where T: Iterator<Item = (u32, u32)>
{
	pub fn new(inner: &Terminal, iter: T) -> Indexed<T> {
		Indexed {
			iter:  iter,
			inner: inner,
		}
	}
}

impl<'a, T> Iterator for Indexed<'a, T>
	where T: Iterator<Item = (u32, u32)>
{
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
