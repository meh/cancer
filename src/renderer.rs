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

use std::iter;
use std::mem;
use std::sync::Arc;
use std::ops::{Deref, DerefMut};

use picto::Area;
use config::Config;
use config::style::Shape;
use sys::cairo;
use sys::pango;
use font::Font;
use style;
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
		let width  = f.width() as f32;
		let height = (f.height() + s) as f32;

		// Cache terminal dimension.
		let columns = (self.width - (m * 2)) / width as u32;
		let rows    = (self.height - (m * 2)) / height as u32;

		// Remove the margin from coordinates.
		let x = area.x.saturating_sub(m) as f32;
		let y = area.y.saturating_sub(m) as f32;

		// Remove margins from width.
		let w = area.width
			.saturating_sub(m.saturating_sub(area.x))
			.saturating_sub(m.saturating_sub(self.width - (area.x + area.width))) as f32;

		// Remove margins from height.
		let h = area.height
			.saturating_sub(m.saturating_sub(area.y))
			.saturating_sub(m.saturating_sub(self.height - (area.y + area.height))) as f32;

		let x = (x / width).floor() as u32;
		let y = (y / height).floor() as u32;
		let w = (w / width).ceil() as u32;
		let h = (h / height).ceil() as u32;

		// Increment width and height by one if it fits within dimensions.
		//
		// This is done because the dirty area is actually bigger than the one
		// reported, or because the algorithm is broken. Regardless, this way it
		// works properly.
		Area::from(x, y,
			w + if x + w < columns { 1 } else { 0 },
			h + if y + h < rows { 1 } else { 0 })
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
		let (c, o) = (&self.config, &mut self.context);
		let m      = c.style().margin();

		// Bail out if there's no margin.
		if m == 0 {
			return;
		}

		o.save();
		{
			// Set to background color.
			o.rgba(c.style().color().background());

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

	/// Draw the cursor.
	pub fn cursor(&mut self, cell: &Cell, blinking: bool, focus: bool) {
		debug_assert!(!cell.is_reference());

		// Cache needed values in various places.
		//
		// FIXME(meh): Find better names/and or ways to deal with this stuff.
		let (c, o, l, f) = (&self.config, &mut self.context, &mut self.layout, &self.font);
		let     cb       = blinking && c.style().cursor().blink();
		let     bc       = blinking && cell.style().attributes().contains(style::BLINK);
		let mut fg       = cell.style().foreground().unwrap_or_else(|| c.style().color().foreground());
		let mut bg       = cell.style().background().unwrap_or_else(|| c.style().color().background());
		let     cfg      = c.style().cursor().foreground();
		let     cbg      = c.style().cursor().background();

		if cell.style().attributes().contains(style::REVERSE) {
			mem::swap(&mut fg, &mut bg);
		}

		o.save();
		{
			let w = f.width() * cell.width();
			let h = f.height() + c.style().spacing();
			let x = c.style().margin() + (cell.x() * f.width());
			let y = c.style().margin() + (cell.y() * h);

			// Clip to the cell area.
			o.rectangle(x as f64, y as f64, w as f64, h as f64);
			o.clip();

			// Clear the area for rendering.
			o.rgba(bg);
			o.paint();

			// Render cursors that require to be on the bottom.
			match c.style().cursor().shape() {
				// The block cursor only requires to be fully drawn when the window is
				// focused or when the terminal is blinking.
				//
				// It also changes the foreground color to the appropriate value.
				Shape::Block => {
					if focus && !cb {
						o.rgba(cbg);
						o.paint();
					}

					o.rgba(cfg);
				}

				// Other cursors keep the foreground color normal.
				Shape::Beam | Shape::Line => {
					o.rgba(fg);
				}
			}

			// Draw the text in the cell.
			o.move_to(x as f64, y as f64);
			l.attributes(attributes(c, cell));

			if cell.is_empty() {
				o.text(l, " ");
			}
			else if bc || cell.style().attributes().contains(style::INVISIBLE) {
				o.text(l, &iter::repeat(' ').take(cell.width() as usize).collect::<String>());
			}
			else {
				o.text(l, cell.char().unwrap());
			}

			// Render cursors that require to be on top.
			match c.style().cursor().shape() {
				// If the window is not focused or the terminal is blinking draw an
				// outline of the cell.
				Shape::Block => {
					if !focus || cb {
						o.rgba(cbg);
						o.rectangle(x as f64, y as f64, w as f64, h as f64);
						o.stroke();
					}
				}

				// The line always covers any glyph underneath, unless it's blinking.
				Shape::Line => {
					if !(cb && focus) {
						o.rgba(cbg);
						o.move_to(x as f64, (y + f.underline().1) as f64 + 0.5);
						o.line_to((x + w) as f64, (y + f.underline().1) as f64 + 0.5);
						o.line_width(f.underline().0 as f64);
						o.stroke();
					}
				}

				// The beam always covers any glyph underneath, unless it's blinking.
				Shape::Beam => {
					if !(cb && focus) {
						o.rgba(cbg);
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
	pub fn cell(&mut self, cell: &Cell, blinking: bool) {
		debug_assert!(!cell.is_reference());

		// Cache needed values in various places.
		//
		// FIXME(meh): Find better names/and or ways to deal with this stuff.
		let (c, o, l, f) = (&self.config, &mut self.context, &mut self.layout, &self.font);
		let mut fg       = cell.style().foreground().unwrap_or_else(|| c.style().color().foreground());
		let mut bg       = cell.style().background().unwrap_or_else(|| c.style().color().background());
		let     bc       = blinking && cell.style().attributes().contains(style::BLINK);

		if cell.style().attributes().contains(style::REVERSE) {
			mem::swap(&mut fg, &mut bg);
		}

		o.save();
		{
			let w = f.width() * cell.width();
			let h = f.height() + c.style().spacing();
			let x = c.style().margin() + (cell.x() * f.width());
			let y = c.style().margin() + (cell.y() * h);

			// Clip to the cell area.
			o.rectangle(x as f64, y as f64, w as f64, h as f64);
			o.clip();

			// Draw background.
			o.rgba(bg);
			o.paint();

			// Draw text in the cell.
			o.rgba(fg);
			o.move_to(x as f64, y as f64);
			l.attributes(attributes(c, cell));

			if cell.is_empty() {
				o.text(l, " ");
			}
			else if bc || cell.style().attributes().contains(style::INVISIBLE) {
				o.text(l, &iter::repeat(' ').take(cell.width() as usize).collect::<String>());
			}
			else {
				o.text(l, cell.char().unwrap());
			}
		}
		o.restore();
	}
}

/// Pango attribute builder from configuration and cell.
fn attributes(config: &Config, cell: &Cell) -> pango::Attributes {
	let attrs = pango::Attributes::new();
	let fg    = cell.style().foreground()
		.unwrap_or_else(|| config.style().color().foreground());

	// Set bold.
	let attrs = if cell.style().attributes().contains(style::BOLD) {
		attrs.weight(pango::Weight::Bold)
	}
	else if cell.style().attributes().contains(style::FAINT) {
		attrs.weight(pango::Weight::Light)
	}
	else {
		attrs.weight(pango::Weight::Normal)
	};

	// Set italic.
	let attrs = if cell.style().attributes().contains(style::ITALIC) {
		attrs.style(pango::Style::Italic)
	}
	else {
		attrs.style(pango::Style::Normal)
	};

	// Set underline.
	let attrs = if cell.style().attributes().contains(style::UNDERLINE) {
		attrs.underline(Some(config.style().color().underline().unwrap_or(fg)))
	}
	else {
		attrs.underline(None)
	};

	// Set strikethrough.
	let attrs = if cell.style().attributes().contains(style::STRUCK) {
		attrs.strikethrough(Some(config.style().color().strike().unwrap_or(fg)))
	}
	else {
		attrs.strikethrough(None)
	};

	attrs
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
