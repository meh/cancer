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

use picto::color::{Rgb, Rgba};

use ffi::cairo::*;
use ffi::pango::*;
use sys::pango;
use super::Surface;

pub struct Context(pub *mut cairo_t);

impl Context {
	pub fn new<S: AsRef<Surface>>(surface: S) -> Self {
		unsafe {
			Context(cairo_create(surface.as_ref().0))
		}
	}

	pub fn push(&mut self) {
		unsafe {
			cairo_push_group(self.0);
		}
	}

	pub fn pop(&mut self) {
		unsafe {
			cairo_pop_group_to_source(self.0);
		}
	}

	pub fn save(&mut self) {
		unsafe {
			cairo_save(self.0);
		}
	}

	pub fn restore(&mut self) {
		unsafe {
			cairo_restore(self.0);
		}
	}

	pub fn rectangle(&mut self, x: f64, y: f64, width: f64, height: f64) {
		unsafe {
			cairo_rectangle(self.0, x, y, width, height);
		}
	}

	pub fn clip(&mut self) {
		unsafe {
			cairo_clip(self.0);
		}
	}

	pub fn rgb(&mut self, px: &Rgb<f64>) {
		unsafe {
			cairo_set_source_rgb(self.0, px.red, px.green, px.blue);
		}
	}

	pub fn rgba(&mut self, px: &Rgba<f64>) {
		unsafe {
			cairo_set_source_rgba(self.0, px.red, px.green, px.blue, px.alpha);
		}
	}

	pub fn move_to(&mut self, x: f64, y: f64) {
		unsafe {
			cairo_move_to(self.0, x, y);
		}
	}

	pub fn line_to(&mut self, x: f64, y: f64) {
		unsafe {
			cairo_line_to(self.0, x, y);
		}
	}

	pub fn line_width(&mut self, w: f64) {
		unsafe {
			cairo_set_line_width(self.0, w);
		}
	}

	pub fn fill(&mut self, preserve: bool) {
		unsafe {
			if preserve {
				cairo_fill_preserve(self.0);
			}
			else {
				cairo_fill(self.0);
			}
		}
	}

	pub fn stroke(&mut self) {
		unsafe {
			cairo_stroke(self.0);
		}
	}

	pub fn paint(&mut self) {
		unsafe {
			cairo_paint(self.0)
		}
	}

	pub fn glyph<T: AsRef<str>>(&mut self, text: T, glyph: &pango::GlyphItem) {
		let text = text.as_ref();

		unsafe {
			pango_cairo_show_glyph_item(self.0, text.as_ptr() as *const _, &glyph.0);
		}
	}
}

impl AsRef<Context> for Context {
	fn as_ref(&self) -> &Context {
		self
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			cairo_destroy(self.0);
		}
	}
}
