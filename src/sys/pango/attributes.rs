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
use super::{Weight, Style};
use picto::color::Rgb;

pub struct Attributes(pub *mut PangoAttrList);

impl Attributes {
	pub fn new() -> Self {
		unsafe {
			Attributes(pango_attr_list_new())
		}
	}

	pub fn weight(self, weight: Weight) -> Self {
		unsafe {
			pango_attr_list_insert(self.0,
				pango_attr_weight_new(weight));
		}

		self
	}

	pub fn style(self, style: Style) -> Self {
		unsafe {
			pango_attr_list_insert(self.0,
				pango_attr_style_new(style));
		}

		self
	}
}

impl Drop for Attributes {
	fn drop(&mut self) {
		unsafe {
			pango_attr_list_unref(self.0)
		}
	}
}
