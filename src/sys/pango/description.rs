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
use ffi::pango::*;

use super::{Weight, Style};

pub struct Description(pub *mut PangoFontDescription);

impl Description {
	pub fn from<S: AsRef<str>>(name: S) -> Self {
		let name = CString::new(name.as_ref()).unwrap();

		unsafe {
			Description(pango_font_description_from_string(name.as_ptr()))
		}
	}

	pub fn weight(&mut self, weight: Weight) {
		unsafe {
			pango_font_description_set_weight(self.0, weight);
		}
	}

	pub fn style(&mut self, style: Style) {
		unsafe {
			pango_font_description_set_style(self.0, style);
		}
	}
}

impl Drop for Description {
	fn drop(&mut self) {
		unsafe {
			pango_font_description_free(self.0);
		}
	}
}
