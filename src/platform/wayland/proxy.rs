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
use std::mem;

use wayland_client::Proxy as WlProxy;
use wayland_client::egl::WlEglSurface;
use wayland_client::protocol::{wl_surface, wl_display};

use egl::{self, EGLDisplay, EGLSurface, EGLContext};
use gl;
use gl::types::*;

use sys::cairo;
use ffi::cairo::*;
use ffi::cairo::platform::*;
use error;
use config::Config;
use platform::{self, Clipboard};
use platform::glut;

pub struct Proxy {
	pub(super) display: Arc<wl_display::WlDisplay>,
	pub(super) surface: Arc<wl_surface::WlSurface>,
	pub(super) inner:   Option<Inner>,

	pub(super) width:  Arc<AtomicU32>,
	pub(super) height: Arc<AtomicU32>,
}

pub struct Inner {
	device:   *mut cairo_device_t,
	window:   WlEglSurface,
	display:  EGLDisplay,
	surface:  EGLSurface,
	main:     EGLContext,
	cairo:    EGLContext,
	renderer: glut::Renderer,
}

impl Drop for Inner {
	fn drop(&mut self) {
		unsafe {
			egl::make_current(self.display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, self.cairo);
			cairo_device_destroy(self.device);

			egl::make_current(self.display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, self.main);
			self.renderer.release();

			egl::destroy_context(self.display, self.cairo);
			egl::destroy_context(self.display, self.main);
			egl::destroy_surface(self.display, self.surface);
		}
	}
}

unsafe impl Send for Proxy { }

impl platform::Proxy for Proxy {
	fn dimensions(&self) -> (u32, u32) {
		(self.width.load(Ordering::Relaxed), self.height.load(Ordering::Relaxed))
	}

	fn surface(&mut self) -> error::Result<cairo::Surface> {
		let (width, height) = self.dimensions();
		self.inner.take();

		unsafe {
			let display = egl::get_display(self.display.ptr() as *mut _)
				.ok_or(error::platform::Error::EGL("could not get display".into()))?;

			let mut major = 0;
			let mut minor = 0;

			if !egl::initialize(display, &mut major, &mut minor) {
				return Err(error::platform::Error::EGL("initialization failed".into()).into());
			}

			egl::bind_api(egl::EGL_OPENGL_API);
			gl::load_with(|s| egl::get_proc_address(s) as *const _);

			let config = egl::choose_config(display, &[
				egl::EGL_SURFACE_TYPE, egl::EGL_WINDOW_BIT,
				egl::EGL_RENDERABLE_TYPE, egl::EGL_OPENGL_BIT,
				egl::EGL_CONFORMANT, egl::EGL_OPENGL_BIT,
				egl::EGL_COLOR_BUFFER_TYPE, egl::EGL_RGB_BUFFER,
				egl::EGL_RED_SIZE, 1,
				egl::EGL_GREEN_SIZE, 1,
				egl::EGL_BLUE_SIZE, 1,
				egl::EGL_ALPHA_SIZE, 1,
				egl::EGL_NONE], 1)
					.ok_or(error::platform::Error::EGL("choose config failed".into()))?;

			let main = egl::create_context(display, config, egl::EGL_NO_CONTEXT, &[])
				.ok_or(error::platform::Error::EGL("could not create context".into()))?;

			egl::make_current(display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, main);

			let window = WlEglSurface::new(&self.surface, width as i32, height as i32);

			let surface = egl::create_window_surface(display, config, window.ptr() as *mut _, &[])
				.ok_or(error::platform::Error::EGL("could not create surface".into()))?;

			egl::make_current(display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, egl::EGL_NO_CONTEXT);

			let renderer = glut::Renderer::new(width, height)?;
			let texture  = renderer.texture();

			let cairo = egl::create_context(display, config, main, &[])
				.ok_or(error::platform::Error::EGL("could not create context".into()))?;

			let device = cairo_egl_device_create(display, cairo);

			self.inner = Some(Inner {
				device:   device,
				window:   window,
				display:  display,
				surface:  surface,
				main:     main,
				cairo:    cairo,
				renderer: renderer,
			});

			Ok(cairo::Surface::new(device, texture, width, height))
		}
	}

	fn render<F: FnOnce()>(&mut self, surface: &mut cairo::Surface, f: F) {
		if let Some(inner) = self.inner.as_mut() {
			unsafe {
				egl::make_current(inner.display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, inner.cairo);
				f();
				surface.flush();
				cairo_gl_surface_swapbuffers(surface.0);
				cairo_device_flush(inner.device);

				egl::make_current(inner.display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, inner.main);
				inner.renderer.render();

				egl::swap_buffers(inner.display, inner.surface);
				egl::make_current(inner.display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, egl::EGL_NO_CONTEXT);
			}
		}
	}
}
