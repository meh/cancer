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

use sys::pango;
use config::Config;
use error;

/// The font to use for rendering.
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
		let set     = context.font(config.style().font()).ok_or(error::Error::Message("missing font".into()))?;
		let metrics = set.metrics();

		Ok(Font {
			map:     map,
			context: context,
			set:     set,
			metrics: metrics,
		})
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
