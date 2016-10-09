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
use super::{Weight, Style, Attribute};
use picto::color::Rgb;

#[derive(Debug)]
pub struct AttributeList(pub *mut PangoAttrList);

impl AttributeList {
	pub fn new() -> Self {
		unsafe {
			AttributeList(pango_attr_list_new())
		}
	}

	pub fn change(&mut self, attr: Attribute) {
		unsafe {
			pango_attr_list_change(self.0, attr.take());
		}
	}
}

impl Drop for AttributeList {
	fn drop(&mut self) {
		unsafe {
			pango_attr_list_unref(self.0)
		}
	}
}
