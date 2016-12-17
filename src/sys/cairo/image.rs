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
use std::cell::Cell;

use libc::{c_uchar, c_int};
use ffi::cairo::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Image {
	width:  u32,
	height: u32,
	stride: u32,

	buffer:  Vec<u8>,
	pattern: Cell<*mut cairo_pattern_t>,
}

impl Image {
	pub fn new(width: u32, height: u32) -> Self {
		unsafe {
			let stride = cairo_format_stride_for_width(cairo_format_t::Argb32, width as c_int) as u32;
			let buffer = vec![0u8; (stride * height) as usize];

			Image {
				width:  width,
				height: height,
				stride: stride,

				buffer:  buffer,
				pattern: Cell::new(ptr::null_mut()),
			}
		}
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn stride(&self) -> u32 {
		self.stride
	}

	pub fn set(&mut self, x: u32, y: u32, (r, g, b, a): (u8, u8, u8, u8)) {
		let offset = ((x * 4) + (y * self.stride)) as usize;

		self.buffer[offset + 0] = b;
		self.buffer[offset + 1] = g;
		self.buffer[offset + 2] = r;
		self.buffer[offset + 3] = a;
	}

	pub fn as_ptr(&self) -> *mut cairo_pattern_t {
		unsafe {
			if let Some(ptr) = self.pattern.get().as_mut() {
				ptr
			}
			else {
				let surface = cairo_image_surface_create_for_data(self.buffer.as_ptr(), cairo_format_t::Argb32,
					self.width as c_int, self.height as c_int, self.stride as c_int);

				let pattern = cairo_pattern_create_for_surface(surface);

				self.pattern.set(pattern);
				pattern
			}
		}
	}
}

impl Drop for Image {
	fn drop(&mut self) {
		unsafe {
			if let Some(ptr) = self.pattern.get().as_mut() {
				cairo_pattern_destroy(ptr);
			}
		}
	}
}
