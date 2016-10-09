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
use std::collections::HashSet;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

use terminal::iter;

#[derive(Eq, PartialEq, Clone, Default, Debug)]
pub struct Dirty {
	marked: HashSet<(u32, u32), BuildHasherDefault<FnvHasher>>,
}

impl Dirty {
	pub fn mark(&mut self, x: u32, y: u32) -> &mut Self {
		self.marked.insert((x, y));
		self
	}

	pub fn push(&mut self, pair: (u32, u32)) -> &mut Self {
		self.marked.insert(pair);
		self
	}

	pub fn take(&mut self) -> HashSet<(u32, u32), BuildHasherDefault<FnvHasher>> {
		mem::replace(&mut self.marked, Default::default())
	}
}
