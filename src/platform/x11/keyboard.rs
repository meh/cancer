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
use xkb;

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
			.ok_or(error::platform::x11::Error::MissingExtension)?;

		// Check the XKB extension version.
		{
			let cookie = xcb::xkb::use_extension(&connection,
				xkb::x11::MIN_MAJOR_XKB_VERSION,
				xkb::x11::MIN_MINOR_XKB_VERSION);

			if !cookie.get_reply()?.supported() {
				return Err(error::platform::x11::Error::MissingExtension.into());
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

		let context = xkb::Context::default();
		let device  = xkb::x11::device(&connection)?;
		let keymap  = xkb::x11::keymap(&connection, device, &context, Default::default())?;
		let state   = xkb::x11::state(&connection, device, &keymap)?;

		let (table, compose) = {
			let     locale = locale.map(String::from).or(env::var("LANG").ok()).unwrap_or("C".into());
			let mut table  = if let Ok(table) = xkb::compose::Table::new(&context, &locale, Default::default()) {
				table
			}
			else {
				xkb::compose::Table::new(&context, "C", Default::default()).unwrap()
			};

			let state = table.state(Default::default());

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
				self.keymap = xkb::x11::keymap(&self.connection, self.device, &self.context, Default::default()).unwrap();
				self.state  = xkb::x11::state(&self.connection, self.device, &self.keymap).unwrap();
			}

			xcb::xkb::STATE_NOTIFY => {
				let event = xcb::cast_event::<xcb::xkb::StateNotifyEvent>(event);

				self.state.update().mask(
					event.base_mods(),
					event.latched_mods(),
					event.locked_mods(),
					event.base_group(),
					event.latched_group(),
					event.locked_group());
			}

			_ => ()
		}
	}

	/// Translate a key code to the key symbol.
	pub fn symbol(&self, code: u8) -> Option<xkb::Keysym> {
		self.state.key(code).sym()
	}

	/// Translate a key code to an UTF-8 string.
	pub fn string(&self, code: u8) -> Option<String> {
		self.state.key(code).utf8()
	}

	pub fn key(&mut self, code: u8) -> Option<Key> {
		let modifier = [
			(xkb::name::mods::ALT,   key::ALT),
			(xkb::name::mods::CTRL,  key::CTRL),
			(xkb::name::mods::LOGO,  key::LOGO),
			(xkb::name::mods::SHIFT, key::SHIFT)
		].iter().fold(Modifier::empty(), |modifier, &(n, m)|
			if self.state.mods().active(n, xkb::state::component::MODS_EFFECTIVE) {
				modifier | m
			}
			else {
				modifier
			});

		let lock = [
			(xkb::name::mods::CAPS, key::CAPS),
			(xkb::name::mods::NUM,  key::NUM)
		].iter().fold(Lock::empty(), |lock, &(n, m)|
			if self.state.mods().active(n, xkb::state::component::MODS_EFFECTIVE) {
				lock | m
			}
			else {
				lock
			});

		let symbol = try!(option self.symbol(code));
		self.compose.feed(symbol);

		debug!(target: "cancer::platform::key", "compose status: {:?}", self.compose.status());

		match self.compose.status() {
			xkb::compose::Status::Nothing => (),
			xkb::compose::Status::Composing =>
				return None,

			xkb::compose::Status::Composed => {
				if let Some(string) = self.compose.utf8() {
					self.compose.reset();
					return Some(Key::new(string.into(), modifier, lock));
				}
			}

			xkb::compose::Status::Cancelled => {
				self.compose.reset();
				return None;
			}
		}

		Some(Key::new(match symbol {
			xkb::key::Tab | xkb::key::ISO_Left_Tab =>
				Button::Tab.into(),

			xkb::key::Escape =>
				Button::Escape.into(),

			xkb::key::Return =>
				Button::Enter.into(),

			xkb::key::BackSpace =>
				Button::Backspace.into(),

			xkb::key::Delete =>
				Button::Delete.into(),

			xkb::key::Insert =>
				Button::Insert.into(),

			xkb::key::Home =>
				Button::Home.into(),

			xkb::key::End =>
				Button::End.into(),

			xkb::key::Page_Up =>
				Button::PageUp.into(),

			xkb::key::Page_Down =>
				Button::PageDown.into(),

			xkb::key::Up =>
				Button::Up.into(),

			xkb::key::Down =>
				Button::Down.into(),

			xkb::key::Right =>
				Button::Right.into(),

			xkb::key::Left =>
				Button::Left.into(),

			xkb::key::F1  |
			xkb::key::F2  |
			xkb::key::F3  |
			xkb::key::F4  |
			xkb::key::F5  |
			xkb::key::F6  |
			xkb::key::F7  |
			xkb::key::F8  |
			xkb::key::F9  |
			xkb::key::F10 |
			xkb::key::F11 |
			xkb::key::F12 |
			xkb::key::F13 |
			xkb::key::F14 |
			xkb::key::F15 |
			xkb::key::F16 |
			xkb::key::F17 |
			xkb::key::F18 |
			xkb::key::F19 |
			xkb::key::F20 |
			xkb::key::F21 |
			xkb::key::F22 |
			xkb::key::F23 |
			xkb::key::F24 |
			xkb::key::F25 |
			xkb::key::F26 |
			xkb::key::F27 |
			xkb::key::F28 |
			xkb::key::F29 |
			xkb::key::F30 |
			xkb::key::F31 |
			xkb::key::F32 |
			xkb::key::F33 |
			xkb::key::F34 |
			xkb::key::F35 =>
				Button::F((symbol.into(): u32 - xkb::key::F1.into(): u32 + 1) as u8).into(),

			xkb::key::Menu =>
				Button::Menu.into(),

			xkb::key::KP_Enter =>
				Keypad::Enter.into(),

			xkb::key::KP_Home =>
				Keypad::Home.into(),

			xkb::key::KP_Begin =>
				Keypad::Begin.into(),

			xkb::key::KP_End =>
				Keypad::End.into(),

			xkb::key::KP_Insert =>
				Keypad::Insert.into(),

			xkb::key::KP_Multiply =>
				Keypad::Multiply.into(),

			xkb::key::KP_Add =>
				Keypad::Add.into(),

			xkb::key::KP_Subtract =>
				Keypad::Subtract.into(),

			xkb::key::KP_Divide =>
				Keypad::Divide.into(),

			xkb::key::KP_Decimal =>
				Keypad::Decimal.into(),

			xkb::key::KP_Page_Up =>
				Keypad::PageUp.into(),

			xkb::key::KP_Page_Down =>
				Keypad::PageDown.into(),

			xkb::key::KP_Up =>
				Keypad::Up.into(),

			xkb::key::KP_Down =>
				Keypad::Down.into(),

			xkb::key::KP_Right =>
				Keypad::Right.into(),

			xkb::key::KP_Left =>
				Keypad::Left.into(),

			xkb::key::KP_0 |
			xkb::key::KP_1 |
			xkb::key::KP_2 |
			xkb::key::KP_3 |
			xkb::key::KP_4 |
			xkb::key::KP_5 |
			xkb::key::KP_6 |
			xkb::key::KP_7 |
			xkb::key::KP_8 |
			xkb::key::KP_9 =>
				Keypad::Number(((symbol.into(): u32 - xkb::key::KP_0.into(): u32) as u8)).into(),

			xkb::key::space =>
				String::from(if modifier.contains(key::CTRL) { "@" } else { " " }).into(),

			_ => {
				let mut string = try!(option self.string(code));

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
