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

use ffi::cairo::*;
use ffi::pango::*;

use super::Context;
use sys::cairo;

pub struct Layout(pub *mut PangoLayout);

impl Layout {
	pub fn new<C: AsRef<Context>>(context: C) -> Self {
		unsafe {
			Layout(pango_layout_new(context.as_ref().0))
		}
	}

	pub fn update<C: AsRef<cairo::Context>>(&mut self, context: C) {
		unsafe {
			pango_cairo_update_layout(context.as_ref().0, self.0);
		}
	}
}
