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
use std::ffi::{CStr, CString};

use gl;
use gl::types::*;

use error;

pub fn compile_shader(kind: GLenum, source: &str) -> error::Result<GLuint> {
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

pub fn link_program(vertex: GLuint, fragment: GLuint) -> error::Result<GLuint> {
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
