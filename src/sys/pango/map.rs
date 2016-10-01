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

use std::ffi::CString;

use ffi::cairo::*;
use ffi::pango::*;
use ffi::glib::*;

pub struct Map(pub *mut PangoFontMap);

impl Map {
	pub fn new() -> Self {
		unsafe {
			Map(pango_cairo_font_map_new())
		}
	}
}

impl Drop for Map {
	fn drop(&mut self) {
		unsafe {
			g_object_unref(self.0 as *mut _);
		}
	}
}
