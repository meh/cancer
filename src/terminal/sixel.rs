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

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;
use std::f32;

use picto;
use picto::color::{Rgba, Hsl, RgbHue};
use control::DEC::SIXEL;

#[derive(Debug)]
pub struct Sixel {
	raster: SIXEL::Header,

	grid:   Vec<Vec<picto::buffer::Rgba>>,
	width:  u32,
	height: u32,
	x:      u32,
	y:      u32,

	colors:     HashMap<u32, Rgba, BuildHasherDefault<FnvHasher>>,
	background: Rgba,
	color:      u32,
}

impl Sixel {
	pub fn new(header: SIXEL::Header, background: &Rgba<f64>, width: u32, height: u32) -> Self {
		Sixel {
			raster: header,

			grid:   Default::default(),
			width:  width,
			height: height,
			x:      0,
			y:      0,

			colors:     Default::default(),
			background: Rgba::new(background.red as f32, background.green as f32, background.blue as f32, background.alpha as f32),
			color:      0,

		}
	}

	pub fn rows(&self) -> usize {
		self.grid.len()
	}

	pub fn into_inner(self) -> Vec<Vec<picto::buffer::Rgba>> {
		self.grid
	}

	pub fn aspect(&mut self, aspect: (u32, u32)) {
		self.raster.aspect = aspect;
	}

	pub fn enable(&mut self, id: u32) {
		self.color = id;
	}

	pub fn define(&mut self, id: u32, color: SIXEL::Color) {
		self.colors.insert(id, match color {
			SIXEL::Color::Hsl(h, s, l) =>
				Rgba::from(Hsl::new(RgbHue::from_radians(h as f32 * f32::consts::PI / 180.0),
					s as f32 / 100.0, l as f32 / 100.0)),

			SIXEL::Color::Rgb(r, g, b) =>
				Rgba::new_u8(r, g, b, 255),

			SIXEL::Color::Rgba(r, g, b, a) =>
				Rgba::new_u8(r, g, b, a)
		});
	}

	pub fn start(&mut self) {
		self.x = 0;
	}

	pub fn next(&mut self) {
		self.x  = 0;
		self.y += 6 * self.raster.aspect.0;
	}

	pub fn draw(&mut self, value: SIXEL::Map) {
		let c = self.colors.get(&self.color).unwrap_or(&self.background);
		let x = (self.x / self.width) as usize;

		for (i, y) in (self.y .. self.y + (6 * self.raster.aspect.0)).enumerate() {
			let ox = self.x % self.width;
			let oy = y as u32 % self.height;

			let y = (y / self.height) as usize;

			if y >= self.grid.len() {
				self.grid.push(Vec::new());
			}

			if x >= self.grid[y].len() {
				self.grid[y].push(
					picto::buffer::Rgba::from_pixel(self.width, self.height, &self.background));
			}

			if value.get((i as u32 / self.raster.aspect.0) as u8) {
				self.grid[y][x].set(ox, oy, c);
			}
			else if self.raster.background {
				self.grid[y][x].set(ox, oy, &self.background);
			}
		}

		self.x += 1;
	}
}
