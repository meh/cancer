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

use std::os::raw::c_void;
use libc::{c_int, c_uchar, c_double};

#[repr(C)]
pub struct cairo_t(c_void);

#[repr(C)]
pub struct cairo_device_t(c_void);

#[repr(C)]
pub struct cairo_surface_t(c_void);

#[repr(C)]
pub struct cairo_pattern_t(c_void);

#[repr(C)]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum cairo_format_t {
	Invalid = -1,
	Argb32,
	Rgb24,
	A8,
	A1,
	Rgb16_565,
	Rgb30,
}

#[repr(C)]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum cairo_operator_t {
	Clear,
	Source,
	Over,
	In,
	Out,
	Atop,
	Dest,
	DestOver,
	DestIn,
	DestOut,
	DestAtop,
	Xor,
	Add,
	Saturate,
	Multiply,
	Screen,
	Overlay,
	Darken,
	Lighten,
	ColorDodge,
	ColorBurn,
	HardLight,
	SoftLight,
	Difference,
	Exclusion,
	HslHue,
	HslSaturation,
	HslColor,
	HslLuminosity,
}

#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct cairo_matrix_t {
	pub xx: c_double,
	pub yx: c_double,

	pub xy: c_double,
	pub yy: c_double,

	pub x0: c_double,
	pub y0: c_double,
}

extern "C" {
	pub fn cairo_format_stride_for_width(format: cairo_format_t, width: c_int) -> c_int;

	pub fn cairo_matrix_init_translate(matrix: *mut cairo_matrix_t, x: c_double, y: c_double);

	pub fn cairo_create(surface: *mut cairo_surface_t) -> *mut cairo_t;
	pub fn cairo_destroy(cr: *mut cairo_t);

	pub fn cairo_push_group(cr: *mut cairo_t);
	pub fn cairo_pop_group_to_source(cr: *mut cairo_t);

	pub fn cairo_save(cr: *mut cairo_t);
	pub fn cairo_restore(cr: *mut cairo_t);

	pub fn cairo_paint(cr: *mut cairo_t);
	pub fn cairo_set_source(cr: *mut cairo_t, pattern: *mut cairo_pattern_t);
	pub fn cairo_set_source_rgb(cr: *mut cairo_t, r: c_double, g: c_double, b: c_double);
	pub fn cairo_set_source_rgba(cr: *mut cairo_t, r: c_double, g: c_double, b: c_double, a: c_double);
	pub fn cairo_set_source_surface(cr: *mut cairo_t, surface: *const cairo_surface_t, x: c_double, y: c_double);

	pub fn cairo_fill(cr: *mut cairo_t);
	pub fn cairo_stroke(cr: *mut cairo_t);

	pub fn cairo_move_to(cr: *mut cairo_t, x: c_double, y: c_double);
	pub fn cairo_line_to(cr: *mut cairo_t, x: c_double, y: c_double);
	pub fn cairo_set_line_width(cr: *mut cairo_t, w: c_double);

	pub fn cairo_clip(cr: *mut cairo_t);
	pub fn cairo_rectangle(cr: *mut cairo_t, x: c_double, y: c_double, w: c_double, h: c_double);

	pub fn cairo_image_surface_create_for_data(data: *const c_uchar, format: cairo_format_t, width: c_int, height: c_int, stride: c_int) -> *mut cairo_surface_t;
	pub fn cairo_surface_flush(surface: *mut cairo_surface_t);
	pub fn cairo_surface_destroy(surface: *mut cairo_surface_t);

	pub fn cairo_pattern_create_for_surface(surface: *mut cairo_surface_t) -> *mut cairo_pattern_t;
	pub fn cairo_pattern_set_matrix(pattern: *mut cairo_pattern_t, matrix: *const cairo_matrix_t);
	pub fn cairo_pattern_destroy(pattern: *mut cairo_pattern_t);

	pub fn cairo_set_operator(cr: *mut cairo_t, operator: cairo_operator_t);
}

#[cfg(all(unix, not(target_os = "macos")))]
pub mod platform {
	use super::*;

	pub use libc::c_int;
	pub use xcb::ffi::*;
	pub use xcb;

	extern "C" {
		pub fn cairo_xcb_surface_set_size(surface: *mut cairo_surface_t, width: c_int, height: c_int);
		pub fn cairo_xcb_surface_create(connection: *mut xcb_connection_t, drawable: xcb_drawable_t, visual: *const xcb_visualtype_t, width: c_int, height: c_int) -> *mut cairo_surface_t;
	}
}

#[cfg(target_os = "macos")]
pub mod platform {
	use super::*;

	pub use std::os::raw::c_void;
	pub use libc::c_uint;
	pub use core_graphics::base::CGFloat;

	extern "C" {
		pub fn cairo_quartz_surface_create(format: cairo_format_t, width: c_uint, height: c_uint) -> *mut cairo_surface_t;
		pub fn cairo_quartz_surface_create_for_cg_context(context: *mut c_void, width: c_uint, height: c_uint) -> *mut cairo_surface_t;
		pub fn cairo_quartz_surface_get_cg_context(surface: *const cairo_surface_t) -> *mut c_void;

		pub fn CGContextTranslateCTM(context: *mut c_void, tx: CGFloat, ty: CGFloat);
		pub fn CGContextScaleCTM(context: *mut c_void, sx: CGFloat, sy: CGFloat);
	}
}
