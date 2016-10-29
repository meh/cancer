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
use std::ops::{Deref, DerefMut};
use std::cmp;

use picto::Area;
use config::Config;
use config::style::Shape;
use sys::cairo;
use font::Font;
use style;
use terminal::{cell, cursor};
use renderer::{Cache, Options};

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
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Margin {
	horizontal: u32,
	vertical:   u32,
}

impl Renderer {
	/// Create a new renderer for the given settings and surface.
	pub fn new<S: AsRef<cairo::Surface>>(config: Arc<Config>, font: Arc<Font>, surface: S, width: u32, height: u32) -> Self {
		let spacing = config.style().spacing();
		let margin  = Margin {
			horizontal: config.style().margin(),
			vertical:   config.style().margin(),
		};

		let context = cairo::Context::new(surface);
		let cache   = Cache::new(config.environment().cache(), font.clone(), 0, 0);

		let mut value = Renderer {
			config: config,
			width:  0,
			height: 0,

			spacing: spacing,
			margin:  margin,

			context: context,
			font:    font,
			cache:   cache,
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

	/// Turn the damaged area to cell-space.
	///
	/// FIXME(meh): this is kinda approximate, should find a proper algorithm.
	pub fn damaged(&mut self, area: &Area) -> Area {
		let (f, h, v, s) = (&self.font, self.margin.horizontal, self.margin.vertical, self.spacing);

		// Check if the area falls exactly within a margin, if so bail out.
		if h != 0 && v != 0 &&
		   ((area.x < h && area.width <= h - area.x) ||
		    (area.x >= self.width - h) ||
		    (area.y < v && area.height <= v - area.y) ||
		    (area.y >= self.height - v))
		{
			return Area::from(0, 0, 0, 0);
		}

		// Cache font dimensions.
		let width  = f.width() as f32;
		let height = (f.height() + s) as f32;

		// Remove the margin from coordinates.
		let x = area.x.saturating_sub(h) as f32;
		let y = area.y.saturating_sub(v) as f32;

		// Remove margins from width.
		let w = area.width
			.saturating_sub(h.saturating_sub(area.x))
			.saturating_sub(h.saturating_sub(self.width - (area.x + area.width))) as f32;

		// Remove margins from height.
		let h = area.height
			.saturating_sub(v.saturating_sub(area.y))
			.saturating_sub(v.saturating_sub(self.height - (area.y + area.height))) as f32;

		let x = (x / width).floor() as u32;
		let y = (y / height).floor() as u32;
		let w = cmp::min(self.columns(), (w / width).ceil() as u32);
		let h = cmp::min(self.rows(), (h / height).ceil() as u32);

		// Increment width and height by one if it fits within dimensions.
		//
		// This is done because the dirty area is actually bigger than the one
		// reported, or because the algorithm is broken. Regardless, this way it
		// works properly.
		Area::from(x, y,
			w + if x + w < self.columns() { 1 } else { 0 },
			h + if y + h < self.rows() { 1 } else { 0 })
	}

	/// Batch the drawing operations within the closure.
	pub fn update<T, F: FnOnce(&mut Self) -> T>(&mut self, func: F) -> T {
		self.push();
		let out = func(self);
		self.pop();
		self.paint();

		out
	}

	/// Draw the margins within the given area.
	pub fn margin(&mut self, area: &Area) {
		let (c, o, h, v) = (&self.config, &mut self.context, self.margin.horizontal, self.margin.vertical);

		// Bail out if there's no margin.
		if h == 0 && v == 0 {
			return;
		}

		o.save();
		{
			// Set to background color.
			o.rgba(c.style().color().background());

			// Left margin.
			if area.x < h {
				o.rectangle(0.0, area.y as f64, h as f64, area.height as f64);
				o.fill(false);
			}

			// Right margin.
			if area.x + area.width >= self.width - h {
				o.rectangle((self.width - h) as f64, area.y as f64, h as f64, area.height as f64);
				o.fill(false);
			}

			// Top margin.
			if area.y < v {
				o.rectangle(area.x as f64, 0.0, area.width as f64, v as f64);
				o.fill(false);
			}

			// Bottom margin.
			if area.y + area.height >= self.height - v {
				o.rectangle(area.x as f64, (self.height - v) as f64, area.width as f64, v as f64);
				o.fill(false);
			}
		}
		o.restore();
	}

	/// Draw the cursor.
	pub fn cursor(&mut self, cursor: &cursor::Cell, options: Options) {
		self.cache.invalidate(&cursor.cell());

		// Cache needed values in various places.
		//
		// FIXME(meh): Find better names/and or ways to deal with this stuff.
		let (c, o, f) = (&self.config, &mut self.context, &self.font);
		let cell = cursor.cell();
		let bc   = options.blinking() && cursor.blink();
		let fg   = cursor.foreground();
		let bg   = cursor.background();

		let w = f.width() * cell.width();
		let h = f.height() + c.style().spacing();
		let x = c.style().margin() + (cell.x() * f.width());
		let y = c.style().margin() + (cell.y() * h);

		o.save();
		{
			// Draw the background.
			o.rectangle(x as f64, y as f64, w as f64, h as f64);
			o.line_width(1.0);

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

			o.fill(false);

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

				let computed = self.cache.compute(cell.value(), cell.style().attributes());
				o.glyph(computed.text(), computed.glyphs());
			}

			// Render cursors that require to be on top.
			match cursor.shape() {
				// If the window is not focused or the terminal is blinking draw an
				// outline of the cell.
				Shape::Block => {
					if !options.focus() || bc {
						o.rgba(bg);
						o.rectangle(x as f64 + 0.5, y as f64 + 0.5, w as f64 - 1.0, h as f64 - 1.0);
						o.line_width(1.0);
						o.stroke();
					}
				}

				// The line always covers any glyph underneath, unless it's blinking.
				Shape::Line => {
					if !(bc && options.focus()) {
						o.rgba(bg);
						o.move_to(x as f64, (y + f.underline().1) as f64 + 0.5);
						o.line_to((x + w) as f64, (y + f.underline().1) as f64 + 0.5);
						o.line_width(f.underline().0 as f64);
						o.stroke();
					}
				}

				// The beam always covers any glyph underneath, unless it's blinking.
				Shape::Beam => {
					if !(bc && options.focus()) {
						o.rgba(bg);
						o.move_to(x as f64 + 0.5, y as f64);
						o.line_to(x as f64 + 0.5, (y + h) as f64);
						o.line_width(f.underline().0 as f64);
						o.stroke();
					}
				}
			}
		}
		o.restore();
	}

	/// Draw the given cell.
	pub fn cell(&mut self, cell: &cell::Position, options: Options, force: bool) -> bool {
		// Bail out if the character is up to date.
		if !force && !self.cache.update(cell, options) {
			return false;
		}

		// Cache needed values in various places.
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
			o.line_width(1.0);
			o.rgba(bg);
			o.fill(false);

			// Draw the glyph.
			if !(options.blinking() && cell.style().attributes().contains(style::BLINK)) {
				if cell.is_occupied() {
					o.move_to(x as f64, (y + f.ascent()) as f64);
					o.rgba(fg);

					let computed = self.cache.compute(cell.value(), cell.style().attributes());
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
