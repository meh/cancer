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
use std::rc::Rc;

use util::Region;
use config::style::Shape;
use sys::cairo;
use style;
use terminal::{cell, cursor};
use interface::Interface;
use renderer::{State, Options};
use renderer::standard::{Cache, Glyphs};

/// Renderer for a `cairo::Surface`.
pub struct Renderer {
	context: cairo::Context,
	cache:   Cache,
	glyphs:  Glyphs,
}

unsafe impl Send for Renderer { }

impl Renderer {
	/// Create a new renderer for the given settings and surface.
	pub fn new(surface: &cairo::Surface, state: &State) -> Self {
		let context = cairo::Context::new(surface);
		let cache   = Cache::new(state.columns(), state.rows());
		let glyphs  = Glyphs::new(state.config().environment().cache(), state.font().clone());

		Renderer {
			context: context,
			cache:   cache,
			glyphs:  glyphs,
		}
	}

	/// Resize the renderer viewport.
	pub fn resize(&mut self, surface: &cairo::Surface, state: &State) {
		self.context = cairo::Context::new(surface);
		self.cache.resize(state.columns(), state.rows());
	}

	/// Render the given changes.
	pub fn render<I>(&mut self, state: &State, options: Options, region: Option<Region>, interface: &Interface, iter: I)
		where I: Iterator<Item = (u32, u32)>
	{
		if let Some(region) = region {
			self.margin(state, &region);
		}

		for cell in interface.iter(iter) {
			self.cell(state, &cell, options);
		}

		if options.cursor() {
			self.cursor(state, &interface.cursor(), options);
		}
		else {
			self.cell(state, &interface.cursor().cell(), options);
		}
	}

	/// Draw the margins within the given region.
	pub fn margin(&mut self, state: &State, region: &Region) {
		let (rows, columns)    = (state.rows(), state.columns());
		let (c, f, o, s, h, v) = (state.config(), state.font(), &mut self.context, state.spacing(), state.margin().horizontal, state.margin().vertical);

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
				o.fill();
			}

			// Right margin.
			if region.x + region.width >= state.width() - h {
				o.rectangle((h + (columns * f.width())) as f64, region.y as f64, h as f64 * 2.0, region.height as f64);
				o.fill();
			}

			// Top margin.
			if region.y < v {
				o.rectangle(region.x as f64, 0.0, region.width as f64, v as f64);
				o.fill();
			}

			// Bottom margin.
			if region.y + region.height >= state.height() - v {
				o.rectangle(region.x as f64, (v + (rows * (f.height() + s))) as f64, region.width as f64, v as f64 * 2.0);
				o.fill();
			}
		}
		o.restore();
	}

	/// Draw the cursor.
	fn cursor(&mut self, state: &State, cursor: &cursor::Cell, options: Options) {
		self.cache.invalidate(&cursor.cell());

		let (c, o, f) = (state.config(), &mut self.context, state.font());
		let cell = cursor.cell();
		let bc   = options.blinking() && cursor.blink();
		let fg   = cursor.foreground();
		let bg   = cursor.background();

		let w = f.width() * cell.width();
		let h = f.height() + c.style().spacing();
		let x = state.margin().horizontal + (cell.x() * f.width());
		let y = state.margin().vertical + (cell.y() * h);

		o.save();
		{
			// Draw the background.
			o.rectangle(x as f64, y as f64, w as f64, h as f64);
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
			o.fill();

			// Draw the glyph.
			if cell.is_occupied() && !(options.blinking() && cell.style().attributes().contains(style::BLINK)) {
				o.save();
				o.rectangle(x as f64, y as f64, w as f64, h as f64);
				o.clip();
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
				o.restore();
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
	fn cell(&mut self, state: &State, cell: &cell::Position, options: Options) -> bool {
		// Bail out if the cell is up to date.
		if !self.cache.update(cell, options) && !options.damage() {
			return false;
		}

		let (c, o, f) = (state.config(), &mut self.context, state.font());

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
		let x = state.margin().horizontal + (cell.x() * f.width());
		let y = state.margin().vertical + (cell.y() * h);

		o.save();
		{
			// Draw the background.
			o.rectangle(x as f64, y as f64, w as f64, h as f64);
			o.rgba(bg);
			o.fill();

			// Draw the glyph.
			if !cell.style().attributes().contains(style::BLINK) || !options.blinking() {
				if cell.is_occupied() {
					o.save();
					o.rectangle(x as f64, y as f64, w as f64, h as f64);
					o.clip();

					o.move_to(x as f64, (y + f.ascent()) as f64);
					o.rgba(fg);
					let computed = self.glyphs.compute(Rc::new(cell.value().into()), cell.style().attributes());
					o.glyph(computed.text(), computed.glyphs());

					o.restore();
				}

				// Draw underline.
				if cell.style().attributes().contains(style::UNDERLINE) {
					let (thickness, position) = f.underline();

					o.rgba(c.style().color().underline().unwrap_or(fg));
					o.rectangle(x as f64, (y + position) as f64, w as f64, thickness as f64);
					o.line_width(1.0);
					o.fill();
				}

				// Draw strikethrough.
				if cell.style().attributes().contains(style::STRUCK) {
					let (thickness, position) = f.strikethrough();

					o.rgba(c.style().color().strikethrough().unwrap_or(fg));
					o.rectangle(x as f64, (y + position) as f64, w as f64, thickness as f64);
					o.line_width(1.0);
					o.fill();
				}
			}
		}
		o.restore();

		true
	}
}
