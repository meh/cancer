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
use std::sync::Arc;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};

use picto::Region;
use config::Config;
use config::style::Shape;
use sys::cairo;
use font::Font;
use style;
use util::clamp;
use terminal::{cell, cursor};
use interface::Interface;
use renderer::{Cache, Glyphs, Options};

/// Renderer for a `cairo::Surface`.
pub struct Renderer {
	config: Arc<Config>,
	width:  u32,
	height: u32,

	spacing: u32,
	margin:  Margin,

	context: cairo::Context,
	font:    Arc<Font>,
	cache:   Cache,
	glyphs:  Glyphs,
}

/// Adaptable margins depending on the view size.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Margin {
	horizontal: u32,
	vertical:   u32,
}

impl Renderer {
	/// Get the window dimensions for the given grid.
	pub fn dimensions(columns: u32, rows: u32, config: &Config, font: &Font) -> (u32, u32) {
		let margin  = config.style().margin();
		let spacing = config.style().spacing();

		let width  = (columns * font.width()) + (margin * 2);
		let height = (rows * (font.height() + spacing)) + (margin * 2);

		(width, height)
	}

	/// Create a new renderer for the given settings and surface.
	pub fn new(config: Arc<Config>, font: Arc<Font>, surface: &cairo::Surface, width: u32, height: u32) -> Self {
		let spacing = config.style().spacing();
		let margin  = Margin {
			horizontal: config.style().margin(),
			vertical:   config.style().margin(),
		};

		let context = cairo::Context::new(surface);
		let cache   = Cache::new(0, 0);
		let glyphs  = Glyphs::new(config.environment().cache(), font.clone());

		let mut value = Renderer {
			config: config,
			width:  0,
			height: 0,

			spacing: spacing,
			margin:  margin,

			context: context,
			font:    font,
			cache:   cache,
			glyphs:  glyphs,
		};

		value.resize(width, height);
		value
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

	/// Resize the renderer viewport.
	pub fn resize(&mut self, width: u32, height: u32) {
		let (m, s) = (self.config.style().margin(), self.spacing);

		self.margin.horizontal = m +
			((width - (m * 2)) % self.font.width()) / 2;

		self.margin.vertical = m +
			((height - (m * 2)) % (self.font.height() + s)) / 2;

		self.width  = width;
		self.height = height;

		let rows    = self.rows();
		let columns = self.columns();
		self.cache.resize(columns, rows);
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
	pub fn damaged(&mut self, region: &Region) -> Region {
		let (f, h, v, s) = (&self.font, self.margin.horizontal, self.margin.vertical, self.spacing);

		// Check if the region falls exactly within a margin, if so bail out.
		if h != 0 && v != 0 &&
		   ((region.x < h && region.width <= h - region.x) ||
		    (region.x >= self.width - h) ||
		    (region.y < v && region.height <= v - region.y) ||
		    (region.y >= self.height - v))
		{
			return Region::from(0, 0, 0, 0);
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
		Region::from(x, y,
			w + if x + w < self.columns() { 1 } else { 0 },
			h + if y + h < self.rows() { 1 } else { 0 })
	}

	/// Batch the drawing operations within the closure.
	pub fn batch<T, F: FnOnce(&mut Self) -> T>(&mut self, func: F) -> T {
		self.push();
		let out = func(self);
		self.pop();
		self.paint();

		out
	}

	/// Draw the margins within the given region.
	pub fn margin(&mut self, region: &Region) {
		let (rows, columns)    = (self.rows(), self.columns());
		let (c, f, o, s, h, v) = (&self.config, &self.font, &mut self.context, self.spacing, self.margin.horizontal, self.margin.vertical);

		// Bail out if there's no margin.
		if h == 0 && v == 0 {
			return;
		}

		o.save();
		{
			// Set to background color.
			o.rgba(c.style().color().background());

			// Left margin.
			if region.x < h {
				o.rectangle(0.0, region.y as f64, h as f64, region.height as f64);
				o.fill(false);
			}

			// Right margin.
			if region.x + region.width >= self.width - h {
				o.rectangle((h + (columns * f.width())) as f64, region.y as f64, h as f64 * 2.0, region.height as f64);
				o.fill(false);
			}

			// Top margin.
			if region.y < v {
				o.rectangle(region.x as f64, 0.0, region.width as f64, v as f64);
				o.fill(false);
			}

			// Bottom margin.
			if region.y + region.height >= self.height - v {
				o.rectangle(region.x as f64, (v + (rows * (f.height() + s))) as f64, region.width as f64, v as f64 * 2.0);
				o.fill(false);
			}
		}
		o.restore();
	}

	pub fn update<I>(&mut self, interface: &Interface, iter: I, options: Options)
		where I: Iterator<Item = (u32, u32)>
	{
		for cell in interface.iter(iter) {
			self.cell(&cell, options);
		}

		if options.cursor() {
			self.cursor(&interface.cursor(), options);
		}
		else {
			self.cell(&interface.cursor().cell(), options);
		}
	}

	/// Draw the cursor.
	fn cursor(&mut self, cursor: &cursor::Cell, options: Options) {
		self.cache.invalidate(&cursor.cell());

		let (c, o, f) = (&self.config, &mut self.context, &self.font);
		let cell = cursor.cell();
		let bc   = options.blinking() && cursor.blink();
		let fg   = cursor.foreground();
		let bg   = cursor.background();

		let w = f.width() * cell.width();
		let h = f.height() + c.style().spacing();
		let x = self.margin.horizontal + (cell.x() * f.width());
		let y = self.margin.vertical + (cell.y() * h);

		o.save();
		{
			// Draw the background.
			o.rectangle(x as f64, y as f64, w as f64, h as f64);
			o.clip();

			match cursor.shape() {
				Shape::Block => {
					if options.focus() && !bc {
						o.rgba(bg);
					}
					else {
						o.rgba(c.style().color().background());
					}
				}

				Shape::Beam | Shape::Line => {
					o.rgba(cell.style().background().unwrap_or(
						c.style().color().background()));
				}
			}

			o.paint();

			// Draw the glyph.
			if cell.is_occupied() && !(options.blinking() && cell.style().attributes().contains(style::BLINK)) {
				o.move_to(x as f64, (y + f.ascent()) as f64);

				match cursor.shape() {
					Shape::Block => {
						o.rgba(fg);
					}

					Shape::Beam | Shape::Line => {
						o.rgba(cell.style().foreground().unwrap_or(
							c.style().color().foreground()));
					}
				}

				let computed = self.glyphs.compute(Rc::new(cell.value().into()), cell.style().attributes());
				o.glyph(computed.text(), computed.glyphs());
			}

			// Render cursors that require to be on top.
			match cursor.shape() {
				// If the window is not focused or the terminal is blinking draw an
				// outline of the cell.
				Shape::Block => {
					if !options.focus() || bc {
						o.rectangle(x as f64 + 0.5, y as f64 + 0.5, w as f64 - 1.0, h as f64 - 1.0);
						o.line_width(1.0);
						o.rgba(bg);
						o.stroke();
					}
				}

				// The line always covers any glyph underneath, unless it's blinking.
				Shape::Line => {
					if !(bc && options.focus()) {
						o.move_to(x as f64, (y + f.underline().1) as f64 + 0.5);
						o.line_to((x + w) as f64, (y + f.underline().1) as f64 + 0.5);
						o.line_width(f.underline().0 as f64);
						o.rgba(bg);
						o.stroke();
					}
				}

				// The beam always covers any glyph underneath, unless it's blinking.
				Shape::Beam => {
					if !(bc && options.focus()) {
						o.move_to(x as f64 + 0.5, y as f64);
						o.line_to(x as f64 + 0.5, (y + h) as f64);
						o.line_width(f.underline().0 as f64);
						o.rgba(bg);
						o.stroke();
					}
				}
			}
		}
		o.restore();
	}

	/// Draw the given cell.
	fn cell(&mut self, cell: &cell::Position, options: Options) -> bool {
		// Bail out if the cell is up to date.
		if !self.cache.update(cell, options) && !options.damage() {
			return false;
		}

		let (c, o, f) = (&self.config, &mut self.context, &self.font);

		let mut fg = cell.style().foreground().unwrap_or_else(||
			c.style().color().foreground());

		let mut bg = cell.style().background().unwrap_or_else(||
			c.style().color().background());

		if options.reverse() {
			mem::swap(&mut fg, &mut bg);
		}

		if cell.style().attributes().contains(style::REVERSE) {
			mem::swap(&mut fg, &mut bg);
		}

		let w = f.width() * cell.width();
		let h = f.height() + c.style().spacing();
		let x = self.margin.horizontal + (cell.x() * f.width());
		let y = self.margin.vertical + (cell.y() * h);

		o.save();
		{
			// Draw the background.
			o.rectangle(x as f64, y as f64, w as f64, h as f64);
			o.clip();
			o.rgba(bg);
			o.paint();

			// Draw the glyph.
			if !cell.style().attributes().contains(style::BLINK) || !options.blinking() {
				if cell.is_occupied() {
					o.move_to(x as f64, (y + f.ascent()) as f64);
					o.rgba(fg);

					let computed = self.glyphs.compute(Rc::new(cell.value().into()), cell.style().attributes());
					o.glyph(computed.text(), computed.glyphs());
				}

				// Draw underline.
				if cell.style().attributes().contains(style::UNDERLINE) {
					let (thickness, position) = f.underline();

					o.rgba(c.style().color().underline().unwrap_or(fg));
					o.rectangle(x as f64, (y + position) as f64, w as f64, thickness as f64);
					o.line_width(1.0);
					o.fill(false);
				}

				// Draw strikethrough.
				if cell.style().attributes().contains(style::STRUCK) {
					let (thickness, position) = f.strikethrough();

					o.rgba(c.style().color().strikethrough().unwrap_or(fg));
					o.rectangle(x as f64, (y + position) as f64, w as f64, thickness as f64);
					o.line_width(1.0);
					o.fill(false);
				}
			}
		}
		o.restore();

		true
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
