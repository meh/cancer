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
use std::os::unix::io::RawFd;

use wayland_client::{StateGuard, EnvHandler, default_connect, EventQueue, EventQueueHandle, Init, Proxy};
use wayland_client::protocol::{wl_compositor, wl_seat, wl_shell, wl_shm, wl_subcompositor};
use wayland_client::protocol::{wl_display, wl_registry, wl_output, wl_surface, wl_pointer};
use wayland_client::protocol::{wl_keyboard};
use wayland_kbd::MappedKeyboard;
use wayland_window;

use platform::Event;

wayland_env!(Inner,
	compositor:    wl_compositor::WlCompositor,
	subcompositor: wl_subcompositor::WlSubcompositor,
	shm:           wl_shm::WlShm,
	shell:         wl_shell::WlShell);

pub struct Environment {
	pub(super) registry: Arc<wl_registry::WlRegistry>,
	pub(super) id:       usize,
	pub(super) seat:     Option<wl_seat::WlSeat>,
	pub(super) mouse:    Option<wl_pointer::WlPointer>,
	pub(super) keyboard: Option<wl_keyboard::WlKeyboard>,

	pub(super) monitors: Vec<Monitor>,
	pub(super) surface:  Option<Arc<wl_surface::WlSurface>>,
	pub(super) manager:  Option<Sender<Event>>,
}

pub struct Monitor {
	pub(super) output: wl_output::WlOutput,
	pub(super) id:     u32,
	pub(super) scale:  f32,
	pub(super) size:   (u32, u32),
	pub(super) name:   String,
}

impl Monitor {
	pub fn new(output: wl_output::WlOutput, id: u32) -> Self {
		Monitor {
			output: output,
			id:     id,
			scale:  1.0,
			size:   (0, 0),
			name:   "".into(),
		}
	}
}

impl Environment {
	pub fn new(queue: &mut EventQueue, registry: Arc<wl_registry::WlRegistry>) -> usize {
		let value = Environment {
			registry: registry.clone(),
			id:       0,
			seat:     None,
			mouse:    None,
			keyboard: None,

			monitors: Vec::new(),
			surface:  None,
			manager:  None,
		};

		queue.add_handler(EnvHandler::<Inner>::new());
		queue.register::<_, EnvHandler<Inner>>(&registry, 0);
		queue.add_handler_with_init(value)
	}
}

pub fn with<T, F>(registry: &wl_registry::WlRegistry, state: &mut StateGuard, func: F) -> T
	where F: FnOnce(&wl_compositor::WlCompositor,
		              &wl_subcompositor::WlSubcompositor,
		              &wl_shm::WlShm,
		              &wl_shell::WlShell,
		              Option<wl_seat::WlSeat>) -> T
{
	let mut seat  = None;
	let mut inner = state.get_mut_handler::<EnvHandler<Inner>>(0);

	for &(name, ref interface, version) in inner.globals() {
		if interface == "wl_seat" {
			seat = Some(registry.bind::<wl_seat::WlSeat>(5, name).expect("no seat"));
			break;
		}
	}

	func(&inner.compositor, &inner.subcompositor, &inner.shm, &inner.shell, seat)
}

impl Init for Environment {
	fn init(&mut self, queue: &mut EventQueueHandle, index: usize) {
		queue.register::<_, Environment>(&*self.registry, index);
		self.id = index;
	}
}

impl wl_registry::Handler for Environment {
	fn global(&mut self, queue: &mut EventQueueHandle, registry: &wl_registry::WlRegistry, name: u32, interface: String, version: u32) {
		match &*interface {
			"wl_output" => {
				let output = self.registry.bind::<wl_output::WlOutput>(1, name).expect("no output");

				queue.register::<_, Environment>(&output, self.id);
				self.monitors.push(Monitor::new(output, name));
			}

			"wl_seat" if self.seat.is_none() => {
				let seat = self.registry.bind::<wl_seat::WlSeat>(5, name).expect("no seat");

				queue.register::<_, Environment>(&seat, self.id);
				self.seat = Some(seat);
			}

			_ => ()
		}
	}

	fn global_remove(&mut self, queue: &mut EventQueueHandle, registry: &wl_registry::WlRegistry, name: u32) {
		self.monitors.retain(|m| m.id != name);
	}
}

