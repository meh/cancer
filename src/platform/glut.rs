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

use std::ptr;
use std::mem;
use std::ffi::{CStr, CString};

use gl;
use gl::types::*;

use error;

fn compile_shader(kind: GLenum, source: &str) -> error::Result<GLuint> {
	unsafe {
		let shader = gl::CreateShader(kind);
		let source = CString::new(source.as_bytes()).unwrap();
		gl::ShaderSource(shader, 1, &source.as_ptr(), ptr::null());
		gl::CompileShader(shader);

		let mut status = gl::FALSE as GLint;
		gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

		if status != gl::TRUE as GLint {
			let mut length = 0;
			gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut length);

			let mut buffer = Vec::with_capacity(length as usize);
			gl::GetShaderInfoLog(shader, length, &mut length, buffer.as_mut_ptr() as *mut GLchar);

			Err(CStr::from_bytes_with_nul(&buffer)
				.map(|s| s.to_string_lossy().into_owned())
				.unwrap_or("unknown error".to_owned()).into())
		}
		else {
			Ok(shader)
		}
	}
}

fn link_program(vertex: GLuint, fragment: GLuint) -> error::Result<GLuint> {
	unsafe {
		let program = gl::CreateProgram();
		gl::AttachShader(program, vertex);
		gl::AttachShader(program, fragment);
		gl::LinkProgram(program);

		let mut status = gl::FALSE as GLint;
		gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

		if status != gl::TRUE as GLint {
			let mut length = 0;
			gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut length);

			let mut buffer = Vec::with_capacity(length as usize);
			gl::GetProgramInfoLog(program, length, &mut length, buffer.as_mut_ptr() as *mut GLchar);

			Err(CStr::from_bytes_with_nul(&buffer)
				.map(|s| s.to_string_lossy().into_owned())
				.unwrap_or("unknown error".to_owned()).into())
		}
		else {
			Ok(program)
		}
	}
}

#[derive(Debug)]
pub struct Renderer {
	program:   GLuint,
	vertex:    GLuint,
	fragment:  GLuint,
	target:    GLuint,
	attribute: (GLuint, GLint),
	rect:      (GLuint, GLuint),
}

impl Renderer {
	pub fn new(width: u32, height: u32) -> error::Result<Self> {
		unsafe {
			let vertex = compile_shader(gl::VERTEX_SHADER, r"
				#version 100

				precision lowp float;

				attribute vec2 position;
				varying vec2 v_texture;

				void main() {
					gl_Position = vec4(position, 0.0, 1.0);
					v_texture   = (position + vec2(1.0, 1.0)) / 2.0;
				}
			")?;

			let fragment = compile_shader(gl::FRAGMENT_SHADER, r"
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

			let program   = link_program(vertex, fragment)?;
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

			Ok(Renderer {
				program:   program,
				vertex:    vertex,
				fragment:  fragment,
				target:    target,
				attribute: attribute,
				rect:      (rect_array, rect_buffer),
			})
		}
	}

	pub fn texture(&self) -> GLuint {
		self.target
	}

	pub fn render(&mut self) {
		unsafe {
			gl::UseProgram(self.program);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D, self.target);
			gl::BindVertexArray(self.rect.0);
			gl::Uniform1i(self.attribute.1, 0);

			gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

			gl::BindVertexArray(0);
			gl::BindTexture(gl::TEXTURE_2D, 0);
			gl::UseProgram(0);
		}
	}

	pub fn release(&mut self) {
		unsafe {
			gl::DeleteProgram(self.program);
			gl::DeleteShader(self.vertex);
			gl::DeleteShader(self.fragment);
			gl::DeleteTextures(1, &self.target);
		}
	}
}
