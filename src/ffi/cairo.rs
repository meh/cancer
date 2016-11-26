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
use libc::{c_double};

#[cfg(target_os = "linux")]
use libc::c_int;

#[cfg(target_os = "macos")]
use libc::c_uint;
#[cfg(target_os = "macos")]
use core_graphics::base::CGFloat;

#[repr(C)]
pub struct cairo_t(c_void);

#[repr(C)]
pub struct cairo_surface_t(c_void);

#[repr(C)]
pub enum cairo_format_t {
	Invalid = -1,
	Argb32,
	Rgb24,
	A8,
	A1,
	Rgb16_565,
	Rgb30,
}

extern "C" {
	pub fn cairo_create(surface: *mut cairo_surface_t) -> *mut cairo_t;
	pub fn cairo_destroy(cr: *mut cairo_t);

	pub fn cairo_save(cr: *mut cairo_t);
	pub fn cairo_restore(cr: *mut cairo_t);

	pub fn cairo_push_group(cr: *mut cairo_t);
	pub fn cairo_pop_group_to_source(cr: *mut cairo_t);

	pub fn cairo_paint(cr: *mut cairo_t);
	pub fn cairo_set_source_rgb(cr: *mut cairo_t, r: c_double, g: c_double, b: c_double);
	pub fn cairo_set_source_rgba(cr: *mut cairo_t, r: c_double, g: c_double, b: c_double, a: c_double);

	pub fn cairo_fill(cr: *mut cairo_t);
	pub fn cairo_fill_preserve(cr: *mut cairo_t);
	pub fn cairo_stroke(cr: *mut cairo_t);

	pub fn cairo_move_to(cr: *mut cairo_t, x: c_double, y: c_double);
	pub fn cairo_line_to(cr: *mut cairo_t, x: c_double, y: c_double);
	pub fn cairo_set_line_width(cr: *mut cairo_t, w: c_double);

	pub fn cairo_clip(cr: *mut cairo_t);
	pub fn cairo_rectangle(cr: *mut cairo_t, x: c_double, y: c_double, w: c_double, h: c_double);

	pub fn cairo_surface_flush(surface: *mut cairo_surface_t);
	pub fn cairo_surface_destroy(surface: *mut cairo_surface_t);
}

#[cfg(target_os = "linux")]
use xcb::ffi::*;

#[cfg(target_os = "linux")]
extern "C" {
	pub fn cairo_xcb_surface_set_size(surface: *mut cairo_surface_t, width: c_int, height: c_int);
	pub fn cairo_xcb_surface_create(connection: *mut xcb_connection_t, drawable: xcb_drawable_t, visual: *const xcb_visualtype_t, width: c_int, height: c_int) -> *mut cairo_surface_t;
}

#[cfg(target_os = "macos")]
extern "C" {
	pub fn cairo_quartz_surface_create(format: cairo_format_t, width: c_uint, height: c_uint) -> *mut cairo_surface_t;
	pub fn cairo_quartz_surface_create_for_cg_context(context: *mut c_void, width: c_uint, height: c_uint) -> *mut cairo_surface_t;
	pub fn cairo_quartz_surface_get_cg_context(surface: *const cairo_surface_t) -> *mut c_void;

	pub fn CGContextTranslateCTM(context: *mut c_void, tx: CGFloat, ty: CGFloat);
	pub fn CGContextScaleCTM(context: *mut c_void, sx: CGFloat, sy: CGFloat);
}