declare_handler!(Environment, wl_registry::Handler, wl_registry::WlRegistry);

impl wl_output::Handler for Environment {
	fn geometry(&mut self, _queue: &mut EventQueueHandle, output: &wl_output::WlOutput, _x: i32, _y: i32, _width: i32, _height: i32, _subpixel: wl_output::Subpixel, make: String, model: String, _transform: wl_output::Transform) {
		for monitor in self.monitors.iter_mut().filter(|m| m.output.equals(output)) {
			monitor.name = format!("{} ({})", model, make);
			break;
		}
	}

	fn mode(&mut self, _queue: &mut EventQueueHandle, output: &wl_output::WlOutput, mode: wl_output::Mode, width: i32, height: i32, _refresh: i32) {
		if mode.contains(wl_output::Current) {
			for monitor in self.monitors.iter_mut().filter(|m| m.output.equals(output)) {
				monitor.size = (width as u32, height as u32);
				break;
			}
		}
	}

	fn scale(&mut self, _queue: &mut EventQueueHandle, output: &wl_output::WlOutput, factor: i32) {
		for monitor in self.monitors.iter_mut().filter(|m| m.output.equals(output)) {
			monitor.scale = factor as f32;
			break;
		}
	}
}

declare_handler!(Environment, wl_output::Handler, wl_output::WlOutput);

impl wl_seat::Handler for Environment {
	fn capabilities(&mut self, queue: &mut EventQueueHandle, seat: &wl_seat::WlSeat, capabilities: wl_seat::Capability) {
		if capabilities.contains(wl_seat::Pointer) && self.mouse.is_none() {
			let pointer = seat.get_pointer().expect("no pointer");

			queue.register::<_, Environment>(&pointer, self.id);
			self.mouse = Some(pointer);
		}

		if !capabilities.contains(wl_seat::Pointer) {
			if let Some(pointer) = self.mouse.take() {
				pointer.release();
			}
		}

		if capabilities.contains(wl_seat::Keyboard) && self.keyboard.is_none() {
			let keyboard = seat.get_keyboard().expect("no keyboard");

			queue.register::<_, Environment>(&keyboard, self.id);
			self.keyboard = Some(keyboard);
		}

		if !capabilities.contains(wl_seat::Keyboard) {
			if let Some(keyboard) = self.keyboard.take() {
				keyboard.release();
			}
		}
	}
}

declare_handler!(Environment, wl_seat::Handler, wl_seat::WlSeat);

impl wl_pointer::Handler for Environment {
	fn enter(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _serial: u32, surface: &wl_surface::WlSurface, x: f64, y: f64) {

	}

	fn leave(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _serial: u32, surface: &wl_surface::WlSurface) {

	}

	fn motion(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _time: u32, x: f64, y: f64) {

	}

	fn button(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _serial: u32, _time: u32, button: u32, state: wl_pointer::ButtonState) {

	}

	fn axis(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _time: u32, axis: wl_pointer::Axis, value: f64) {

	}

	fn frame(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer) {

	}

	fn axis_source(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _source: wl_pointer::AxisSource) {

	}

	fn axis_stop(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _time: u32, _axis: wl_pointer::Axis) {

	}

	fn axis_discrete(&mut self, _queue: &mut EventQueueHandle, _pointer: &wl_pointer::WlPointer, _axis: wl_pointer::Axis, discrete: i32) {

	}
}

declare_handler!(Environment, wl_pointer::Handler, wl_pointer::WlPointer);

impl wl_keyboard::Handler for Environment {
	fn keymap(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, _format: wl_keyboard::KeymapFormat, fd: RawFd, size: u32) {

	}

	fn enter(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, _serial: u32, _surface: &wl_surface::WlSurface, keys: Vec<u8>) {

	}

	fn leave(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, _serial: u32, _surface: &wl_surface::WlSurface) {

	}

	fn key(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, _serial: u32, _time: u32, _key: u32, _state: wl_keyboard::KeyState) {

	}

	fn modifiers(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, _serial: u32, depressed: u32, latched: u32, locked: u32, group: u32) {

	}

	fn repeat_info(&mut self, _queue: &mut EventQueueHandle, _keyboard: &wl_keyboard::WlKeyboard, rate: i32, delay: i32) {

	}
}

declare_handler!(Environment, wl_keyboard::Handler, wl_keyboard::WlKeyboard);
