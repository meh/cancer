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
use std::convert::TryFrom;

use xcb;
use xcbu::ewmh;
use xkbcommon::xkb::{self, keysyms};

use error;
use terminal;

pub struct Keyboard {
	connection: Arc<ewmh::Connection>,
	context:    xkb::Context,
	device:     i32,
	keymap:     xkb::Keymap,
	state:      xkb::State,
}

impl Keyboard {
	/// Create a keyboard for the given connection.
	pub fn new(connection: Arc<ewmh::Connection>) -> error::Result<Self> {
		connection.get_extension_data(xcb::xkb::id())
			.ok_or(error::X::MissingExtension)?;

		// Check the XKB extension version.
		{
			let cookie = xcb::xkb::use_extension(&connection,
				xkb::x11::MIN_MAJOR_XKB_VERSION,
				xkb::x11::MIN_MINOR_XKB_VERSION);

			if !cookie.get_reply()?.supported() {
				return Err(error::X::MissingExtension.into());
			}
		}

		// Select events.
		{
			let map =
				xcb::xkb::MAP_PART_KEY_TYPES |
				xcb::xkb::MAP_PART_KEY_SYMS |
				xcb::xkb::MAP_PART_MODIFIER_MAP |
				xcb::xkb::MAP_PART_EXPLICIT_COMPONENTS |
				xcb::xkb::MAP_PART_KEY_ACTIONS |
				xcb::xkb::MAP_PART_KEY_BEHAVIORS |
				xcb::xkb::MAP_PART_VIRTUAL_MODS |
				xcb::xkb::MAP_PART_VIRTUAL_MOD_MAP;

			let events =
				xcb::xkb::EVENT_TYPE_NEW_KEYBOARD_NOTIFY |
				xcb::xkb::EVENT_TYPE_MAP_NOTIFY |
				xcb::xkb::EVENT_TYPE_STATE_NOTIFY;

			xcb::xkb::select_events_checked(&connection,
				xcb::xkb::ID_USE_CORE_KBD as u16,
				events as u16, 0, events as u16,
				map as u16, map as u16, None).request_check()?;
		}

		let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
		let device  = xkb::x11::get_core_keyboard_device_id(&connection);
		let keymap  = xkb::x11::keymap_new_from_device(&context, &connection, device, xkb::KEYMAP_COMPILE_NO_FLAGS);
		let state   = xkb::x11::state_new_from_device(&keymap, &connection, device);

		Ok(Keyboard {
			connection: connection,
			context: context,
			device:  device,
			keymap:  keymap,
			state:   state,
		})
	}

	/// Get the extension data.
	pub fn extension(&self) -> xcb::QueryExtensionData {
		self.connection.get_extension_data(xcb::xkb::id()).unwrap()
	}

	/// Checks if an event belongs to the keyboard.
	pub fn owns_event(&self, event: u8) -> bool {
		event >= self.extension().first_event() &&
		event < self.extension().first_event() + xcb::xkb::EXTENSION_DEVICE_NOTIFY
	}

	/// Handles an X event.
	pub fn handle(&mut self, event: &xcb::GenericEvent) {
		match event.response_type() - self.extension().first_event() {
			xcb::xkb::NEW_KEYBOARD_NOTIFY | xcb::xkb::MAP_NOTIFY => {
				self.keymap = xkb::x11::keymap_new_from_device(&self.context, &self.connection, self.device, xkb::KEYMAP_COMPILE_NO_FLAGS);
				self.state  = xkb::x11::state_new_from_device(&self.keymap, &self.connection, self.device);
			}

			xcb::xkb::STATE_NOTIFY => {
				let event = xcb::cast_event::<xcb::xkb::StateNotifyEvent>(event);

				self.state.update_mask(
					event.base_mods() as xkb::ModMask,
					event.latched_mods() as xkb::ModMask,
					event.locked_mods() as xkb::ModMask,
					event.base_group() as xkb::LayoutIndex,
					event.latched_group() as xkb::LayoutIndex,
					event.locked_group() as xkb::LayoutIndex);
			}

			_ => ()
		}
	}

	/// Translate a key code to the key symbol.
	pub fn symbol(&self, code: u8) -> xkb::Keysym {
		self.state.key_get_one_sym(code as xkb::Keycode)
	}

	/// Translate a key code to an UTF-8 string.
	pub fn string(&self, code: u8) -> String {
		self.state.key_get_utf8(code as xkb::Keycode)
	}
}

impl TryFrom<u32> for terminal::Key {
	type Err = ();

	fn try_from(value: u32) -> Result<Self, ()> {
		Ok(match value {
			keysyms::KEY_Return =>
				terminal::Key::Enter,

			keysyms::KEY_Escape =>
				terminal::Key::Escape,

			keysyms::KEY_Up =>
				terminal::Key::Up,

			keysyms::KEY_Down =>
				terminal::Key::Down,

			keysyms::KEY_Right =>
				terminal::Key::Right,

			keysyms::KEY_Left =>
				terminal::Key::Left,

			_ =>
				return Err(())
		})
	}
}
