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
use std::sync::atomic::{AtomicU32, Ordering};
use std::process::Command;
use std::ptr;
use std::cell::RefCell;

use wayland_client::Proxy as Proxied;
use wayland_client::egl::WlEglSurface;
use wayland_client::protocol::wl_surface;
use egl;

use sys::cairo;
use error;
use config::Config;
use platform::{self, Clipboard};
use platform::wayland::Context;

pub struct Proxy {
	pub(super) context: Arc<Context>,
	pub(super) surface: Arc<wl_surface::WlSurface>,
	pub(super) window:  RefCell<Option<WlEglSurface>>,

	pub(super) width:  Arc<AtomicU32>,
	pub(super) height: Arc<AtomicU32>,
}

unsafe impl Send for Proxy { }

impl platform::Proxy for Proxy {
	fn dimensions(&self) -> (u32, u32) {
		(self.width.load(Ordering::Relaxed), self.height.load(Ordering::Relaxed))
	}

	fn surface(&self) -> error::Result<cairo::Surface> {
		let (width, height) = self.dimensions();
		self.window.borrow_mut().take();

		unsafe {
			let display = egl::get_display(self.context.display.ptr() as *mut _)
				.ok_or(error::platform::Error::EGL("could not get display".into()))?;

			let mut major = 0;
			let mut minor = 0;

			if !egl::initialize(display, &mut major, &mut minor) {
				return Err(error::platform::Error::EGL("initialization failed".into()).into());
			}

			let config = egl::choose_config(display, &[
				egl::EGL_RED_SIZE, 8,
				egl::EGL_GREEN_SIZE, 8,
				egl::EGL_BLUE_SIZE, 8,
				egl::EGL_NONE], 1)
					.ok_or(error::platform::Error::EGL("choose config failed".into()))?;

			let context = egl::create_context(display, config, ptr::null_mut(), &[])
				.ok_or(error::platform::Error::EGL("could not create context".into()))?;

			let window = WlEglSurface::new(&self.surface, width as i32, height as i32);

			let surface = egl::create_window_surface(display, config, window.ptr() as *mut _, &[])
				.ok_or(error::platform::Error::EGL("could not create surface".into()))?;

			*self.window.borrow_mut() = Some(window);

			Ok(cairo::Surface::new(display, context, surface, width, height))
		}
	}
}
