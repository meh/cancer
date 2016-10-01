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
use std::ops::{Deref, DerefMut};

use unicode_width::UnicodeWidthStr;
use picto::Area;
use config::Config;
use sys::cairo;
use sys::pango;
use font::Font;
use style::Style;

pub struct Renderer {
	config: Arc<Config>,
	font:   Arc<Font>,
	width:  u32,
	height: u32,

	context: cairo::Context,
	layout:  pango::Layout,
}

impl Renderer {
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

	pub fn rows(&self) -> u32 {
		(self.height - (self.config.style().margin() * 2)) /
			(self.font.height() + self.config.style().spacing())
	}

	pub fn columns(&self) -> u32 {
		(self.width - (self.config.style().margin() * 2)) /
			self.font.width()
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		self.width  = width;
		self.height = height;
	}

	pub fn damaged(&self, area: &Area) -> Area {
		Area::from(0, 0, self.columns(), self.rows())
	}

	pub fn update<T, F: FnOnce(&mut Self) -> T>(&mut self, func: F) -> T {
		self.push();
		let out = func(self);
		self.pop();
		self.paint();

		out
	}

	pub fn margin(&mut self, area: &Area) {
		let (c, o, l, f) = (&self.config, &mut self.context, &mut self.layout, &self.font);
		let m            = c.style().margin();

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

	pub fn cell<S: AsRef<str>>(&mut self, x: u32, y: u32, ch: S, style: &Style) {
		let (c, o, l, f) = (&self.config, &mut self.context, &mut self.layout, &self.font);
		let ch           = ch.as_ref();

		o.save();
		{
			let w = f.width() * ch.width() as u32;
			let h = f.height() + c.style().spacing();
			let x = c.style().margin() + (x * f.width());
			let y = c.style().margin() + (y * h);

			// Clip to the cell area.
			o.rectangle(x as f64, y as f64, w as f64, h as f64);
			o.clip();

			// Set background.
			if let Some(bg) = style.background() {
				o.rgba(bg);
			}
			else {
				o.rgba(&c.style().background());
			}
			o.paint();

			// Set foreground.
			if let Some(fg) = style.foreground() {
				o.rgba(fg);
			}
			else {
				o.rgba(c.style().foreground());
			}

			// Draw the glyph at the right point.
			o.move_to(x as f64, y as f64);
			o.text(l, ch);
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
