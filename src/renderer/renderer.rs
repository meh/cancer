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

use util::Region;
use sys::cairo;
use config::Config;
use font::Font;
use renderer::{State, Margin};
use renderer::{option, Options};
use renderer::{standard, ligatures};
use interface::Interface;

pub struct Renderer {
	state: State,
	mode:  Mode,
}

pub enum Mode {
	Standard(standard::Renderer),
	Ligatures(ligatures::Renderer),
}

impl Renderer {
	/// Get the window dimensions for the given grid.
	pub fn dimensions(columns: u32, rows: u32, config: &Config, font: &Font) -> (u32, u32) {
		let margin  = config.style().margin();
		let spacing = config.style().spacing();

		let width  = (columns * font.width()) + (margin * 2);
		let height = (rows * (font.height() + spacing)) + (margin * 2);

		(width, height)
	}

	pub fn new(config: Arc<Config>, font: Arc<Font>, surface: &cairo::Surface, width: u32, height: u32) -> Self {
		let spacing = config.style().spacing();

		let margin = Margin {
			horizontal: config.style().margin() +
				((width - (config.style().margin() * 2)) % font.width()) / 2,

			vertical: config.style().margin() +
				((height - (config.style().margin() * 2)) % (font.height() + spacing)) / 2,
		};

		let state = State {
			config: config,
			font:   font,

			width:  width,
			height: height,

			spacing: spacing,
			margin:  margin,
		};

		let mode = if state.config().style().ligatures() {
			Mode::Ligatures(ligatures::Renderer::new(surface, &state))
		}
		else {
			Mode::Standard(standard::Renderer::new(surface, &state))
		};

		Renderer {
			state: state,
			mode:  mode,
		}
	}

	/// Resize the renderer viewport.
	pub fn resize(&mut self, surface: &cairo::Surface, width: u32, height: u32) {
		let (mode, state) = (&mut self.mode, &mut self.state);

		state.resize(width, height);

		match *mode {
			Mode::Standard(ref mut renderer) =>
				renderer.resize(surface, state),

			Mode::Ligatures(ref mut renderer) =>
				renderer.resize(surface, state),
		}
	}

	/// Render the given changes.
	pub fn render<I>(&mut self, mut options: Options, region: Option<Region>, interface: &Interface, iter: I)
		where I: Iterator<Item = (u32, u32)>
	{
		let (mode, state) = (&mut self.mode, &self.state);

		if region.is_some() {
			options.insert(option::DAMAGE);
		}
		else {
			options.remove(option::DAMAGE);
		}

		match *mode {
			Mode::Standard(ref mut renderer) =>
				renderer.render(state, options, region, interface, iter),

			Mode::Ligatures(ref mut renderer) =>
				renderer.render(state, options, region, interface, iter),
		}
	}
}

impl Deref for Renderer {
	type Target = State;

	fn deref(&self) -> &Self::Target {
		&self.state
	}
}
