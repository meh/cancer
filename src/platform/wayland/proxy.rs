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
	device:  *mut cairo_device_t,
	window:  WlEglSurface,
	display: EGLDisplay,
	surface: EGLSurface,
	main:    EGLContext,
	cairo:   EGLContext,

	program:   GLuint,
	vertex:    GLuint,
	fragment:  GLuint,
	target:    GLuint,
	attribute: (GLuint, GLint),
	rect:      (GLuint, GLuint),
}

impl Drop for Inner {
	fn drop(&mut self) {
		unsafe {
			cairo_device_destroy(self.device);

			egl::make_current(self.display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, self.main);
			gl::DeleteProgram(self.program);
			gl::DeleteShader(self.vertex);
			gl::DeleteShader(self.fragment);
			gl::DeleteTextures(1, &self.target);

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

			let vertex = glut::compile_shader(gl::VERTEX_SHADER, r"
				#version 100

				precision lowp float;

				attribute vec2 position;
				varying vec2 v_texture;

				void main() {
					gl_Position = vec4(position, 0.0, 1.0);
					v_texture   = (position + vec2(1.0, 1.0)) / 2.0;
				}
			")?;

			let fragment = glut::compile_shader(gl::FRAGMENT_SHADER, r"
				#version 100

				precision lowp float;

				uniform sampler2D tex;
				varying vec2 v_texture;

				void main() {
					gl_FragColor = texture2D(tex, v_texture);
				}
			")?;

			const SQUARE: [f32; 8] = [
				-1.0,  1.0,
				 1.0,  1.0,
				-1.0, -1.0,
				 1.0, -1.0,
			];

			let program   = glut::link_program(vertex, fragment)?;
			let attribute = (
				gl::GetAttribLocation(program, b"position\0".as_ptr() as *const _) as GLuint,
				gl::GetUniformLocation(program, b"tex\0".as_ptr() as *const _));

			let mut target = 0;
			gl::GenTextures(1, &mut target);
			gl::BindTexture(gl::TEXTURE_2D, target);
			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, width as GLint, height as GLint,
				0, gl::BGRA, gl::UNSIGNED_BYTE, ptr::null_mut());

			let mut rect_array = 0;
			gl::GenVertexArrays(1, &mut rect_array);
			gl::BindVertexArray(rect_array);

			let mut rect_buffer = 0;
			gl::GenBuffers(1, &mut rect_buffer);
			gl::BindBuffer(gl::ARRAY_BUFFER, rect_buffer);
			gl::BufferData(gl::ARRAY_BUFFER, mem::size_of_val(&SQUARE) as GLsizeiptr, SQUARE.as_ptr() as *const _, gl::STATIC_DRAW);

			gl::EnableVertexAttribArray(attribute.0);
			gl::VertexAttribPointer(attribute.0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());

			gl::BindTexture(gl::TEXTURE_2D, 0);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
			gl::BindVertexArray(0);
			egl::make_current(display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, egl::EGL_NO_CONTEXT);

			let cairo = egl::create_context(display, config, main, &[])
				.ok_or(error::platform::Error::EGL("could not create context".into()))?;

			let device = cairo_egl_device_create(display, cairo);

			self.inner = Some(Inner {
				device:  ::std::ptr::null_mut(),
				window:  window,
				display: display,
				surface: surface,
				main:    main,
				cairo:   cairo,

				program:   program,
				vertex:    vertex,
				fragment:  fragment,
				target:    target,
				attribute: attribute,
				rect:      (rect_array, rect_buffer),
			});

			Ok(cairo::Surface::new(::std::ptr::null_mut(), target, width, height))
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

				gl::UseProgram(inner.program);
				gl::ActiveTexture(gl::TEXTURE0);
				gl::BindTexture(gl::TEXTURE_2D, inner.target);
				gl::BindVertexArray(inner.rect.0);
				gl::Uniform1i(inner.attribute.1, 0);

				gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

				gl::BindVertexArray(0);
				gl::BindTexture(gl::TEXTURE_2D, 0);
				gl::UseProgram(0);

				egl::swap_buffers(inner.display, inner.surface);
				egl::make_current(inner.display, egl::EGL_NO_SURFACE, egl::EGL_NO_SURFACE, egl::EGL_NO_CONTEXT);
			}
		}
	}
}
