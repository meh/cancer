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

use wayland_client::{Init, EventQueue, EventQueueHandle};
use wayland_client::protocol::{wl_surface, wl_shell_surface};
use wayland_window::{self, DecoratedSurface};

use error;
use config::Config;
use font::Font;
use platform::{Event, Clipboard};
use platform::key;
use platform::mouse::{self, Mouse};
use platform::wayland::Proxy;
use platform::wayland::context::{self, Context};

pub struct Window {
	config: Arc<Config>,
	proxy:  Option<Proxy>,

	context: Arc<Context>,
	surface: Arc<wl_surface::WlSurface>,

	handler:    usize,
	decoration: usize,

	width:  Arc<AtomicU32>,
	height: Arc<AtomicU32>,
}

impl Window {
	pub fn new(name: Option<&str>, config: Arc<Config>, font: Arc<Font>) -> error::Result<Self> {
		let margin  = config.style().margin();
		let spacing = config.style().spacing();

		let width  = (80 * font.width()) + (margin * 2);
		let height = (24 * (font.height() + spacing)) + (margin * 2);

		let context               = Arc::new(Context::new()?);
		let (surface, decoration) = context.create::<Decorator>(width, height)?;

		let decoration = {
			let mut queue = context.queue.lock().unwrap();
			let     id    = queue.add_handler_with_init(decoration);
			let mut state = queue.state();

			let decoration          = state.get_mut_handler::<DecoratedSurface<Decorator>>(id);
			*(decoration.handler()) = Some(Decorator::default());

			id
		};

		let width  = Arc::new(AtomicU32::new(width));
		let height = Arc::new(AtomicU32::new(height));

		let handler = Handler::default();
		let handler = context.queue.lock().unwrap().add_handler_with_init(handler);

		let proxy = Proxy {
			width:  width.clone(),
			height: height.clone(),

			context: context.clone(),
			surface: surface.clone(),
			window:  RefCell::new(None),
		};

		Ok(Window {
			config: config.clone(),
			proxy:  Some(proxy),

			context: context,
			surface: surface,

			handler:    handler,
			decoration: decoration,

			width:  width,
			height: height,
		})
	}

	pub fn proxy(&mut self) -> Proxy {
		self.proxy.take().unwrap()
	}

	pub fn run(&mut self, manager: Sender<Event>) -> error::Result<()> {
		context::prepare(&self.context, manager);

		loop {
			println!("FLUSH");
			self.context.flush()?;
			println!("DISPATCH");
			self.context.dispatch()?;

			println!("RESIZE CHECK");
			{
				let mut queue     = self.context.queue.lock().unwrap();
				let mut state     = queue.state();
				let mut decorator = state.get_mut_handler::<DecoratedSurface<Decorator>>(self.decoration);

				if let Some((w, h)) = decorator.handler().as_mut().unwrap().size.take() {
					decorator.resize(w as i32, h as i32);
					println!("RESIZE: {} {}", w, h);
				}
			}

			println!("hue");
		}

		Ok(())
	}
}

impl Drop for Window {
	fn drop(&mut self) {
		self.surface.destroy();
	}
}

#[derive(Default, Debug)]
struct Decorator {
	size: Option<(u32, u32)>,
}

impl wayland_window::Handler for Decorator {
	fn configure(&mut self, _queue: &mut EventQueueHandle, _surface: wl_shell_surface::Resize, width: i32, height: i32) {
		use std::cmp;
		self.size = Some((cmp::max(1, width) as u32, cmp::max(1, height) as u32));
	}
}

#[derive(Default, Debug)]
struct Handler {
	id: usize,
}

impl Init for Handler {
	fn init(&mut self, _queue: &mut EventQueueHandle, index: usize) {
		self.id = index;
	}
}
