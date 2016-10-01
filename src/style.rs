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

use picto::color::{Rgb, Rgba};

#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub struct Style {
	pub foreground: Option<Rgba<f64>>,
	pub background: Option<Rgba<f64>>,
	pub underline:  bool,
	pub bold:       bool,
	pub blink:      bool,
}

impl Style {
	pub fn foreground(&self) -> Option<&Rgba<f64>> {
		self.foreground.as_ref()
	}

	pub fn background(&self) -> Option<&Rgba<f64>> {
		self.background.as_ref()
	}
}
