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
use libc::c_uint;

use super::{Metrics, Font};

#[derive(Debug)]
pub struct Set(pub *mut PangoFontset);

impl Set {
	pub fn metrics(&self) -> Metrics {
		unsafe {
			Metrics(pango_fontset_get_metrics(self.0))
		}
	}

	pub fn font(&self, ch: char) -> Font {
		unsafe {
			Font(pango_fontset_get_font(self.0, ch as c_uint))
		}
	}
}
