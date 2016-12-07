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

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::cell::RefCell;
use std::os::unix::io::RawFd;

use wayland_client::{EnvHandler, default_connect, EventQueue, EventQueueHandle, Init, Proxy as WlProxy};
use wayland_client::protocol::{wl_compositor, wl_seat, wl_shell, wl_shm, wl_subcompositor};
use wayland_client::protocol::{wl_display, wl_registry, wl_output, wl_surface, wl_pointer};
use wayland_client::protocol::{wl_keyboard, wl_shell_surface};
use wayland_kbd::MappedKeyboard;

use platform::{Event, Clipboard};
use platform::key;
use platform::mouse::{self, Mouse};

pub struct Handler {
	manager: Sender<Event>,
	id:      usize,
}

impl Handler {
	pub fn new(manager: Sender<Event>) -> Self {
		Handler {
			manager: manager,
			id:      0,
		}
	}
}

impl Init for Handler {
	fn init(&mut self, _queue: &mut EventQueueHandle, index: usize) {
		self.id = index
	}
}

impl wl_surface::Handler for Handler {
	fn enter(&mut self, _queue: &mut EventQueueHandle, _surface: &wl_surface::WlSurface, _output: &wl_output::WlOutput) {
		println!("surface.enter");
	}

	fn leave(&mut self, _queue: &mut EventQueueHandle, _surface: &wl_surface::WlSurface, _output: &wl_output::WlOutput) {
		println!("surface.leave");
	}
}

declare_handler!(Handler, wl_surface::Handler, wl_surface::WlSurface);

impl wl_pointer::Handler for Handler {
	fn enter(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _serial: u32, surface: &wl_surface::WlSurface, x: f64, y: f64) {
		println!("pointer.enter");
	}

	fn leave(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _serial: u32, surface: &wl_surface::WlSurface) {
		println!("pointer.leave");
	}

	fn motion(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _time: u32, x: f64, y: f64) {
		println!("pointer.motion");
	}

	fn button(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _serial: u32, _time: u32, button: u32, state: wl_pointer::ButtonState) {
		println!("pointer.button");
	}

	fn axis(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _time: u32, axis: wl_pointer::Axis, value: f64) {
		println!("pointer.axis");
	}

	fn frame(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer) {
		println!("pointer.frame");
	}

	fn axis_source(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _source: wl_pointer::AxisSource) {
		println!("pointer.axis_source");
	}

	fn axis_stop(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _time: u32, _axis: wl_pointer::Axis) {
		println!("pointer.axis_stop");
	}

	fn axis_discrete(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _axis: wl_pointer::Axis, discrete: i32) {
		println!("pointer.axis_discrete");
	}
}

declare_handler!(Handler, wl_pointer::Handler, wl_pointer::WlPointer);

impl wl_keyboard::Handler for Handler {
	fn keymap(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, _format: wl_keyboard::KeymapFormat, fd: RawFd, size: u32) {
		println!("keyboard.keymap");
	}

	fn enter(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, _serial: u32, _surface: &wl_surface::WlSurface, keys: Vec<u8>) {
		println!("keyboard.enter");
	}

	fn leave(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, _serial: u32, _surface: &wl_surface::WlSurface) {
		println!("keyboard.leave");
	}

	fn key(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, _serial: u32, _time: u32, _key: u32, _state: wl_keyboard::KeyState) {
		println!("keyboard.key");
	}

	fn modifiers(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, _serial: u32, depressed: u32, latched: u32, locked: u32, group: u32) {
		println!("keyboard.modifiers");
	}

	fn repeat_info(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, rate: i32, delay: i32) {
		println!("keyboard.repeat_info");
	}
}

declare_handler!(Handler, wl_keyboard::Handler, wl_keyboard::WlKeyboard);
