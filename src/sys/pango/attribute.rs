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
use ffi::pango::*;
use super::{Weight, Style, Underline};
use picto::color::Rgb;

#[derive(Debug)]
pub struct Attribute(pub *mut PangoAttribute);

impl Attribute {
	pub fn weight(weight: Weight) -> Self {
		unsafe {
			Attribute(pango_attr_weight_new(weight))
		}
	}

	pub fn style(style: Style) -> Self {
		unsafe {
			Attribute(pango_attr_style_new(style))
		}
	}

	pub fn underline(style: Underline) -> Self {
		unsafe {
			Attribute(pango_attr_underline_new(style))
		}
	}

	pub fn underline_color(color: &Rgb<f64>) -> Self {
		unsafe {
			Attribute(pango_attr_underline_color_new(
				(color.red * u16::max_value() as f64) as u16,
				(color.green * u16::max_value() as f64) as u16,
				(color.blue * u16::max_value() as f64) as u16))
		}
	}

	pub fn strikethrough(value: bool) -> Self {
		unsafe {
			Attribute(pango_attr_strikethrough_new(value))
		}
	}

	pub fn strikethrough_color(color: &Rgb<f64>) -> Self {
		unsafe {
			Attribute(pango_attr_strikethrough_color_new(
				(color.red * u16::max_value() as f64) as u16,
				(color.green * u16::max_value() as f64) as u16,
				(color.blue * u16::max_value() as f64) as u16))
		}
	}

	pub fn take(mut self) -> *mut PangoAttribute {
		let ptr = self.0;
		self.0 = ptr::null_mut();
		ptr
	}
}

impl Clone for Attribute {
	fn clone(&self) -> Self {
		unsafe {
			Attribute(pango_attribute_copy(self.0))
		}
	}
}

impl Drop for Attribute {
	fn drop(&mut self) {
		unsafe {
			if let Some(ptr) = self.0.as_mut() {
				pango_attribute_destroy(ptr);
			}
		}
	}
}
