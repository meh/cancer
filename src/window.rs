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

use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel};

use xcb;
use xcbu;

use error;
use sys::cairo::Surface;
use config::Config;
use font::Font;

pub struct Window {
	connection: Arc<xcbu::ewmh::Connection>,
	window:     xcb::Window,
	surface:    Surface,
	events:     Option<Receiver<xcb::GenericEvent>>,

	width:  u32,
	height: u32,
}

impl Window {
	/// Create the window.
	pub fn open(config: Arc<Config>, font: Arc<Font>) -> error::Result<Self> {
		let mut width  = (80 * font.width()) + (config.style().margin() * 2);
		let mut height = (24 * font.height()) + (24 * config.style().spacing()) + (config.style().margin() * 2);

		let (connection, screen) = xcb::Connection::connect(config.environment().display())?;
		let connection           = Arc::new(xcbu::ewmh::Connection::connect(connection).map_err(|(e, _)| e)?);
		let events               = sink(connection.clone());
		let (window, surface)    = {
			let window               = connection.generate_id();
			let screen = connection.get_setup().roots().nth(screen as usize).unwrap();

			xcb::create_window(&connection, xcb::COPY_FROM_PARENT as u8, window, screen.root(),
				0, 0, width as u16, height as u16,
				0, xcb::WINDOW_CLASS_INPUT_OUTPUT as u16, screen.root_visual(), &[
					(xcb::CW_BACKING_PIXEL, screen.black_pixel()),
					(xcb::CW_EVENT_MASK,
						xcb::EVENT_MASK_KEY_PRESS |
						xcb::EVENT_MASK_KEY_RELEASE |
						xcb::EVENT_MASK_BUTTON_PRESS |
						xcb::EVENT_MASK_BUTTON_RELEASE |
						xcb::EVENT_MASK_POINTER_MOTION |
						xcb::EVENT_MASK_STRUCTURE_NOTIFY |
						xcb::EVENT_MASK_EXPOSURE)]);

			xcb::map_window(&connection, window);
			connection.flush();

			// Wait for the window to get mapped.
			while let Ok(event) = events.recv() {
				if event.response_type() == xcb::CONFIGURE_NOTIFY {
					let configure = xcb::cast_event::<xcb::ConfigureNotifyEvent>(&event);

					width  = configure.width() as u32;
					height = configure.height() as u32;

					break;
				}
			}

			(window, create(&connection, &screen, window, width, height)?)
		};

		Ok(Window {
			connection: connection,
			window:     window,
			surface:    surface,
			events:     Some(events),

			width:  width,
			height: height,
		})
	}

	/// Get the width.
	pub fn width(&self) -> u32 {
		self.width
	}

	/// Get the height.
	pub fn height(&self) -> u32 {
		self.height
	}

	/// Handle a resize event.
	pub fn resized(&mut self, width: u32, height: u32) {
		self.width  = width;
		self.height = height;
		self.surface.resize(width, height);
	}

	/// Take the events sink.
	pub fn events(&mut self) -> Receiver<xcb::GenericEvent> {
		self.events.take().unwrap()
	}

	/// Flush the surface and connection.
	pub fn flush(&self) {
		self.surface.flush();
		self.connection.flush();
	}
}

fn create(connection: &xcb::Connection, screen: &xcb::Screen, window: xcb::Window, width: u32, height: u32) -> error::Result<Surface> {
	for item in screen.allowed_depths() {
		if item.depth() == 24 {
			for visual in item.visuals() {
				return Ok(Surface::new(connection, window, visual, width, height));
			}
		}
	}

	Err(error::X::MissingDepth(24).into())
}

fn sink(connection: Arc<xcbu::ewmh::Connection>) -> Receiver<xcb::GenericEvent> {
	let (sender, receiver) = sync_channel(1);

	// Drain events into a channel.
	thread::spawn(move || {
		while let Some(event) = connection.wait_for_event() {
			sender.send(event).unwrap();
		}
	});

	receiver
}

impl AsRef<Surface> for Window {
	fn as_ref(&self) -> &Surface {
		&self.surface
	}
}
