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

use std::io::Write;
use std::ops::{Deref, DerefMut};

use error;
use platform::{Key, key};
use terminal::Terminal;
use terminal::touched::{self, Touched};

#[derive(Debug)]
pub struct Overlay {
	inner:   Terminal,
	touched: Touched,
	cache:   Vec<u8>,
}

impl Overlay {
	pub fn new(inner: Terminal) -> Self {
		Overlay {
			inner:   inner,
			touched: Touched::default(),
			cache:   Vec::new(),
		}
	}

	pub fn into_inner<W: Write>(mut self, output: W) -> error::Result<Terminal> {
		try!(self.inner.handle(self.cache, output));
		Ok(self.inner)
	}

	pub fn key(&mut self, key: Key) -> touched::Iter {
		self.touched.iter(self.inner.region())
	}

	pub fn handle<I: AsRef<[u8]>>(&mut self, input: I) {
		self.cache.extend(input.as_ref());
	}
}

impl Deref for Overlay {
	type Target = Terminal;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for Overlay {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}
