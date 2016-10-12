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

use libc::{c_void, c_char, c_int, c_uint};
use super::cairo::*;
use super::glib::*;

#[repr(C)]
pub struct PangoItem {
	pub offset:    c_int,
	pub length:    c_int,
	pub num_chars: c_int,
	pub analysis:  PangoAnalysis,
}

#[repr(C)]
pub struct PangoContext(c_void);

#[repr(C)]
pub struct PangoLanguage(c_void);

#[repr(C)]
pub struct PangoAnalysis {
	pub shape_engine: *mut c_void,
	pub lang_engine:  *mut c_void,
	pub font:         *mut PangoFont,

	pub level:   u8,
	pub gravity: u8,
	pub flags:   u8,

	pub script: u8,
	pub language: *mut c_void,

	pub extra_attrs: *mut GList,
}

#[repr(C)]
pub struct PangoGlyphString(c_void);

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
pub struct PangoFont(c_void);

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
	pub fn pango_context_load_font(context: *mut PangoContext, desc: *const PangoFontDescription) -> *mut PangoFont;
	pub fn pango_fontset_get_font(fontset: *mut PangoFontset, ch: c_uint) -> *mut PangoFont;
	pub fn pango_fontset_get_metrics(fontset: *mut PangoFontset) -> *mut PangoFontMetrics;

	pub fn pango_font_metrics_unref(metrics: *mut PangoFontMetrics);
	pub fn pango_font_metrics_get_ascent(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_descent(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_approximate_digit_width(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_underline_thickness(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_underline_position(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_strikethrough_thickness(metrics: *mut PangoFontMetrics) -> c_int;
	pub fn pango_font_metrics_get_strikethrough_position(metrics: *mut PangoFontMetrics) -> c_int;

	pub fn pango_font_description_from_string(string: *const c_char) -> *mut PangoFontDescription;
	pub fn pango_font_description_free(desc: *mut PangoFontDescription);
	pub fn pango_font_description_set_weight(desc: *mut PangoFontDescription, weight: PangoWeight);
	pub fn pango_font_description_set_style(desc: *mut PangoFontDescription, style: PangoStyle);

	pub fn pango_font_describe(font: *mut PangoFont) -> *mut PangoFontDescription;

	pub fn pango_glyph_string_new() -> *mut PangoGlyphString;
	pub fn pango_glyph_string_copy(glyphs: *mut PangoGlyphString) -> *mut PangoGlyphString;
	pub fn pango_glyph_string_free(glyphs: *mut PangoGlyphString);

	pub fn pango_itemize(context: *mut PangoContext, text: *const c_char, start_index: c_int, length: c_int, attrs: *const c_void, cached_iter: *const c_void) -> *mut GList;
	pub fn pango_shape(text: *const c_char, length: c_int, analysis: *const PangoAnalysis, glyphs: *mut PangoGlyphString);
	pub fn pango_item_free(item: *mut PangoItem);
}

#[link(name = "pangocairo-1.0")]
extern "C" {
	pub fn pango_cairo_font_map_new() -> *mut PangoFontMap;
	pub fn pango_cairo_show_glyph_string(cr: *mut cairo_t, font: *mut PangoFont, glyphs: *mut PangoGlyphString);
	pub fn pango_cairo_glyph_string_path(cr: *mut cairo_t, font: *mut PangoFont, glyphs: *mut PangoGlyphString);
}
