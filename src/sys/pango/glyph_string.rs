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

#[derive(Debug)]
pub struct GlyphString(pub *mut PangoGlyphString);

impl GlyphString {
	pub fn new() -> Self {
		unsafe {
			GlyphString(pango_glyph_string_new())
		}
	}
}

impl Clone for GlyphString {
	fn clone(&self) -> Self {
		unsafe {
			GlyphString(pango_glyph_string_copy(self.0))
		}
	}
}

impl Drop for GlyphString {
	fn drop(&mut self) {
		unsafe {
			pango_glyph_string_free(self.0);
		}
	}
}
