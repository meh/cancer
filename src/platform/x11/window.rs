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
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel, sync_channel};

use xcb;
use xcbu::{icccm, ewmh};

use error;
use sys::cairo::Surface;
use config::Config;
use font::Font;
use platform::Event;
use platform::x11::Keyboard;
use picto::Region;

/// X11 window.
pub struct Window {
	connection: Arc<ewmh::Connection>,
	window:     xcb::Window,
	surface:    Option<Surface>,

	receiver: Option<Receiver<Event>>,
	sender:   Sender<Request>,

	width:  Arc<AtomicU32>,
	height: Arc<AtomicU32>,
	focus:  Arc<AtomicBool>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Request {
	Title(String),
	Resize(u32, u32),
	Clipboard(String, String),
}

impl Window {
	/// Create the window.
	pub fn open(name: Option<&str>, config: &Config, font: &Font) -> error::Result<Self> {
		let margin  = config.style().margin();
		let spacing = config.style().spacing();

		let mut width  = (80 * font.width()) + (margin * 2);
		let mut height = (24 * (font.height() + spacing)) + (margin * 2);

		let (connection, screen) = xcb::Connection::connect(config.environment().display())?;
		let connection           = Arc::new(ewmh::Connection::connect(connection).map_err(|(e, _)| e)?);
		let keyboard             = Keyboard::new(connection.clone())?;
		let (window, surface)    = {
			let window = connection.generate_id();
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
						xcb::EVENT_MASK_FOCUS_CHANGE |
						xcb::EVENT_MASK_EXPOSURE)]);

			icccm::set_wm_class(&connection, window, name.unwrap_or("cancer"), "Terminal");
			icccm::set_wm_name(&connection, window, name.unwrap_or("cancer"));
			ewmh::set_wm_name(&connection, window, name.unwrap_or("cancer"));

			icccm::set_wm_size_hints(&connection, window, xcb::ATOM_WM_NORMAL_HINTS, &icccm::SizeHints::empty()
				.base((margin * 2) as i32, (margin * 2) as i32)
				.min_size((font.width() + (margin * 2)) as i32, (font.height() + spacing + (margin * 2)) as i32)
				.resize(font.width() as i32, (font.height() + spacing) as i32)
				.build());

			xcb::map_window(&connection, window);
			connection.flush();

			// Wait for the window to get mapped.
			while let Some(event) = connection.wait_for_event() {
				if event.response_type() == xcb::CONFIGURE_NOTIFY {
					let configure = xcb::cast_event::<xcb::ConfigureNotifyEvent>(&event);

					width  = configure.width() as u32;
					height = configure.height() as u32;

					break;
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

			(window, create(&connection, &screen, window, width, height)?)
		};

		let (sender, i_receiver) = channel();
		let (i_sender, receiver) = channel();

		let width  = Arc::new(AtomicU32::new(width));
		let height = Arc::new(AtomicU32::new(height));
		let focus  = Arc::new(AtomicBool::new(true));

		{
			let connection = connection.clone();
			let width      = width.clone();
			let height     = height.clone();
			let focus      = focus.clone();

			fn sink(connection: Arc<ewmh::Connection>) -> Receiver<xcb::GenericEvent> {
				let (sender, receiver) = sync_channel(16);

				// Drain events into a channel.
				thread::spawn(move || {
					while let Some(event) = connection.wait_for_event() {
						try!(return sender.send(event));
					}
				});

				receiver
			}

			thread::spawn(move || {
				let     sender   = i_sender;
				let     receiver = i_receiver;
				let     events   = sink(connection.clone());
				let mut keyboard = keyboard;

				loop {
					select! {
						request = receiver.recv() => {
							match try!(return request) {
								Request::Title(ref title) => {
									icccm::set_wm_name(&connection, window, title);
									ewmh::set_wm_name(&connection, window, title);
								}

								Request::Resize(w, h) => {
									xcb::configure_window(&connection, window, &[
										(xcb::CONFIG_WINDOW_WIDTH as u16, w),
										(xcb::CONFIG_WINDOW_HEIGHT as u16, h)]);
								}

								Request::Clipboard(_name, _value) => {
									// TODO: handle clipboard
								}
							}
						},

						event = events.recv() => {
							let event = try!(return event);

							match event.response_type() {
								xcb::EXPOSE => {
									let event = xcb::cast_event::<xcb::ExposeEvent>(&event);
									let x     = event.x() as u32;
									let y     = event.y() as u32;
									let w     = event.width() as u32;
									let h     = event.height() as u32;

									try!(return sender.send(Event::Redraw(Region::from(x, y, w, h))));
								}

								xcb::FOCUS_IN | xcb::FOCUS_OUT => {
									let value = event.response_type() == xcb::FOCUS_IN;

									focus.store(value, Ordering::Relaxed);
									try!(return sender.send(Event::Focus(value)));
								}

								xcb::CONFIGURE_NOTIFY => {
									let event = xcb::cast_event::<xcb::ConfigureNotifyEvent>(&event);
									let w     = event.width() as u32;
									let h     = event.height() as u32;

									if width.load(Ordering::Relaxed) != w || height.load(Ordering::Relaxed) != h {
										width.store(w, Ordering::Relaxed);
										height.store(h, Ordering::Relaxed);

										try!(return sender.send(Event::Resize(w, h)));
									}
								}

								e if keyboard.owns_event(e) => {
									keyboard.handle(&event);
								}

								xcb::KEY_PRESS => {
									let event = xcb::cast_event::<xcb::KeyPressEvent>(&event);

									if let Some(key) = keyboard.key(event.detail()) {
										try!(return sender.send(Event::Key(key)));
									}
								}

								e => {
									debug!(target: "cancer::platform::x11", "unhandled X event: {:?}", e);
								}
							}
						}
					}
				}
			});
		}

		Ok(Window {
			connection: connection,
			window:     window,
			surface:    Some(surface),

			receiver: Some(receiver),
			sender:   sender,

			width:  width,
			height: height,
			focus:  focus,
		})
	}

	/// Get the width.
	pub fn width(&self) -> u32 {
		self.width.load(Ordering::Relaxed)
	}

	/// Get the height.
	pub fn height(&self) -> u32 {
		self.height.load(Ordering::Relaxed)
	}

	/// Check if the window has focus.
	pub fn has_focus(&self) -> bool {
		self.focus.load(Ordering::Relaxed)
	}

	/// Take the events sink.
	pub fn events(&mut self) -> Receiver<Event> {
		self.receiver.take().unwrap()
	}

	/// Take the surface.
	pub fn surface(&mut self) -> Surface {
		self.surface.take().unwrap()
	}

	/// Resize the window.
	pub fn resize(&mut self, width: u32, height: u32) {
		self.sender.send(Request::Resize(width, height)).unwrap();
	}

	/// Set the window title.
	pub fn set_title<T: Into<String>>(&self, title: T) {
		self.sender.send(Request::Title(title.into())).unwrap();
	}

	/// Set the clipboard.
	pub fn clipboard<T1: Into<String>, T2: Into<String>>(&self, name: T1, value: T2) {
		self.sender.send(Request::Clipboard(name.into(), value.into())).unwrap();
	}

	/// Flush the surface and connection.
	pub fn flush(&self) {
		self.connection.flush();
	}
}
