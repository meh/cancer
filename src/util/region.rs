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

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Region {
	pub x: u32,
	pub y: u32,

	pub width:  u32,
	pub height: u32,
}

impl Region {
	pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
		Region {
			x: x,
			y: y,

			width:  width,
			height: height,
		}
	}

	/// Get an iterator over relative coordinates, based on the coordinates
	/// within the parent view.
	pub fn relative(&self) -> Coordinates {
		Coordinates::new(*self)
	}

	/// Get an iterator over absolute coordinates.
	pub fn absolute(&self) -> Coordinates {
		Coordinates::new(Region {
			x: 0,
			y: 0,

			width:  self.width,
			height: self.height,
		})
	}
}

/// Iterator over X and Y coordinates within an `Region`.
#[derive(Eq, PartialEq, Debug)]
pub struct Coordinates {
	x: u32,
	y: u32,

	region: Region,
}

impl Coordinates {
	/// Create a new `Iterator` for the given `Region`.
	pub fn new(region: Region) -> Self {
		Coordinates {
			x: 0,
			y: 0,

			region: region,
		}
	}

	/// The `Region` being iterated over.
	pub fn region(&self) -> Region {
		self.region
	}
}

impl Iterator for Coordinates {
	type Item = (u32, u32);

	fn next(&mut self) -> Option<Self::Item> {
		if self.x >= self.region.width {
			self.x  = 0;
			self.y += 1;
		}

		if self.y >= self.region.height {
			return None;
		}

		self.x += 1;

		Some((self.x - 1 + self.region.x, self.y + self.region.y))
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.len(), Some(self.len()))
	}
}

impl ExactSizeIterator for Coordinates {
	fn len(&self) -> usize {
		let length    = self.region.width * self.region.height;
		let remaining = length - (self.y * self.region.width + self.x);

		remaining as usize
	}
}
