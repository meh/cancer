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

use std::ptr;

use ffi::pango::*;
use ffi::glib::*;

use super::{Description, Map, Set};

pub struct Context(pub *mut PangoContext);

impl Context {
	pub fn new(map: &Map) -> Self {
		unsafe {
			Context(pango_font_map_create_context(map.0))
		}
	}

	pub fn font<S: AsRef<str>>(&self, name: S) -> Option<Set> {
		let description = Description::from(name);

		unsafe {
			pango_context_set_font_description(self.0, description.0);
			pango_context_load_fontset(self.0, description.0, ptr::null_mut())
				.as_mut().map(|v| Set(v as *mut _))
		}
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			g_object_unref(self.0 as *mut _);
		}
	}
}
