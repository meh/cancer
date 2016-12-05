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

use wayland_client::{EnvHandler, default_connect, EventQueue, EventQueueHandle, Init, Proxy as WlProxy};
use wayland_client::protocol::{wl_compositor, wl_seat, wl_shell, wl_shm, wl_subcompositor};
use wayland_client::protocol::{wl_display, wl_registry, wl_output, wl_surface, wl_pointer};
use wayland_client::protocol::{wl_keyboard, wl_shell_surface};
use wayland_kbd::MappedKeyboard;
use wayland_window::{self, DecoratedSurface};

use error;
use config::Config;
use font::Font;
use platform::{Event, Clipboard};
use platform::key;
use platform::mouse::{self, Mouse};
use platform::wayland::Proxy;

pub struct Window {
	config: Arc<Config>,
	proxy:  Option<Proxy>,

	display:  Arc<wl_display::WlDisplay>,
	queue:    EventQueue,
	registry: Arc<wl_registry::WlRegistry>,
	surface:  Arc<wl_surface::WlSurface>,
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

		// Create the surface.
		let surface = Arc::new({
			let state = queue.state();
			let env   = state.get_handler::<EnvHandler<Environment>>(0);

			env.compositor.create_surface().expect("no surface")
		});

		// Find the first available seat.
		let mut seat = None;
		{
			let state = queue.state();
			let env   = state.get_handler::<EnvHandler<Environment>>(0);

			for &(id, ref interface, _) in env.globals() {
				if interface == "wl_seat" {
					seat = Some(registry.bind(1, id).expect("no registry"));
					break;
				}
			}
		}

		// Create the window decoration.
		let mut decorator = {
			let state = queue.state();
			let env   = state.get_handler::<EnvHandler<Environment>>(0);

			DecoratedSurface::new(&surface, 800, 600,
				&env.compositor, &env.subcompositor, &env.shm, &env.shell, seat, true)?
		};

		*decorator.handler() = Some(Decorator::default());
		queue.add_handler_with_init(decorator);

		let proxy = Proxy {
			display: display.clone(),
			surface: surface.clone(),
			inner:   RefCell::new(None),
		};

		Ok(Window {
			config: config.clone(),
			proxy:  Some(proxy),

			display:  display,
			queue:    queue,
			registry: registry,
			surface:  surface,
		})
	}

	pub fn proxy(&mut self) -> Proxy {
		self.proxy.take().unwrap()
	}

	pub fn run(&mut self, manager: Sender<Event>) -> error::Result<()> {
		loop {
			self.display.flush()?;
			self.queue.dispatch()?;

			println!("HUE");
		}
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
		self.size = Some((width, height))
	}
}
