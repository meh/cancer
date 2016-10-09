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

use libc::{c_void, c_char, c_int};
use super::cairo::*;

#[repr(C)]
pub struct PangoContext(c_void);

#[repr(C)]
pub struct PangoLayout(c_void);

#[repr(C)]
pub struct PangoLanguage(c_void);

#[repr(C)]
pub struct PangoAttribute(c_void);

#[repr(C)]
pub struct PangoAttrList(c_void);

#[repr(C)]
pub enum PangoWeight {
	Thin       = 100,
	UltraLight = 200,
	Light      = 300,
	SemiLight  = 350,
	Book       = 380,
	Normal     = 400,
	Medium     = 500,
	SemiBold   = 600,
	Bold       = 700,
	UltraBold  = 800,
	Heavy      = 900,
	UltraHeavy = 1000,
}

#[repr(C)]
pub enum PangoStyle {
	Normal,
	Oblique,
	Italic,
}

#[repr(C)]
pub enum PangoUnderline {
	None,
	Single,
	Double,
	Low,
	Error,
}

#[repr(C)]
pub struct PangoFontMap(c_void);

#[repr(C)]
pub struct PangoFontset(c_void);

#[repr(C)]
pub struct PangoFontDescription(c_void);

#[repr(C)]
pub struct PangoFontMetrics(c_void);

#[link(name = "pango-1.0")]
extern "C" {
	pub fn pango_font_map_create_context(fontmap: *mut PangoFontMap) -> *mut PangoContext;
	pub fn pango_context_set_font_description(context: *mut PangoContext, desc: *const PangoFontDescription);
	pub fn pango_context_load_fontset(context: *mut PangoContext, desc: *const PangoFontDescription, language: *mut PangoLanguage) -> *mut PangoFontset;
	pub fn pango_fontset_get_metrics(fontset: *mut PangoFontset) -> *mut PangoFontMetrics;

	pub fn pango_font_metrics_unref(metrics: *mut PangoFontMetrics);
	pub fn pango_font_metrics_get_ascent(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_descent(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_approximate_digit_width(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_underline_thickness(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_underline_position(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_strikethrough_thickness(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_strikethrough_position(metrics: *mut PangoFontMetrics) -> c_int;

	pub fn pango_layout_new(ctx: *mut PangoContext) -> *mut PangoLayout;

	pub fn pango_font_description_from_string(string: *const c_char) -> *mut PangoFontDescription;
	pub fn pango_font_description_free(desc: *mut PangoFontDescription);

	pub fn pango_layout_set_font_description(layout: *mut PangoLayout, description: *mut PangoFontDescription);
	pub fn pango_layout_set_text(layout: *mut PangoLayout, text: *const c_char, length: c_int);
	pub fn pango_layout_set_attributes(layout: *mut PangoLayout, list: *mut PangoAttrList);

	pub fn pango_attr_list_new() -> *mut PangoAttrList;
	pub fn pango_attr_list_unref(list: *mut PangoAttrList);
	pub fn pango_attr_list_insert(list: *mut PangoAttrList, attr: *mut PangoAttribute);
	pub fn pango_attr_list_change(list: *mut PangoAttrList, attr: *mut PangoAttribute);

	pub fn pango_attribute_copy(attr: *const PangoAttribute) -> *mut PangoAttribute;
	pub fn pango_attribute_destroy(attr: *mut PangoAttribute);

	pub fn pango_attr_weight_new(weight: PangoWeight) -> *mut PangoAttribute;
	pub fn pango_attr_style_new(style: PangoStyle) -> *mut PangoAttribute;
	pub fn pango_attr_strikethrough_new(strike: bool) -> *mut PangoAttribute;
	pub fn pango_attr_strikethrough_color_new(red: u16, green: u16, blue: u16) -> *mut PangoAttribute;
	pub fn pango_attr_underline_new(underline: PangoUnderline) -> *mut PangoAttribute;
	pub fn pango_attr_underline_color_new(red: u16, green: u16, blue: u16) -> *mut PangoAttribute;
}

#[link(name = "pangocairo-1.0")]
extern "C" {
	pub fn pango_cairo_font_map_new() -> *mut PangoFontMap;

	pub fn pango_cairo_create_layout(cr: *mut cairo_t) -> *mut PangoLayout;
	pub fn pango_cairo_update_layout(cr: *mut cairo_t, layout: *mut PangoLayout);
	pub fn pango_cairo_show_layout(cr: *mut cairo_t, layout: *mut PangoLayout);
}
