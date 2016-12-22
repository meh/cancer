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
use std::sync::mpsc::{Receiver, Sender, channel, sync_channel};
use std::collections::HashMap;

use xcb;
use xcbu::{icccm, ewmh};

use error;
use config::Config;
use font::Font;
use platform::{Event, Clipboard};
use platform::key;
use platform::mouse::{self, Mouse};
use platform::x11::{Keyboard, Proxy};
use picto::Region;

/// X11 window.
pub struct Window {
	config:     Arc<Config>,
	connection: Arc<ewmh::Connection>,
	root:       xcb::Window,
	window:     xcb::Window,
	keyboard:   Keyboard,
	receiver:   Option<Receiver<Request>>,
	proxy:      Option<Proxy>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Request {
	Flush,
	Urgent,
	Title(String),
	Resize(u32, u32),
	Copy(Clipboard, String),
	Paste(Clipboard),
}

impl Window {
	/// Create the window.
	pub fn new(name: Option<&str>, config: Arc<Config>, font: Arc<Font>) -> error::Result<Self> {
		let margin  = config.style().margin();
		let spacing = config.style().spacing();

		let width  = (80 * font.width()) + (margin * 2);
		let height = (24 * (font.height() + spacing)) + (margin * 2);

		let (request, requests)  = channel();
		let (connection, screen) = xcb::Connection::connect(config.environment().x11().display())?;
		let connection           = Arc::new(ewmh::Connection::connect(connection).map_err(|(e, _)| e)?);
		let keyboard             = Keyboard::new(connection.clone(), config.input().locale())?;
		let (root, window)       = {
			let window = connection.generate_id();
			let screen = connection.get_setup().roots().nth(screen as usize).unwrap();
			let root   = screen.root();

			xcb::create_window(&connection, xcb::COPY_FROM_PARENT as u8, window, root,
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
						xcb::EVENT_MASK_PROPERTY_CHANGE |
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

			xcb::change_property(&connection, xcb::PROP_MODE_REPLACE as u8,
				window, connection.WM_PROTOCOLS(), xcb::ATOM_ATOM, 32, &[
					connection.WM_PING(), connection.WM_ACTION_CLOSE()]);

			xcb::map_window(&connection, window);
			connection.flush();

			(root, window)
		};

		let proxy = Proxy {
			request:    request,
			connection: connection.clone(),
			window:     window,
			screen:     screen,
		};

		Ok(Window {
			config:     config.clone(),
			connection: connection,
			root:       root,
			window:     window,
			keyboard:   keyboard,
			receiver:   Some(requests),
			proxy:      Some(proxy),
		})
	}

	pub fn proxy(&mut self) -> Proxy {
		self.proxy.take().unwrap()
	}

	#[allow(non_snake_case)]
	pub fn run(&mut self, manager: Sender<Event>) -> error::Result<()> {
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

		let mut clipboard = HashMap::new();
		let     requests  = self.receiver.take().unwrap();
		let     events    = sink(self.connection.clone());

		let PRIMARY     = xcb::ATOM_PRIMARY;
		let SECONDARY   = xcb::ATOM_SECONDARY;
		let CLIPBOARD   = xcb::intern_atom(&self.connection, false, "CLIPBOARD").get_reply().unwrap().atom();
		let UTF8_STRING = xcb::intern_atom(&self.connection, false, "UTF8_STRING").get_reply().unwrap().atom();
		let STRING      = xcb::ATOM_STRING;
		let TARGETS     = xcb::intern_atom(&self.connection, false, "TARGETS").get_reply().unwrap().atom();
		let SELECTION   = xcb::intern_atom(&self.connection, false, "CANCER_CLIPBOARD").get_reply().unwrap().atom();

		let WM_DELETE_WINDOW = xcb::intern_atom(&self.connection, false, "WM_DELETE_WINDOW").get_reply().unwrap().atom();

		loop {
			select! {
				request = requests.recv() => {
					match try!(ok request) {
						Request::Flush => {
							self.connection.flush();
						}

						Request::Urgent => {
							icccm::set_wm_hints(&self.connection, self.window, &icccm::WmHints::empty().is_urgent().build());
							xcb::bell(&self.connection, self.config.environment().x11().bell());

							self.connection.flush();
						}

						Request::Title(ref title) => {
							icccm::set_wm_name(&self.connection, self.window, title);
							ewmh::set_wm_name(&self.connection, self.window, title);
						}

						Request::Resize(w, h) => {
							xcb::configure_window(&self.connection, self.window, &[
								(xcb::CONFIG_WINDOW_WIDTH as u16, w),
								(xcb::CONFIG_WINDOW_HEIGHT as u16, h)]);
						}

						Request::Copy(name, value) => {
							let atom = match name {
								Clipboard::Primary   => PRIMARY,
								Clipboard::Secondary => SECONDARY,
								Clipboard::System    => CLIPBOARD,
							};

							debug!(target: "cancer::platform::clipboard", "set clipboard: {:?}({:?}) = {:?}", name, atom, value);

							clipboard.insert(atom, value);
							xcb::set_selection_owner(&self.connection, self.window, atom, xcb::CURRENT_TIME);
							self.connection.flush();
						}

						Request::Paste(name) => {
							let atom = match name {
								Clipboard::Primary   => PRIMARY,
								Clipboard::Secondary => SECONDARY,
								Clipboard::System    => CLIPBOARD,
							};

							debug!(target: "cancer::platform::clipboard", "get clipboard: {:?}({:?})", name, atom);

							xcb::convert_selection(&self.connection, self.window, atom, UTF8_STRING, SELECTION, xcb::CURRENT_TIME);
							self.connection.flush();
						}
					}
				},

				event = events.recv() => {
					let event = try!(event);

					match event.response_type() {
						xcb::EXPOSE => {
							let event = xcb::cast_event::<xcb::ExposeEvent>(&event);
							let x     = event.x() as u32;
							let y     = event.y() as u32;
							let w     = event.width() as u32;
							let h     = event.height() as u32;

							try!(manager.send(Event::Damaged(Region::from(x, y, w, h))));
						}

						xcb::MAP_NOTIFY | xcb::UNMAP_NOTIFY => {
							try!(manager.send(Event::Show(event.response_type() == xcb::MAP_NOTIFY)));
						}

						xcb::FOCUS_IN | xcb::FOCUS_OUT => {
							try!(manager.send(Event::Focus(event.response_type() == xcb::FOCUS_IN)));
						}

						xcb::CONFIGURE_NOTIFY => {
							let event = xcb::cast_event::<xcb::ConfigureNotifyEvent>(&event);
							let w     = event.width() as u32;
							let h     = event.height() as u32;

							try!(manager.send(Event::Resize(w, h)));
						}

						xcb::REPARENT_NOTIFY => {
							let event = xcb::cast_event::<xcb::ReparentNotifyEvent>(&event);
							let reply = try!(continue xcb::get_geometry(&self.connection, event.parent()).get_reply());

							try!(manager.send(Event::Resize(reply.width() as u32, reply.height() as u32)));
							try!(manager.send(Event::Redraw));
						}

						xcb::CLIENT_MESSAGE => {
							let event = xcb::cast_event::<xcb::ClientMessageEvent>(&event);

							if event.type_() == self.connection.WM_PROTOCOLS() {
								match event.data().data32()[0] {
									n if n == self.connection.WM_PING() => {
										xcb::send_event(&self.connection, false, self.root, 0, &xcb::ClientMessageEvent::new(
											event.format(), self.root, self.connection.WM_PROTOCOLS(), event.data().clone()));
									}

									n if n == WM_DELETE_WINDOW => {
										try!(manager.send(Event::Closed));
									}

									_ => ()
								}
							}
						}

						xcb::SELECTION_CLEAR => {
							let event = xcb::cast_event::<xcb::SelectionClearEvent>(&event);
							clipboard.remove(&event.selection());
						}

						xcb::SELECTION_REQUEST => {
							let event = xcb::cast_event::<xcb::SelectionRequestEvent>(&event);
							let reply = try!(continue xcb::get_atom_name(&self.connection, event.target()).get_reply());

							debug!(target: "cancer::platform::clipboard", "request clipboard: {:?}", reply.name());

							match reply.name() {
								"TARGETS" => {
									xcb::change_property(&self.connection, xcb::PROP_MODE_REPLACE as u8,
										event.requestor(), event.property(), xcb::ATOM_ATOM, 32, &[
											TARGETS, STRING, UTF8_STRING]);

									xcb::send_event(&self.connection, false, event.requestor(), 0, &xcb::SelectionNotifyEvent::new(
										event.time(), event.requestor(), event.selection(), event.target(), event.property()));
								}

								"UTF8_STRING" => {
									if let Some(value) = clipboard.get(&event.selection()) {
										xcb::change_property(&self.connection, xcb::PROP_MODE_REPLACE as u8,
											event.requestor(), event.property(), UTF8_STRING, 8, value.as_bytes());

										xcb::send_event(&self.connection, false, event.requestor(), 0, &xcb::SelectionNotifyEvent::new(
											event.time(), event.requestor(), event.selection(), event.target(), event.property()));
									}
								}

								"STRING" => {
									if let Some(value) = clipboard.get(&event.selection()) {
										xcb::change_property(&self.connection, xcb::PROP_MODE_REPLACE as u8,
											event.requestor(), event.property(), STRING, 8, value.as_bytes());

										xcb::send_event(&self.connection, false, event.requestor(), 0, &xcb::SelectionNotifyEvent::new(
											event.time(), event.requestor(), event.selection(), event.target(), event.property()));
									}
								}

								_ => ()
							}

							self.connection.flush();
						}

						xcb::PROPERTY_NOTIFY => {
							let event = xcb::cast_event::<xcb::PropertyNotifyEvent>(&event);

							if event.atom() == SELECTION {
								let reply = try!(continue icccm::get_text_property(&self.connection, self.window, SELECTION).get_reply());
								xcb::delete_property(&self.connection, self.window, SELECTION);

								if !reply.name().is_empty() {
									try!(manager.send(Event::Paste(reply.name().as_bytes().to_vec())));
								}
							}
						}

						xcb::BUTTON_PRESS | xcb::BUTTON_RELEASE => {
							let press = event.response_type() == xcb::BUTTON_PRESS;
							let event = xcb::cast_event::<xcb::ButtonPressEvent>(&event);

							let button = match event.detail() {
								1 => mouse::Button::Left,
								2 => mouse::Button::Middle,
								3 => mouse::Button::Right,
								4 => mouse::Button::Up,
								5 => mouse::Button::Down,
								_ => continue,
							};

							if !press && (button == mouse::Button::Up || button == mouse::Button::Down) {
								continue;
							}

							try!(manager.send(Event::Mouse(Mouse::Click(mouse::Click {
								press:    press,
								button:   button,
								modifier: key::Modifier::from(event.state()),
								position: mouse::Position {
									x: event.event_x() as u32,
									y: event.event_y() as u32,
								}
							}))));
						}

						xcb::MOTION_NOTIFY => {
							let event = xcb::cast_event::<xcb::MotionNotifyEvent>(&event);

							try!(manager.send(Event::Mouse(Mouse::Motion(mouse::Motion {
								modifier: key::Modifier::from(event.state()),
								position: mouse::Position {
									x: event.event_x() as u32,
									y: event.event_y() as u32,
								}
							}))));
						}

						e if self.keyboard.owns_event(e) => {
							self.keyboard.handle(&event);
						}

						xcb::KEY_PRESS => {
							let event = xcb::cast_event::<xcb::KeyPressEvent>(&event);

							if let Some(key) = self.keyboard.key(event.detail()) {
								try!(manager.send(Event::Key(key)));
							}
						}

						e => {
							debug!(target: "cancer::platform", "unhandled X event: {:?}", e);
						}
					}
				}
			}
		}
	}
}

impl From<u16> for key::Modifier {
	fn from(value: u16) -> Self {
		let mut result = key::Modifier::empty();

		if (value as u32 & xcb::MOD_MASK_SHIFT) != 0 {
			result.insert(key::SHIFT);
		}

		if (value as u32 & xcb::MOD_MASK_CONTROL) != 0 {
			result.insert(key::CTRL);
		}

		if (value as u32 & xcb::MOD_MASK_4) != 0 {
			result.insert(key::ALT);
		}

		result
	}
}
