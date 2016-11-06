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

use terminal::{Access, cell};

pub struct Iter<'a, A, I>
	where A: Access + 'a,
	      I: Iterator<Item = (u32, u32)>
{
	iter:  I,
	inner: &'a A,
	seen:  HashSet<(u32, u32), BuildHasherDefault<FnvHasher>>,
}

impl<'a, A, I> Iter<'a, A, I>
	where A: Access + 'a,
	      I: Iterator<Item = (u32, u32)>
{
	pub fn new(inner: &A, iter: I) -> Iter<A, I> {
		Iter {
			iter:  iter,
			inner: inner,
			seen:  Default::default(),
		}
	}
}

impl<'a, A, I> Iterator for Iter<'a, A, I>
	where A: Access + 'a,
	      I: Iterator<Item = (u32, u32)>
{
	type Item = cell::Position<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let (x, y) = if let Some((x, y)) = self.iter.next() {
				(x, y)
			}
			else {
				return None;
			};

			let mut cell = cell::Position::new(x, y, self.inner.access(x, y));

			if cell.is_reference() {
				let offset = cell.offset();
				let x      = x - offset;

				if self.seen.contains(&(x, y)) {
					continue;
				}

				cell = cell::Position::new(x, y, self.inner.access(x, y));
			}
			else if cell.is_wide() {
				self.seen.insert((x, y));
			}

			return Some(cell)
		}
	}
}
