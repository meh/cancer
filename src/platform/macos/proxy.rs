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
use std::process::Command;
use std::cell::RefCell;
use std::ffi::CStr;
use std::str;
use std::sync::mpsc::Sender;

use cocoa::foundation::{NSString, NSSize, NSArray};
use cocoa::appkit::{NSWindow, NSView, NSSound, NSPasteboard, NSPasteboardTypeString};
use cocoa::base::{class, nil};

use sys::cairo;
use error;
use platform::{self, Clipboard, Event};
use platform::macos::IdRef;
use config::Config;

#[derive(Debug)]
pub struct Proxy {
	pub(super) config:  Arc<Config>,
	pub(super) manager: Option<Sender<Event>>,

	pub(super) window:  IdRef,
	pub(super) view:    IdRef,
	pub(super) context: RefCell<IdRef>,
}

unsafe impl Send for Proxy { }

impl platform::Proxy for Proxy {
	fn dimensions(&self) -> (u32, u32) {
		unsafe {
			let rect   = NSView::frame(*self.view);
			let factor = NSWindow::backingScaleFactor(*self.window) as f32;
			let width  = factor * rect.size.width as f32;
			let height = factor * rect.size.height as f32;

			(width as u32, height as u32)
		}
	}

	fn surface(&self) -> error::Result<cairo::Surface> {
		unsafe {
			let (width, height) = self.dimensions();

			*self.context.borrow_mut() = IdRef::new(msg_send![class("NSGraphicsContext"),
				graphicsContextWithWindow:*self.window]);

			Ok(cairo::Surface::new(msg_send![**self.context.borrow(), CGContext], width, height))
		}
	}

	fn prepare(&mut self, manager: Sender<Event>) {
		self.manager = Some(manager);
	}

	fn resize(&mut self, width: u32, height: u32) {
		unsafe {
			self.window.setContentSize_(NSSize::new(width as f64, height as f64));
		}
	}

	fn set_title(&self, title: String) {
		unsafe {
			self.window.setTitle_(*IdRef::new(NSString::alloc(nil).init_str(&title)))
		}
	}

	fn copy(&self, _name: Clipboard, value: String) {
		unsafe {
			let paste = NSPasteboard::generalPasteboard(nil);
			paste.clearContents();
			paste.writeObjects(NSArray::arrayWithObjects(nil, &[NSString::alloc(nil).init_str(&value)]));
		}
	}

	fn paste(&self, _name: Clipboard) {
		unsafe {
			if let Some(manager) = self.manager.as_ref() {
				let paste = NSPasteboard::generalPasteboard(nil);
				let value = paste.stringForType(NSPasteboardTypeString);

				if value != nil {
					let string = value.UTF8String();
					let string = CStr::from_ptr(string);
					let string = str::from_utf8_unchecked(string.to_bytes());
					let _      = manager.send(Event::Paste(string.into()));
				}
			}
		}
	}

	fn urgent(&self) {
		unsafe {
			if let Some(sound) = self.config.environment().bell() {
				NSSound::soundNamed_(nil, NSString::alloc(nil).init_str(sound)).play();
			}
		}
	}

	fn render<F: FnOnce()>(&mut self, surface: &mut cairo::Surface, f: F) {
		f(); surface.flush();

		unsafe {
			msg_send![**self.context.borrow(), flushGraphics];
		}
	}

	fn open(&self, through: Option<&str>, value: &str) -> error::Result<()> {
		Command::new(through.unwrap_or("open")).arg(value).spawn()?;

		Ok(())
	}
}
