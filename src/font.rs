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

use std::sync::Arc;
use std::ops::Deref;
use std::ptr;

use libc::c_int;
use ffi::pango::*;
use sys::glib;
use sys::pango;
use config::Config;
use style;
use error;

/// The font to use for rendering.
#[derive(Debug)]
pub struct Font {
	map:     pango::Map,
	context: pango::Context,
	set:     pango::Set,
	metrics: pango::Metrics,
}

impl Font {
	/// Load the font from the given configuration.
	pub fn load(config: Arc<Config>) -> error::Result<Self> {
		let map     = pango::Map::new();
		let context = pango::Context::new(&map);
		let set     = context.fonts(&pango::Description::from(config.style().font())).ok_or(error::Error::Message("missing font".into()))?;
		let metrics = set.metrics();

		Ok(Font {
			map:     map,
			context: context,
			set:     set,
			metrics: metrics,
		})
	}

	/// Load the proper font for the given attributes.
	pub fn font(&self, ch: char, style: style::Attributes) -> pango::Font {
		let font = self.set.font(ch);

		if !style.intersects(style::BOLD | style::FAINT | style::ITALIC) {
			return font;
		}

		let mut desc = font.description();

		if style.contains(style::BOLD) {
			desc.weight(pango::Weight::Bold);
		}
		else if style.contains(style::FAINT) {
			desc.weight(pango::Weight::Light);
		}
		else {
			desc.weight(pango::Weight::Normal);
		}

		if style.contains(style::ITALIC) {
			desc.style(pango::Style::Italic);
		}

		self.context.font(&desc).unwrap_or(font)
	}

	/// Shape the string.
	pub fn shape<T: AsRef<str>>(&self, text: T, style: style::Attributes) -> pango::GlyphString {
		let text = text.as_ref();

		unsafe {
			let result = pango::GlyphString::new();
			let list   = glib::List(pango_itemize(self.context.0, text.as_ptr() as *const _, 0, text.len() as c_int, ptr::null(), ptr::null()));
			let item   = pango::Item((*list.0).data as *mut _);

			pango_shape(text.as_ptr() as *const _, text.len() as c_int, &(*item.0).analysis, result.0);

			result
		}
	}
}

impl AsRef<pango::Context> for Font {
	fn as_ref(&self) -> &pango::Context {
		&self.context
	}
}

impl Deref for Font {
	type Target = pango::Metrics;

	fn deref(&self) -> &Self::Target {
		&self.metrics
	}
}
