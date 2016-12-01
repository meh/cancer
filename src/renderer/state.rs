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

use std::sync::Arc;

use util::{clamp, Region};
use config::Config;
use font::Font;

#[derive(Clone, Debug)]
pub struct State {
	pub(super) config: Arc<Config>,
	pub(super) font:   Arc<Font>,

	pub(super) width:  u32,
	pub(super) height: u32,

	pub(super) spacing: u32,
	pub(super) margin:  Margin,
}

/// Adaptable margins depending on the view size.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Margin {
	pub horizontal: u32,
	pub vertical:   u32,
}

impl State {
	/// The terminal configuration.
	pub fn config(&self) -> &Arc<Config> {
		&self.config
	}

	/// The font being used.
	pub fn font(&self) -> &Arc<Font> {
		&self.font
	}

	/// The view width.
	pub fn width(&self) -> u32 {
		self.width
	}

	/// The view height.
	pub fn height(&self) -> u32 {
		self.height
	}

	/// How many rows fit the view.
	pub fn rows(&self) -> u32 {
		(self.height - (self.margin.vertical * 2)) /
			(self.font.height() + self.spacing)
	}

	/// How many columns fit the view.
	pub fn columns(&self) -> u32 {
		(self.width - (self.margin.horizontal * 2)) /
			self.font.width()
	}

	/// The current spacing.
	pub fn spacing(&self) -> u32 {
		self.spacing
	}

	/// The current margins.
	pub fn margin(&self) -> &Margin {
		&self.margin
	}

	/// Resize the state.
	pub fn resize(&mut self, width: u32, height: u32) {
		let (m, s) = (self.config.style().margin(), self.spacing);

		self.margin.horizontal = m +
			((width - (m * 2)) % self.font.width()) / 2;

		self.margin.vertical = m +
			((height - (m * 2)) % (self.font.height() + s)) / 2;

		self.width  = width;
		self.height = height;
	}

	/// Find the cell position from the real position.
	pub fn position(&self, x: u32, y: u32) -> Option<(u32, u32)> {
		let (f, h, v, s) = (&self.font, self.margin.horizontal, self.margin.vertical, self.spacing);

		// Check if the region falls exactly within a margin, if so bail out.
		if h != 0 && v != 0 &&
		   (x < h || x >= self.width - h ||
		    y < v || y >= self.height - v)
		{
			return None;
		}

		// Cache font dimensions.
		let width  = f.width() as f32;
		let height = (f.height() + s) as f32;

		// Remove the margin from coordinates.
		let x = x.saturating_sub(h) as f32;
		let y = y.saturating_sub(v) as f32;

		let x = (x / width).floor() as u32;
		let y = (y / height).floor() as u32;

		Some((x, y))
	}

	/// Turn the damaged region to cell-space.
	pub fn damaged(&self, region: &Region) -> Region {
		let (f, h, v, s) = (&self.font, self.margin.horizontal, self.margin.vertical, self.spacing);

		// Check if the region falls exactly within a margin, if so bail out.
		if h != 0 && v != 0 &&
		   ((region.x < h && region.width <= h - region.x) ||
		    (region.x >= self.width - h) ||
		    (region.y < v && region.height <= v - region.y) ||
		    (region.y >= self.height - v))
		{
			return Region::new(0, 0, 0, 0);
		}

		// Cache font dimensions.
		let width  = f.width() as f32;
		let height = (f.height() + s) as f32;

		// Remove the margin from coordinates.
		let x = region.x.saturating_sub(h) as f32;
		let y = region.y.saturating_sub(v) as f32;

		// Remove margins from width.
		let w = region.width
			.saturating_sub(h.saturating_sub(region.x))
			.saturating_sub(h.saturating_sub(self.width - (region.x + region.width))) as f32;

		// Remove margins from height.
		let h = region.height
			.saturating_sub(v.saturating_sub(region.y))
			.saturating_sub(v.saturating_sub(self.height - (region.y + region.height))) as f32;

		let x = (x / width).floor() as u32;
		let y = (y / height).floor() as u32;
		let w = clamp((w / width).ceil() as u32, 0, self.columns());
		let h = clamp((h / height).ceil() as u32, 0, self.rows());

		// Increment width and height by one if it fits within dimensions.
		//
		// This is done because the dirty region is actually bigger than the one
		// reported, or because the algorithm is broken. Regardless, this way it
		// works properly.
		Region::new(x, y,
			w + if x + w < self.columns() { 1 } else { 0 },
			h + if y + h < self.rows() { 1 } else { 0 })
	}
}
