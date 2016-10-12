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

use ffi::pango::*;
use ffi::glib::*;

use super::Description;

#[derive(Debug)]
pub struct Font(pub *mut PangoFont);

impl Font {
	pub fn description(&self) -> Description {
		unsafe {
			Description(pango_font_describe(self.0))
		}
	}
}

impl Clone for Font {
	fn clone(&self) -> Self {
		unsafe {
			Font(g_object_ref(self.0 as *mut _) as *mut _)
		}
	}
}

impl Drop for Font {
	fn drop(&mut self) {
		unsafe {
			g_object_unref(self.0 as *mut _);
		}
	}
}
