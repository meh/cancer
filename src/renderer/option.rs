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

bitflags! {
	pub struct Options: u8 {
		const DAMAGE   = 1 << 0;
		const BLINKING = 1 << 1;
		const FOCUS    = 1 << 2;
		const REVERSE  = 1 << 3;
		const CURSOR   = 1 << 4;
	}
}

impl Default for Options {
	fn default() -> Self {
		Options::empty()
	}
}

impl Options {
	pub fn damage(&self) -> bool {
		self.contains(DAMAGE)
	}

	pub fn blinking(&self) -> bool {
		self.contains(BLINKING)
	}

	pub fn focus(&self) -> bool {
		self.contains(FOCUS)
	}

	pub fn reverse(&self) -> bool {
		self.contains(REVERSE)
	}

	pub fn cursor(&self) -> bool {
		self.contains(CURSOR)
	}
}
