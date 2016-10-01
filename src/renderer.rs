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
use palette::Rgb;

use config::Config;
use sys::cairo::{Surface, Context};
use sys::pango::{Layout};
use font::Font;

pub struct Renderer {
	context: Context,
	layout:  Layout,
	font:    Font,
}

impl Renderer {
	pub fn new<S: AsRef<Surface>>(config: Arc<Config>, font: Font, surface: S) -> Self {
		let context = Context::new(surface);
		let layout  = Layout::new(&font);

		Renderer {
			context: context,
			layout:  layout,
			font:    font,
		}
	}

	pub fn draw(&mut self) {
		let (c, l, f) = (&mut self.context, &mut self.layout, &mut self.font);

		l.update(&c);

		c.group(|mut c| {
			c.rgb(Rgb::new(1.0, 1.0, 1.0));
			c.paint();

			c.rgb(Rgb::new(1.0, 0.0, 0.0));
			c.move_to(10.0, 10.0);
			c.text(l, "█wanna sign a contract? █／人◕ ‿‿ ◕人＼█");

			c.rgb(Rgb::new(0.0, 0.5, 0.0));
			c.move_to(10.0, 22.0);
			c.text(l, format!("{}x{} MOTHERFUCKER", f.width(), f.height()));
		});
	}
}
