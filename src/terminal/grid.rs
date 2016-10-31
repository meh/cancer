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

use std::collections::VecDeque;
use std::rc::Rc;

use util::clamp;
use style::Style;
use terminal::Cell;

pub type Row = VecDeque<Cell>;

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
	inner: VecDeque<Row>
}

impl Free {
	pub fn new() -> Self {
		Free {
			empty: Rc::new(Style::default()),
			inner: VecDeque::new(),
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
				// Reset the cells.
				for cell in &mut row {
					cell.into_empty(self.empty.clone());
				}

				// Resize the row to the wanted width.
				if row.len() > cols {
					row.drain(cols ..);
				}
				else {
					for _ in 0 .. cols - row.len() {
						row.push_back(Cell::empty(self.empty.clone()));
					}
				}

				row
			}

			None =>
				vec_deque![Cell::empty(self.empty.clone()); cols]
		}
	}

	/// Push a `Row` for reuse.
	pub fn push(&mut self, row: Row) {
		self.inner.push_back(row);
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
		for x in x .. self.cols {
			let cell = &mut self.view[y as usize][x as usize];

			if cell.is_reference() {
				cell.into_empty(self.free.style());
			}
		}
	}

	/// Resize the grid.
	pub fn resize(&mut self, cols: u32, rows: u32) {
		self.cols = cols;
		self.rows = rows;

		for row in &mut self.view {
			row.resize(cols as usize, self.free.cell());
		}

		if self.view.len() > rows as usize {
			let overflow = self.view.len() - rows as usize;
			self.back.extend(self.view.drain(.. overflow));
		}
		else {
			for _ in 0 .. rows as usize - self.view.len() {
				self.view.push_back(self.free.pop(cols as usize));
			}
		}

		self.clean_history();
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

	/// Get a cell at the given location.
	pub fn get(&self, x: u32, y: u32) -> &Cell {
		&self.view[y as usize][x as usize]
	}

	/// Get a mutable cell at the given location.
	pub fn get_mut(&mut self, x: u32, y: u32) -> &mut Cell {
		&mut self.view[y as usize][x as usize]
	}
}
