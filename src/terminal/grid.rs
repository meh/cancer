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

use std::mem;
use std::ops::{Deref, DerefMut};
use std::collections::{VecDeque, LinkedList};
use std::rc::Rc;

use itertools::Itertools;
use util::clamp;
use style::Style;
use terminal::{Access, Cell};

#[derive(Debug)]
pub struct Grid {
	cols:    u32,
	rows:    u32,
	history: usize,

	free: Free,
	back: VecDeque<Row>,
	view: VecDeque<Row>,
}

#[derive(Debug)]
pub struct Free {
	empty: Rc<Style>,
	inner: LinkedList<Row>
}

impl Free {
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
				row.wrap = false;
				row.resize(cols, Cell::empty(self.empty.clone()));

				for cell in row.iter_mut().filter(|c| !c.is_default()) {
					cell.make_empty(self.empty.clone());
				}

				row
			}

			None => {
				Row {
					inner: vec_deque![Cell::empty(self.empty.clone()); cols],
					wrap:  false
				}
			}
		}
	}

	/// Push a `Row` for reuse.
	pub fn push(&mut self, row: Row) {
		self.inner.push_front(row);
	}
}

/// A row within the view or scroll back.
#[derive(PartialEq, Clone, Debug)]
pub struct Row {
	inner: VecDeque<Cell>,
	wrap:  bool,
}

impl Row {
	pub fn wrap(&self) -> bool {
		self.wrap
	}
}

