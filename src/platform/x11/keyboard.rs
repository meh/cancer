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

use xcb;
use xcbu::ewmh;
use xkbcommon::xkb::{self, keysyms};

use error;
use platform::key::{self, Key, Button, Keypad, Modifier};

pub struct Keyboard {
	connection: Arc<ewmh::Connection>,
	context:    xkb::Context,
	device:     i32,
	keymap:     xkb::Keymap,
	state:      xkb::State,
}

unsafe impl Send for Keyboard { }

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

	pub fn key(&self, code: u8) -> Option<Key> {
		const MODIFIERS: &[(&str, Modifier)] = &[
			(xkb::MOD_NAME_ALT,   key::ALT),
			(xkb::MOD_NAME_CTRL,  key::CTRL),
			(xkb::MOD_NAME_CAPS,  key::CAPS),
			(xkb::MOD_NAME_LOGO,  key::LOGO),
			(xkb::MOD_NAME_NUM,   key::NUM),
			(xkb::MOD_NAME_SHIFT, key::SHIFT)];

		let modifier = MODIFIERS.iter().fold(Modifier::empty(), |modifier, &(n, m)|
			if self.state.mod_name_is_active(&n, xkb::STATE_MODS_EFFECTIVE) {
				modifier | m
			}
			else {
				modifier
			});

		let symbol = self.symbol(code);

		Some(Key::new(match symbol {
			keysyms::KEY_Escape =>
				Button::Escape.into(),

			keysyms::KEY_Return =>
				Button::Enter.into(),

			keysyms::KEY_BackSpace =>
				Button::Backspace.into(),

			keysyms::KEY_Delete =>
				Button::Delete.into(),

			keysyms::KEY_Insert =>
				Button::Insert.into(),

			keysyms::KEY_Home =>
				Button::Home.into(),

			keysyms::KEY_End =>
				Button::End.into(),

			keysyms::KEY_Page_Up =>
				Button::PageUp.into(),

			keysyms::KEY_Page_Down =>
				Button::PageDown.into(),

			keysyms::KEY_Up =>
				Button::Up.into(),

			keysyms::KEY_Down =>
				Button::Down.into(),

			keysyms::KEY_Right =>
				Button::Right.into(),

			keysyms::KEY_Left =>
				Button::Left.into(),

			keysyms::KEY_F1  => Button::F1.into(),
			keysyms::KEY_F2  => Button::F2.into(),
			keysyms::KEY_F3  => Button::F3.into(),
			keysyms::KEY_F4  => Button::F4.into(),
			keysyms::KEY_F5  => Button::F5.into(),
			keysyms::KEY_F6  => Button::F6.into(),
			keysyms::KEY_F7  => Button::F7.into(),
			keysyms::KEY_F8  => Button::F8.into(),
			keysyms::KEY_F9  => Button::F9.into(),
			keysyms::KEY_F10 => Button::F10.into(),
			keysyms::KEY_F11 => Button::F11.into(),
			keysyms::KEY_F12 => Button::F12.into(),
			keysyms::KEY_F13 => Button::F13.into(),
			keysyms::KEY_F14 => Button::F14.into(),
			keysyms::KEY_F15 => Button::F15.into(),
			keysyms::KEY_F16 => Button::F16.into(),
			keysyms::KEY_F17 => Button::F17.into(),
			keysyms::KEY_F18 => Button::F18.into(),
			keysyms::KEY_F19 => Button::F19.into(),
			keysyms::KEY_F20 => Button::F20.into(),
			keysyms::KEY_F21 => Button::F21.into(),
			keysyms::KEY_F22 => Button::F22.into(),
			keysyms::KEY_F23 => Button::F23.into(),
			keysyms::KEY_F24 => Button::F24.into(),
			keysyms::KEY_F25 => Button::F25.into(),
			keysyms::KEY_F26 => Button::F26.into(),
			keysyms::KEY_F27 => Button::F27.into(),
			keysyms::KEY_F28 => Button::F28.into(),
			keysyms::KEY_F29 => Button::F29.into(),
			keysyms::KEY_F30 => Button::F30.into(),
			keysyms::KEY_F31 => Button::F31.into(),
			keysyms::KEY_F32 => Button::F32.into(),
			keysyms::KEY_F33 => Button::F33.into(),
			keysyms::KEY_F34 => Button::F34.into(),
			keysyms::KEY_F35 => Button::F35.into(),

			keysyms::KEY_KP_Enter =>
				Keypad::Enter.into(),

			keysyms::KEY_KP_Home =>
				Keypad::Home.into(),

			keysyms::KEY_KP_Begin =>
				Keypad::Begin.into(),

			keysyms::KEY_KP_End =>
				Keypad::End.into(),

			keysyms::KEY_KP_Insert =>
				Keypad::Insert.into(),

			keysyms::KEY_KP_Multiply =>
				Keypad::Multiply.into(),

			keysyms::KEY_KP_Add =>
				Keypad::Add.into(),

			keysyms::KEY_KP_Subtract =>
				Keypad::Subtract.into(),

			keysyms::KEY_KP_Divide =>
				Keypad::Divide.into(),

			keysyms::KEY_KP_Decimal =>
				Keypad::Decimal.into(),

			keysyms::KEY_KP_Page_Up =>
				Keypad::PageUp.into(),

			keysyms::KEY_KP_Page_Down =>
				Keypad::PageDown.into(),

			keysyms::KEY_KP_Up =>
				Keypad::Up.into(),

			keysyms::KEY_KP_Down =>
				Keypad::Down.into(),

			keysyms::KEY_KP_Right =>
				Keypad::Right.into(),

			keysyms::KEY_KP_Left =>
				Keypad::Left.into(),

			keysyms::KEY_KP_0 ... keysyms::KEY_KP_9 =>
				Keypad::Number((symbol - keysyms::KEY_KP_0) as u8).into(),

			_ => {
				let string = self.string(code);

				if string.is_empty() {
					return None;
				}

				string.into()
			}
		}, modifier))
	}
}
