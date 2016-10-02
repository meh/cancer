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

use std::cmp;
use std::sync::Arc;
use std::ops::{Deref, DerefMut};

use unicode_width::UnicodeWidthStr;
use picto::Area;
use config::Config;
use sys::cairo;
use sys::pango;
use font::Font;
use style::Style;
use terminal::Cell;

/// Renderer for a `cairo::Surface`.
pub struct Renderer {
	config: Arc<Config>,
	font:   Arc<Font>,
	width:  u32,
	height: u32,

	context: cairo::Context,
	layout:  pango::Layout,
}

impl Renderer {
	/// Create a new renderer for the given settings and surface.
	pub fn new<S: AsRef<cairo::Surface>>(config: Arc<Config>, font: Arc<Font>, surface: S, width: u32, height: u32) -> Self {
		let context = cairo::Context::new(surface);
		let layout  = pango::Layout::new(font.as_ref());

		Renderer {
			config: config,
			font:   font,
			width:  width,
			height: height,

			context: context,
			layout:  layout,
		}
	}

	/// How many rows fit the view.
	pub fn rows(&self) -> u32 {
		(self.height - (self.config.style().margin() * 2)) /
			(self.font.height() + self.config.style().spacing())
	}

	/// How many columns fit the view.
	pub fn columns(&self) -> u32 {
		(self.width - (self.config.style().margin() * 2)) /
			self.font.width()
	}

	/// Resize the renderer viewport.
	pub fn resize(&mut self, width: u32, height: u32) {
		self.width  = width;
		self.height = height;
	}

	/// Turn the damaged area to cell-space.
	pub fn damaged(&self, area: &Area) -> Area {
		let (c, f) = (&self.config, &self.font);
		let m      = c.style().margin();
		let s      = c.style().spacing();

		// Check if the area falls exactly within a margin, if so bail out.
		if (area.x < m && area.width <= m - area.x) ||
		   (area.x >= self.width - m) ||
		   (area.y < m && area.height <= m - area.y) ||
		   (area.y >= self.height - m)
		{
			return Area::from(0, 0, 0, 0);
		}

		// Cache font dimensions.
		let width  = f.width();
		let height = f.height() + s;

		// Cache terminal dimension.
		let columns = self.width - (m * 2) / width;
		let rows    = self.height - (m * 2) / height;

		// Remove the margin from coordinates.
		let x = area.x.saturating_sub(m);
		let y = area.y.saturating_sub(m);

		// Remove margins from width.
		let w = area.width
			.saturating_sub(m.saturating_sub(area.x))
			.saturating_sub(m.saturating_sub(self.width - (area.x + area.width)));

		// Remove margins from height.
		let h = area.height
			.saturating_sub(m.saturating_sub(area.y))
			.saturating_sub(m.saturating_sub(self.height - (area.y + area.height)));

		// Increase dimensions by one, and clamp to maximum dimensions.
		Area::from(
			cmp::min(columns,     (x as f32 / width as f32).floor() as u32),
			cmp::min(rows,        (y as f32 / height as f32).floor() as u32),
			cmp::min(columns, 1 + (w as f32 / width as f32).ceil() as u32),
			cmp::min(rows,    1 + (h as f32 / height as f32).ceil() as u32))
	}

	/// Batch the drawing operations within the closure.
	pub fn update<T, F: FnOnce(&mut Self) -> T>(&mut self, func: F) -> T {
		self.push();
		let out = func(self);
		self.pop();
		self.paint();

		out
	}

	/// Render the margins within the given area.
	pub fn margin(&mut self, area: &Area) {
		let (c, o) = (&self.config, &mut self.context);
		let m      = c.style().margin();

		if m == 0 {
			return;
		}

		o.save();
		{
			o.rgba(c.style().background());

			// Left margin.
			if area.x < m {
				o.rectangle(0.0, area.y as f64, m as f64, area.height as f64);
				o.fill(false);
			}

			// Right margin.
			if area.x + area.width >= self.width - m {
				o.rectangle((self.width - m) as f64, area.y as f64, m as f64, area.height as f64);
				o.fill(false);
			}

			// Top margin.
			if area.y < m {
				o.rectangle(area.x as f64, 0.0, area.width as f64, m as f64);
				o.fill(false);
			}

			// Bottom margin.
			if area.y + area.height >= self.height - m {
				o.rectangle(area.x as f64, (self.height - m) as f64, area.width as f64, m as f64);
				o.fill(false);
			}
		}
		o.restore();
	}

	/// Update the given cell.
	pub fn cell(&mut self, cell: &Cell) {
		debug_assert!(match cell { &Cell::Reference { .. } => false, _ => true });

		let (c, o, l, f) = (&self.config, &mut self.context, &mut self.layout, &self.font);

		o.save();
		{
			let w = f.width() * cell.width();
			let h = f.height() + c.style().spacing();
			let x = c.style().margin() + (cell.x() * f.width());
			let y = c.style().margin() + (cell.y() * h);

			// Clip to the cell area.
			o.rectangle(x as f64, y as f64, w as f64, h as f64);
			o.clip();

			// Set background.
			if let Some(bg) = cell.style().background() {
				o.rgba(bg);
			}
			else {
				o.rgba(&c.style().background());
			}
			o.paint();

			// Set foreground.
			if let Some(fg) = cell.style().foreground() {
				o.rgba(fg);
			}
			else {
				o.rgba(c.style().foreground());
			}

			if let &Cell::Char { ref value, .. } = cell {
				// Draw the glyph at the right point.
				o.move_to(x as f64, y as f64);
				o.text(l, value);
			}
		}
		o.restore();
	}
}

impl Deref for Renderer {
	type Target = cairo::Context;

	fn deref(&self) -> &Self::Target {
		&self.context
	}
}

impl DerefMut for Renderer {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.context
	}
}
