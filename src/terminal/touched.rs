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
use std::collections::{hash_set, HashSet};
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;
use picto::Region;
use picto::iter::Coordinates;

#[derive(Eq, PartialEq, Clone, Default, Debug)]
pub struct Touched {
	all:      bool,
	line:     HashSet<u32, BuildHasherDefault<FnvHasher>>,
	position: HashSet<(u32, u32), BuildHasherDefault<FnvHasher>>,
}

impl Touched {
	pub fn all(&mut self) -> &mut Self {
		self.all = true;
		self
	}

	pub fn line(&mut self, line: u32) -> &mut Self {
		if !self.all {
			self.line.insert(line);
		}

		self
	}

	pub fn mark(&mut self, x: u32, y: u32) -> &mut Self {
		if !self.all {
			self.position.insert((x, y));
		}

		self
	}

	pub fn push(&mut self, pair: (u32, u32)) -> &mut Self {
		if !self.all {
			self.position.insert(pair);
		}

		self
	}

	pub fn iter(&mut self, region: Region) -> Iter {
		Iter::new(region,
			mem::replace(&mut self.all, false),
			mem::replace(&mut self.line, Default::default()),
			mem::replace(&mut self.position, Default::default()))
	}
}

pub struct Iter {
	region: Region,
	state:  State,
	lines:  HashSet<u32, BuildHasherDefault<FnvHasher>>,

	all:      bool,
	line:     Option<HashSet<u32, BuildHasherDefault<FnvHasher>>>,
	position: Option<HashSet<(u32, u32), BuildHasherDefault<FnvHasher>>>,
}

enum State {
	None,
	Done,

	All(Coordinates),
	Lines(Option<(u32, u32)>, hash_set::IntoIter<u32>),
	Positions(hash_set::IntoIter<(u32, u32)>),
}

impl Iter {
	pub fn empty() -> Self {
		Iter::new(Region::from(0, 0, 0, 0), false, Default::default(), Default::default())
	}

	pub fn new(region:   Region,
	           all:      bool,
	           line:     HashSet<u32, BuildHasherDefault<FnvHasher>>,
	           position: HashSet<(u32, u32), BuildHasherDefault<FnvHasher>>)
	-> Self {
		let all = all || line.len() == region.height as usize || position.len() == (region.width * region.height) as usize;

		Iter {
			region: region,
			state:  State::None,
			lines:  line.clone(),

			all:      all,
			line:     if all || line.is_empty() { None } else { Some(line) },
			position: if all | position.is_empty() { None } else { Some(position) },
		}
	}

	pub fn is_empty(&self) -> bool {
		!self.all && self.line.is_none() && self.position.is_none()
	}

	pub fn all(&self) -> bool {
		self.all
	}
}

impl Iterator for Iter {
	type Item = (u32, u32);

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match mem::replace(&mut self.state, State::None) {
				State::Done => {
					self.state = State::Done;

					return None;
				}

				State::None => {
					self.state = if self.all {
						State::All(self.region.absolute())
					}
					else if let Some(line) = self.line.take() {
						State::Lines(None, line.into_iter())
					}
					else if let Some(position) = self.position.take() {
						State::Positions(position.into_iter())
					}
					else {
						State::Done
					};
				}

				State::All(mut iter) => {
					if let Some(value) = iter.next() {
						self.state = State::All(iter);

						return Some(value);
					}
					else {
						self.state = State::Done;
					}
				}

				State::Lines(mut cur, mut iter) => {
					if let Some((x, y)) = cur.take() {
						if x < self.region.width {
							self.state = State::Lines(Some((x + 1, y)), iter);

							return Some((x, y));
						}
						else {
							self.state = State::Lines(None, iter);
						}
					}
					else if let Some(y) = iter.next() {
						self.state = State::Lines(Some((0, y)), iter);
					}
					else if let Some(position) = self.position.take() {
						self.state = State::Positions(position.into_iter());
					}
					else {
						self.state = State::Done;
					}
				}

				State::Positions(mut iter) => {
					if let Some((x, y)) = iter.next() {
						self.state = State::Positions(iter);

						if !self.lines.contains(&y) {
							return Some((x, y));
						}
					}
					else {
						self.state = State::Done;
					}
				}
			}
		}
	}
}
