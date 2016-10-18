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
use ffi::pango::*;
use sys::pango::{Item, GlyphString};

#[derive(Debug)]
pub struct GlyphItem(pub PangoGlyphItem);

impl GlyphItem {
	pub fn new(item: Item, string: GlyphString) -> Self {
		let item_ptr   = item.0;
		let string_ptr = string.0;

		mem::forget(item);
		mem::forget(string);

		GlyphItem(PangoGlyphItem {
			item:   item_ptr,
			string: string_ptr,
		})
	}
}

impl Drop for GlyphItem {
	fn drop(&mut self) {
		unsafe {
			Item(self.0.item);
			GlyphString(self.0.string);
		}
	}
}
