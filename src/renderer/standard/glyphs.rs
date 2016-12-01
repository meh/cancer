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

use std::rc::Rc;
use std::sync::Arc;

use lru::LruCache;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

use sys::pango;
use font::Font;
use style;

/// Computed glyph cache.
///
/// This is an LRU cache around clusters and their computed glyphs, a cluster
/// can be as small as a grapheme or as big as a full line of text.
#[derive(Debug)]
pub struct Glyphs {
	font:  Arc<Font>,
	inner: LruCache<Cluster, Rc<Computed>, BuildHasherDefault<FnvHasher>>,
}

/// A cluster with text and attributes.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Cluster {
	text:  Rc<String>,
	attrs: style::Attributes,
}

impl Cluster {
	/// Create a new cluster.
	pub fn new(text: Rc<String>, attrs: style::Attributes) -> Self {
		Cluster {
			text:  text,
			attrs: attrs & (style::BOLD | style::FAINT | style::ITALIC),
		}
	}
}

/// The computed glyphs.
#[derive(Debug)]
pub struct Computed {
	text:   Rc<String>,
	glyphs: pango::GlyphItem,
}

impl Computed {
	pub fn text(&self) -> &str {
		&self.text
	}

	pub fn glyphs(&self) -> &pango::GlyphItem {
		&self.glyphs
	}
}

impl Glyphs {
	/// Create a new cache.
	pub fn new(size: usize, font: Arc<Font>) -> Self {
		Glyphs {
			font: font,
			inner: LruCache::with_hasher(size, Default::default()),
		}
	}

	/// Get a computed glyph.
	pub fn compute(&mut self, string: Rc<String>, attrs: style::Attributes) -> Rc<Computed> {
		let cluster = Cluster::new(string.clone(), attrs);

		if let Some(computed) = self.inner.get_mut(&cluster) {
			return computed.clone();
		}

		let computed = Rc::new(Computed {
			text:   cluster.text.clone(),
			glyphs: self.font.shape(&*cluster.text, cluster.attrs),
		});

		self.inner.insert(cluster, computed.clone());
		computed
	}
}
