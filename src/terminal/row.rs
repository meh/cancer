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
use std::ops::{Deref, DerefMut};

use terminal::Cell;

/// A row within the view or scroll back.
#[derive(PartialEq, Clone, Debug)]
pub struct Row {
	pub(super) inner: VecDeque<Cell>,
	pub(super) wrap:  bool,
}

impl Row {
	/// Check if the `Row` is wrapping.
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
