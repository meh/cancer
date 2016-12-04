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
use std::env;

use xcb;
use xcbu::ewmh;
use xkbcommon::xkb::{self, keysyms};
use xkbcommon::xkb::compose::Status;

use error;
use platform::key::{self, Key, Button, Keypad, Modifier, Lock};

pub struct Keyboard {
	connection: Arc<ewmh::Connection>,
	context:    xkb::Context,
	device:     i32,
	keymap:     xkb::Keymap,
	state:      xkb::State,

	#[allow(dead_code)]
	table:   xkb::compose::Table,
	compose: xkb::compose::State,
}

unsafe impl Send for Keyboard { }

impl Keyboard {
	/// Create a keyboard for the given connection.
	pub fn new(connection: Arc<ewmh::Connection>, locale: Option<&str>) -> error::Result<Self> {
		connection.get_extension_data(xcb::xkb::id())
			.ok_or(error::platform::Error::MissingExtension)?;

		// Check the XKB extension version.
		{
			let cookie = xcb::xkb::use_extension(&connection,
				xkb::x11::MIN_MAJOR_XKB_VERSION,
				xkb::x11::MIN_MINOR_XKB_VERSION);

			if !cookie.get_reply()?.supported() {
				return Err(error::platform::Error::MissingExtension.into());
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

		let (table, compose) = {
			let     locale = locale.map(String::from).or(env::var("LANG").ok()).unwrap_or("C".into());
			let mut table  = if let Ok(table) = xkb::compose::Table::new(&context, &locale, 0) {
				table
			}
			else {
				xkb::compose::Table::new(&context, "C", 0).unwrap()
			};

			let state = table.state(0);

			(table, state)
		};

		Ok(Keyboard {
			connection: connection,
			context:    context,
			device:     device,
			keymap:     keymap,
			state:      state,

			table:   table,
			compose: compose,
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

	pub fn key(&mut self, code: u8) -> Option<Key> {
		const MODIFIERS: &[(&str, Modifier)] = &[
			(xkb::MOD_NAME_ALT,   key::ALT),
			(xkb::MOD_NAME_CTRL,  key::CTRL),
			(xkb::MOD_NAME_LOGO,  key::LOGO),
			(xkb::MOD_NAME_SHIFT, key::SHIFT)];

		let modifier = MODIFIERS.iter().fold(Modifier::empty(), |modifier, &(n, m)|
			if self.state.mod_name_is_active(&n, xkb::STATE_MODS_EFFECTIVE) {
				modifier | m
			}
			else {
				modifier
			});

		const LOCKS: &[(&str, Lock)] = &[
			(xkb::MOD_NAME_CAPS, key::CAPS),
			(xkb::MOD_NAME_NUM,  key::NUM)];

		let lock = LOCKS.iter().fold(Lock::empty(), |lock, &(n, m)|
			if self.state.mod_name_is_active(&n, xkb::STATE_MODS_EFFECTIVE) {
				lock | m
			}
			else {
				lock
			});

		let symbol = self.symbol(code);
		self.compose.feed(symbol);

		debug!(target: "cancer::platform::key", "compose status: {:?}", self.compose.status());

		match self.compose.status() {
			Status::NOTHING => (),
			Status::COMPOSING =>
				return None,

			Status::COMPOSED => {
				if let Some(string) = self.compose.utf8() {
					self.compose.reset();
					return Some(Key::new(string.into(), modifier, lock));
				}
			}

			Status::CANCELLED => {
				self.compose.reset();
				return None;
			}
		}

		Some(Key::new(match symbol {
			keysyms::KEY_Tab | keysyms::KEY_ISO_Left_Tab =>
				Button::Tab.into(),

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

			keysyms::KEY_F1 ... keysyms::KEY_F35 =>
				Button::F((symbol - keysyms::KEY_F1 + 1) as u8).into(),

			keysyms::KEY_Menu =>
				Button::Menu.into(),

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
				let mut string = self.string(code);

				// If the string is empty, it means it's a function key which we don't
				// support, so just ignore it.
				if string.is_empty() {
					return None;
				}

				// Convert from the control code representation to the real letter.
				if modifier.contains(key::CTRL) && string.len() == 1 {
					let ch = string.as_bytes()[0];

					if ch >= 1 && ch <= 26 {
						string = unsafe { String::from_utf8_unchecked(vec![(ch - 1 + b'a')]) };
					}
				}

				string.into()
			}
		}, modifier, lock))
	}
}
