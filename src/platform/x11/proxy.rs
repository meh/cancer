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
use std::sync::mpsc::Sender;
use std::process::Command;

use xcb;
use xcbu::ewmh;

use sys::cairo;
use error;
use platform::{self, Clipboard};
use platform::x11::Request;

pub struct Proxy {
	pub(super) request:    Sender<Request>,
	pub(super) connection: Arc<ewmh::Connection>,
	pub(super) window:     xcb::Window,
	pub(super) screen:     i32,
}

unsafe impl Send for Proxy { }

impl platform::Proxy for Proxy {
	fn dimensions(&self) -> (u32, u32) {
		let reply = xcb::get_geometry(&self.connection, self.window).get_reply().unwrap();
		(reply.width() as u32, reply.height() as u32)
	}

	fn surface(&mut self) -> error::Result<cairo::Surface> {
		let screen          = self.connection.get_setup().roots().nth(self.screen as usize).unwrap();
		let (width, height) = self.dimensions();

		for item in screen.allowed_depths() {
			if item.depth() == 24 {
				for visual in item.visuals() {
					return Ok(cairo::Surface::new(&self.connection, self.window, visual, width, height));
				}
			}
		}

		Err(error::platform::Error::MissingDepth(24).into())
	}

	fn resize(&mut self, width: u32, height: u32) {
		self.request.send(Request::Resize(width, height)).unwrap();
	}

	fn set_title(&self, title: String) {
		self.request.send(Request::Title(title)).unwrap();
	}

	fn copy(&self, name: Clipboard, value: String) {
		self.request.send(Request::Copy(name, value)).unwrap();
	}

	fn paste(&self, name: Clipboard) {
		self.request.send(Request::Paste(name)).unwrap();
	}

	fn urgent(&self) {
		self.request.send(Request::Urgent).unwrap();
	}

	fn render<F: FnOnce()>(&mut self, surface: &mut cairo::Surface, f: F) {
		f();
		surface.flush();
		self.request.send(Request::Flush).unwrap();
	}

	fn open(&self, through: Option<&str>, value: &str) -> error::Result<()> {
		Command::new(through.unwrap_or("xdg-open")).arg(value).spawn()?;

		Ok(())
	}
}
