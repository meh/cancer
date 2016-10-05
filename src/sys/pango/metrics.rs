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

use libc::c_int;

use ffi::pango::*;

pub struct Metrics(pub *mut PangoFontMetrics);

impl Metrics {
	pub fn ascent(&self) -> u32 {
		unsafe {
			pixels(pango_font_metrics_get_ascent(self.0))
		}
	}

	pub fn descent(&self) -> u32 {
		unsafe {
			pixels(pango_font_metrics_get_descent(self.0))
		}
	}

	pub fn height(&self) -> u32 {
		self.descent() + self.ascent()
	}

	pub fn width(&self) -> u32 {
		unsafe {
			pixels(pango_font_metrics_get_approximate_digit_width(self.0))
		}
	}

	pub fn underline(&self) -> (u32, u32) {
		unsafe {
			(pixels(pango_font_metrics_get_underline_thickness(self.0)),
			 position(pango_font_metrics_get_underline_position(self.0),
			          pango_font_metrics_get_ascent(self.0)))

		}
	}

	pub fn strike(&self) -> (u32, u32) {
		unsafe {
			(pixels(pango_font_metrics_get_strikethrough_thickness(self.0)),
			 position(pango_font_metrics_get_strikethrough_position(self.0),
			          pango_font_metrics_get_ascent(self.0)))
		}
	}
}

impl Drop for Metrics {
	fn drop(&mut self) {
		unsafe {
			pango_font_metrics_unref(self.0);
		}
	}
}

#[inline(always)]
fn pixels(units: c_int) -> u32 {
	(units.abs() as u32 + 512) >> 10
}

#[inline(always)]
fn position(units: c_int, ascent: c_int) -> u32 {
	pixels(ascent - units)
}
