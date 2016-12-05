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
use std::sync::mpsc::Sender;

use wayland_client::{EnvHandler, default_connect, EventQueue, EventQueueHandle, Init};
use wayland_client::protocol::{wl_compositor, wl_seat, wl_shell, wl_shm, wl_subcompositor};
use wayland_client::protocol::{wl_display, wl_registry, wl_output, wl_surface, wl_pointer};
use wayland_client::protocol::{wl_keyboard};
use wayland_kbd::MappedKeyboard;
use wayland_window;

use error;
use platform::Event;
use platform::wayland::environment::{self, Environment};

pub struct Context {
	pub(super) display:  wl_display::WlDisplay,
	pub(super) queue:    Mutex<EventQueue>,
	pub(super) registry: Arc<wl_registry::WlRegistry>,
	pub(super) id:       usize,
}

impl Context {
	pub fn new() -> error::Result<Self> {
		let (display, mut queue) = default_connect()?;
		let registry             = Arc::new(display.get_registry().expect("no registry"));
		let id                   = Environment::new(&mut queue, registry.clone());

		queue.sync_roundtrip()?;
		queue.sync_roundtrip()?;

		Ok(Context {
			queue:    Mutex::new(queue),
			registry: registry,
			display:  display,
			id:       id,
		})
	}

	pub fn dispatch_pending(&self) -> error::Result<()> {
		self.queue.lock().unwrap().dispatch_pending()?;
		Ok(())
	}

	pub fn dispatch(&self) -> error::Result<()> {
		self.queue.lock().unwrap().dispatch()?;

		Ok(())
	}

	pub fn flush(&self) -> error::Result<()> {
		self.display.flush()?;

		Ok(())
	}

	pub fn create<H>(&self, width: u32, height: u32) -> error::Result<(Arc<wl_surface::WlSurface>, wayland_window::DecoratedSurface<H>)>
		where H: wayland_window::Handler
	{
		let mut queue = self.queue.lock().unwrap();
		let mut state = queue.state();

		let surface = environment::with(&self.registry, &mut state, |compositor, _, _, _, _|
			Arc::new(compositor.create_surface().expect("no surface")));

		let decorated = environment::with(&self.registry, &mut state, |compositor, subcompositor, shm, shell, seat|
			wayland_window::DecoratedSurface::new(
				&surface, width as i32, height as i32,
				compositor, subcompositor, shm, shell, seat, true))?;

		let env = state.get_mut_handler::<Environment>(self.id);
		env.surface = Some(surface.clone());

		Ok((surface, decorated))
	}
}

pub fn prepare(context: &Context, manager: Sender<Event>) {
	let mut queue = context.queue.lock().unwrap();
	let mut state = queue.state();
	let     env   = state.get_mut_handler::<Environment>(context.id);

	env.manager = Some(manager);
}