impl Deref for Row {
	type Target = VecDeque<Cell>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for Row {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl Grid {
	/// Create a new grid.
	pub fn new(cols: u32, rows: u32, history: usize) -> Self {
		let mut value = Grid {
			cols:    0,
			rows:    0,
			history: history,

			free: Free::new(),
			back: VecDeque::new(),
			view: VecDeque::new(),
		};

		value.resize(cols, rows);
		value
	}

	pub fn get(&self, x: u32, y: u32) -> &Cell {
		&self.view[y as usize][x as usize]
	}

	pub fn get_mut(&mut self, x: u32, y: u32) -> &mut Cell {
		&mut self.view[y as usize][x as usize]
	}

	/// Get the scroll back.
	pub fn back(&self) -> &VecDeque<Row> {
		&self.back
	}

	/// Get the current view.
	pub fn view(&self) -> &VecDeque<Row> {
		&self.view
	}

	/// Drop rows in the scrollback that go beyond the history limit.
	pub fn clean_history(&mut self) {
		if self.back.len() > self.history {
			let overflow = self.back.len() - self.history;

			for row in self.back.drain(.. overflow) {
				self.free.push(row);
			}
		}
	}

	/// Clean left-over references from changes.
	pub fn clean_references(&mut self, x: u32, y: u32) {
		if !self.view[y as usize][x as usize].is_reference() {
			return;
		}

		for x in x .. self.cols {
			let cell = &mut self.view[y as usize][x as usize];

			if cell.is_reference() {
				cell.make_empty(self.free.style());
			}
			else {
				break;
			}
		}
	}

	/// Resize the grid.
	pub fn resize(&mut self, cols: u32, rows: u32) -> i32 {
		// Resize the view taking into consideration lines that were wrapped.
		fn resize(view: &mut VecDeque<Row>, free: &mut Free, cols: u32) -> i32 {
			let mut offset  = 0;
			let mut wrapped = Vec::new();

			for i in (0 .. view.len()).rev() {
				if view[i].wrap() {
					wrapped.push(view.remove(i).unwrap());
				}
				else if !wrapped.is_empty() {
					wrapped.push(view.remove(i).unwrap());

					let mut unwrapped = Vec::new();
					let     before    = wrapped.len();

					// Remove any empty leftover before trying to unwrap the row.
					{
						let mut row = &mut wrapped[0];

						while row.len() > 0 && row.back().unwrap().is_empty() {
							row.pop_back();
						}
					}

					// Split the cells into appropriately sized chunks, since we pushed
					// the rows in reverse order we reverse the iterator.
					let chunks = mem::replace(&mut wrapped, Vec::new()).into_iter().rev().flat_map(|v| v.inner.into_iter()).chunks(cols as usize);

					// Create new rows with the cells and mark as wrapped if they do wrap
					// again.
					for (j, cells) in chunks.into_iter().enumerate() {
						unwrapped.push(Row { inner: cells.collect(), wrap: j != 0 });
					}

					// Extend any missing cells from the last row.
					{
						let mut row    = unwrapped.last_mut().unwrap();
						let     length = row.len();

						if length < cols as usize {
							for _ in 0 .. cols as usize - length {
								row.push_back(free.cell());
							}
						}
					}

					// Update the offset for the cursor.
					offset += unwrapped.len() as i32 - before as i32;

					// Reinsert the rows in the view.
					for row in unwrapped.into_iter().rev() {
						view.insert(i, row);
					}
				}
				else if view[i].len() > cols as usize {
					let mut row = view.remove(i).unwrap();

					// Remove any empty leftover before trying to split the row.
					while row.len() > cols as usize && row.back().unwrap().is_empty() {
						row.pop_back();
					}

					if row.len() != cols as usize {
						let mut wrapped = Vec::new();
						let     chunks  = row.inner.into_iter().chunks(cols as usize);

						// Create new rows with the cells and mark as wrapped if they do
						// wrap.
						for (j, cells) in chunks.into_iter().enumerate() {
							wrapped.push(Row { inner: cells.collect(), wrap: j != 0 });
						}

						// Extend any missing cells from the last row.
						{
							let mut row    = wrapped.last_mut().unwrap();
							let     length = row.len();

							if length < cols as usize {
								for _ in 0 .. cols as usize - length {
									row.push_back(free.cell());
								}
							}
						}

						// Update the offset for the cursor.
						offset += wrapped.len() as i32 - 1;

						// Reinsert the rows in the view.
						for row in wrapped.into_iter().rev() {
							view.insert(i, row);
						}
					}
					else {
						view.insert(i, row);
					}
				}
				else {
					view[i].resize(cols as usize, free.cell());
				}
			}

			offset
		}

		self.cols = cols;
		self.rows = rows;

		let mut offset = resize(&mut self.view, &mut self.free, cols);
		resize(&mut self.back, &mut self.free, cols);

		if self.view.len() > rows as usize {
			while self.view.len() > rows as usize && self.view.back().unwrap().iter().all(|v| v.is_empty()) {
				self.view.pop_back();
			}

			let overflow = self.view.len() - rows as usize;
			self.back.extend(self.view.drain(.. overflow));
		}

		if self.view.len() < rows as usize {
			let overflow = rows as usize - self.view.len();

			for _ in 0 .. overflow {
				if let Some(row) = self.back.pop_back() {
					offset += 1;
					self.view.push_front(row);
				}
				else {
					self.view.push_back(self.free.pop(cols as usize));
				}
			}
		}

		self.clean_history();
		offset
	}

	/// Move the view `n` cells to the left, dropping cells from the back.
	pub fn left(&mut self, n: u32) {
		for _ in 0 .. n {
			for row in &mut self.view {
				while row.pop_back().unwrap().is_reference() {
					row.push_front(self.free.cell());
				}

				row.push_front(self.free.cell());
			}
		}
	}

	/// Move the view `n` cells to the right, dropping cells from the front.
	pub fn right(&mut self, n: u32) {
		for _ in 0 .. n {
			for row in &mut self.view {
				row.pop_front();
				row.push_back(self.free.cell());

				while row.front().unwrap().is_reference() {
					row.pop_front();
					row.push_back(self.free.cell());
				}
			}
		}
	}

	/// Scroll the view up by `n`, optionally within the given region.
	pub fn up(&mut self, n: u32, region: Option<(u32, u32)>) {
		if let Some(region) = region {
			let y      = region.0;
			let n      = clamp(n as u32, 0, region.1 - y + 1);
			let offset = self.rows - (region.1 + 1);

			// Remove the lines.
			for row in self.view.drain(y as usize .. (y + n) as usize) {
				self.free.push(row);
			}

			// Fill missing lines.
			let index = self.view.len() - offset as usize;
			for i in 0 .. n {
				self.view.insert(index + i as usize, self.free.pop(self.cols as usize));
			}
		}
		else {
			self.view.push_back(self.free.pop(self.cols as usize));
			self.back.push_back(self.view.pop_front().unwrap());
		}

		self.clean_history();
	}

	/// Scroll the view down by `n`, optionally within the region.
	pub fn down(&mut self, n: u32, region: Option<(u32, u32)>) {
		if let Some(region) = region {
			let y = region.0;
			let n = clamp(n as u32, 0, (region.1 - y + 1));

			// Split the cells at the current line.
			let mut rest = self.view.split_off(y as usize);

			// Extend with new lines.
			for _ in 0 .. n {
				self.view.push_back(self.free.pop(self.cols as usize));
			}

			// Remove the scrolled off lines.
			let offset = region.1 + 1 - y - n;
			for row in rest.drain(offset as usize .. (offset + n) as usize) {
				self.free.push(row);
			}
			self.view.append(&mut rest);
		}

		self.clean_history();
	}

	/// Delete `n` cells starting from the given origin.
	pub fn delete(&mut self, x: u32, y: u32, n: u32) {
		let n   = clamp(n, 0, self.cols - x);
		let row = &mut self.view[y as usize];

		row.drain(x as usize .. x as usize + n as usize);
		row.append(&mut vec_deque![self.free.cell(); n as usize]);
	}

	/// Insert `n` empty cells starting from the given origin.
	pub fn insert(&mut self, x: u32, y: u32, n: u32) {
		let n   = clamp(n as u32, 0, self.cols);
		let row = &mut self.view[y as usize];

		for _ in x .. x + n {
			row.insert(x as usize, self.free.cell());
		}

		row.drain(self.cols as usize ..);
	}

	/// Mark a row as wrapped.
	pub fn wrap(&mut self, y: u32) {
		self.view[y as usize].wrap = true;
	}
}

impl Access for Grid {
	fn access(&self, x: u32, y: u32) -> &Cell {
		self.get(x, y)
	}
}
