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

use std::mem;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use winapi::winuser;
use winapi::windef::{HWND};
use user32;

use error;
use config::Config;
use font::Font;
use platform::Event;
use platform::windows::Proxy;

pub struct Window {
	config: Arc<Config>,
	proxy:  Option<Proxy>,
}

impl Window {
	pub fn new(name: Option<&str>, config: Arc<Config>, font: Arc<Font>) -> error::Result<Self> {
		let margin  = config.style().margin();
		let spacing = config.style().spacing();

		let width  = (80 * font.width()) + (margin * 2);
		let height = (24 * (font.height() + spacing)) + (margin * 2);

		let proxy = Proxy {
			config: config.clone(),
		};

		Ok(Window {
			config: config,
			proxy:  Some(proxy),
		})
	}

	pub fn proxy(&mut self) -> Proxy {
		self.proxy.take().unwrap()
	}

	pub fn run(&mut self, manager: Sender<Event>) -> error::Result<()> {
		unsafe {
			let mut msg = mem::zeroed();

			loop {
				let pm = user32::PeekMessageW(&mut msg, 0 as HWND, 0, 0, winuser::PM_REMOVE);
				if pm == 0 {
					continue;
				}

				match msg.message {
					winuser::WM_QUIT => {
						try!(manager.send(Event::Closed));
					}

					value => {
						debug!(target: "cancer::platform", "unhandled event: {:?}", value);
					}
				}

				user32::TranslateMessage(&mut msg);
				user32::DispatchMessageW(&mut msg);
			}
		}
	}
}
