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

use picto::Region;
use sys::cairo;
use interface::Interface;
use renderer::State;
use renderer::option::{self, Options};

/// Renderer for a `cairo::Surface`.
pub struct Renderer {
	context: cairo::Context,
}

unsafe impl Send for Renderer { }

impl Renderer {
	/// Create a new renderer for the given settings and surface.
	pub fn new(surface: &cairo::Surface, state: &State) -> Self {
		let context = cairo::Context::new(surface);

		Renderer {
			context: context,
		}
	}

	/// Resize the renderer viewport.
	pub fn resize(&mut self, surface: &cairo::Surface, state: &State) {
		self.context = cairo::Context::new(surface);
	}

	pub fn render<I>(&mut self, state: &State, options: Options, region: Option<Region>, interface: &Interface, iter: I)
		where I: Iterator<Item = (u32, u32)>
	{
		self.context.rgba(&::picto::color::Rgba::new_u8(0, 0, 0, 255));
		self.context.paint();
	}
}
