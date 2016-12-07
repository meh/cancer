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

use wayland_client::{EnvHandler, default_connect, EventQueue, EventQueueHandle, Init, Proxy as WlProxy};
use wayland_client::protocol::{wl_compositor, wl_seat, wl_shell, wl_shm, wl_subcompositor};
use wayland_client::protocol::{wl_display, wl_registry, wl_output, wl_surface, wl_pointer};
use wayland_client::protocol::{wl_keyboard, wl_shell_surface};
use wayland_window::{self, DecoratedSurface};

use error;
use config::Config;
use font::Font;
use platform::Event;
use platform::wayland::{Proxy, Handler};

pub struct Window {
	config: Arc<Config>,
	proxy:  Option<Proxy>,

	display:  Arc<wl_display::WlDisplay>,
	queue:    EventQueue,
	registry: Arc<wl_registry::WlRegistry>,
	surface:  Arc<wl_surface::WlSurface>,

	mouse:    Option<wl_pointer::WlPointer>,
	keyboard: Option<wl_keyboard::WlKeyboard>,

	width:  Arc<AtomicU32>,
	height: Arc<AtomicU32>,
}

wayland_env!(Environment,
    compositor:    wl_compositor::WlCompositor,
    subcompositor: wl_subcompositor::WlSubcompositor,
    shm:           wl_shm::WlShm,
    shell:         wl_shell::WlShell);

impl Window {
	pub fn new(name: Option<&str>, config: Arc<Config>, font: Arc<Font>) -> error::Result<Self> {
		let margin  = config.style().margin();
		let spacing = config.style().spacing();

		let width  = (80 * font.width()) + (margin * 2);
		let height = (24 * (font.height() + spacing)) + (margin * 2);

		// Connect to the server.
		let (display, mut queue) = default_connect()?;
		let display              = Arc::new(display);
		queue.add_handler(EnvHandler::<Environment>::new());

		// Initialize connection.
		let registry = Arc::new(display.get_registry().expect("no registry"));
		queue.register::<_, EnvHandler<Environment>>(&registry, 0);
		queue.sync_roundtrip()?;

		// Create the surface and decorations.
		let (surface, decorator) = {
			let seat  = Window::seat(&mut queue, &registry);
			let state = queue.state();
			let env   = state.get_handler::<EnvHandler<Environment>>(0);

			let     surface   = env.compositor.create_surface().expect("no surface");
			let mut decorator = DecoratedSurface::new(&surface, width as i32, height as i32,
				&env.compositor, &env.subcompositor, &env.shm, &env.shell, seat, true)?;

			decorator.set_title("cancer".into());
			decorator.set_class(name.unwrap_or("cancer").into());
			*decorator.handler() = Some(Decorator::default());

			(Arc::new(surface), decorator)
		};

		queue.add_handler_with_init(decorator);

		let width  = Arc::new(AtomicU32::new(width));
		let height = Arc::new(AtomicU32::new(height));

		let proxy = Proxy {
			display: display.clone(),
			surface: surface.clone(),
			inner:   None,

			width:  width.clone(),
			height: height.clone(),
		};

		Ok(Window {
			config: config.clone(),
			proxy:  Some(proxy),

			display:  display,
			queue:    queue,
			registry: registry,
			surface:  surface,

			mouse:    None,
			keyboard: None,

			width:  width,
			height: height,
		})
	}

	pub fn proxy(&mut self) -> Proxy {
		self.proxy.take().unwrap()
	}

	pub fn run(&mut self, manager: Sender<Event>) -> error::Result<()> {
		let handler = self.queue.add_handler_with_init(Handler::new(manager.clone()));
		self.queue.register::<_, Handler>(&*self.surface, handler);

		if let Some(seat) = Window::seat(&mut self.queue, &self.registry) {
			let pointer = seat.get_pointer().expect("no pointer");
			self.queue.register::<_, Handler>(&pointer, handler);
			self.mouse = Some(pointer);

			let keyboard = seat.get_keyboard().expect("no keyboard");
			self.queue.register::<_, Handler>(&keyboard, handler);
			self.keyboard = Some(keyboard);
		}

		manager.send(Event::Redraw);

		loop {
			self.display.flush()?;
			self.queue.dispatch()?;
		}
	}

	fn seat(queue: &mut EventQueue, registry: &wl_registry::WlRegistry) -> Option<wl_seat::WlSeat> {
		let state = queue.state();
		let env   = state.get_handler::<EnvHandler<Environment>>(0);

		for &(id, ref interface, _) in env.globals() {
			if interface == "wl_seat" {
				return Some(registry.bind(1, id).expect("no registry"));
			}
		}

		None
	}
}

impl Drop for Window {
	fn drop(&mut self) {
		self.surface.destroy();
	}
}

#[derive(Default, Debug)]
struct Decorator {
	size: Option<(i32, i32)>,
}

impl wayland_window::Handler for Decorator {
	fn configure(&mut self, _queue: &mut EventQueueHandle, _shell: wl_shell_surface::Resize, width: i32, height: i32) {
		println!("window.resize({}, {})", width, height);
		self.size = Some((width, height))
	}
}
