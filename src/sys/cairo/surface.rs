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

#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::os::raw::c_void;

#[cfg(target_os = "linux")]
use xcb;

#[cfg(target_os = "macos")]
use libc::c_uint;
#[cfg(target_os = "macos")]
use core_graphics::base::CGFloat;

use ffi::cairo::*;
use libc::c_int;

#[derive(Debug)]
pub struct Surface(pub *mut cairo_surface_t);

unsafe impl Send for Surface { }

impl Surface {
	pub fn flush(&self) {
		unsafe {
			cairo_surface_flush(self.0);
		}
	}
}

#[cfg(target_os = "linux")]
impl Surface {
	pub fn new(connection: &xcb::Connection, drawable: xcb::Drawable, visual: xcb::Visualtype, width: u32, height: u32) -> Self {
		unsafe {
			Surface(cairo_xcb_surface_create(connection.get_raw_conn(), drawable, &visual.base, width as c_int, height as c_int))
		}
	}
}

#[cfg(target_os = "macos")]
impl Surface {
	pub fn new(context: *mut c_void, width: u32, height: u32) -> Self {
		unsafe {
			CGContextTranslateCTM(context, 0.0, height as CGFloat);
			CGContextScaleCTM(context, 1.0, -1.0);

			Surface(cairo_quartz_surface_create_for_cg_context(context, width as c_uint, height as c_uint))
		}
	}
}

#[cfg(target_os = "windows")]
impl Surface {
	pub fn new(context: *mut c_void) -> Self {
		unsafe {
			Surface(cairo_win32_surface_create(context))
		}
	}
}

impl Drop for Surface {
	fn drop(&mut self) {
		unsafe {
			cairo_surface_destroy(self.0);
		}
	}
}
