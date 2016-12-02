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
use std::collections::{VecDeque, LinkedList};

use terminal::{Cell, Row};
use style::Style;

/// Wrapper for `Row` reuse.
#[derive(Debug)]
pub struct Free {
	empty: Rc<Style>,
	inner: LinkedList<Row>,
}

impl Free {
	/// Create a new free list.
	pub fn new() -> Self {
		Free {
			empty: Rc::new(Style::default()),
			inner: LinkedList::new(),
		}
	}

	/// Get the empty `Style`.
	pub fn style(&self) -> Rc<Style> {
		self.empty.clone()
	}

	/// Create an empty `Cell`.
	pub fn cell(&self) -> Cell {
		Cell::empty(self.empty.clone())
	}

	/// Reuse or create a new `Row`.
	pub fn pop(&mut self, cols: usize) -> Row {
		match self.inner.pop_front() {
			Some(mut row) => {
				row.wrapped = false;
				row.resize(cols, Cell::empty(self.empty.clone()));

				for cell in row.iter_mut().filter(|c| !c.is_default()) {
					cell.make_empty(self.empty.clone());
				}

				row
			}

			None => {
				Row {
					inner:   vec_deque![Cell::empty(self.empty.clone()); cols],
					wrapped: false,
				}
			}
		}
	}

	/// Push a `Row` for reuse.
	pub fn push(&mut self, row: Row) {
		self.inner.push_front(row);
	}
}
