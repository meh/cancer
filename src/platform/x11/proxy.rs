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
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::process::Command;
use std::cell::RefCell;

use sys::cairo;
use error;
use platform;
use platform::x11::Request;
use config::Config;

#[derive(Debug)]
pub struct Proxy {
	config:  Arc<Config>,
	sender:  Sender<Request>,
	surface: RefCell<Option<cairo::Surface>>,

	width:   Arc<AtomicU32>,
	height:  Arc<AtomicU32>,
	focus:   Arc<AtomicBool>,
	visible: Arc<AtomicBool>,
}

unsafe impl Send for Proxy { }

impl Proxy {
	pub fn new(config: Arc<Config>,
	           sender: Sender<Request>,
	           surface: cairo::Surface,
	           width: Arc<AtomicU32>,
	           height: Arc<AtomicU32>,
	           focus: Arc<AtomicBool>,
	           visible: Arc<AtomicBool>) -> Self {
		Proxy {
			config:  config,
			sender:  sender,
			surface: RefCell::new(Some(surface)),

			width:   width,
			height:  height,
			focus:   focus,
			visible: visible,
		}
	}
}

impl platform::Proxy for Proxy {
	fn width(&self) -> u32 {
		self.width.load(Ordering::Relaxed)
	}

	fn height(&self) -> u32 {
		self.height.load(Ordering::Relaxed)
	}

	fn has_focus(&self) -> bool {
		self.focus.load(Ordering::Relaxed)
	}

	fn is_visible(&self) -> bool {
		self.visible.load(Ordering::Relaxed)
	}

	fn surface(&self) -> cairo::Surface {
		self.surface.borrow_mut().take().unwrap()
	}

	fn resize(&mut self, width: u32, height: u32) {
		self.sender.send(Request::Resize(width, height)).unwrap();
	}

	fn set_title(&self, title: String) {
		self.sender.send(Request::Title(title)).unwrap();
	}

	fn copy(&self, name: String, value: String) {
		self.sender.send(Request::Copy(name, value)).unwrap();
	}

	fn paste(&self, name: String) {
		self.sender.send(Request::Paste(name)).unwrap();
	}

	fn urgent(&self) {
		self.sender.send(Request::Urgent).unwrap();
	}

	fn flush(&self) {
		self.sender.send(Request::Flush).unwrap();
	}

	fn open(&self, value: &str) -> error::Result<()> {
		Command::new(self.config.environment().hinter().opener().unwrap_or("xdg-open"))
			.arg(value).spawn()?;

		Ok(())
	}
}
