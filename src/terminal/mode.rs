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

bitflags! {
	pub flags Mode: u32 {
		const BLINK              = 1 << 0,
		const REVERSE            = 1 << 1,
		const WRAP               = 1 << 2,
		const BRACKETED_PASTE    = 1 << 3,
		const KEYBOARD_LOCK      = 1 << 4,
		const APPLICATION_KEYPAD = 1 << 5,
		const APPLICATION_CURSOR = 1 << 6,
		const CRLF               = 1 << 7,
		const INSERT             = 1 << 8,
		const ECHO               = 1 << 9,
		const FOCUS              = 1 << 10,

		const MOUSE_BUTTON = 1 << 11,
		const MOUSE_MOTION = 1 << 12,
		const MOUSE_SGR    = 1 << 13,
		const MOUSE_X10    = 1 << 14,
		const MOUSE_MANY   = 1 << 15,
		const MOUSE        = MOUSE_BUTTON.bits |
		                     MOUSE_MOTION.bits |
		                     MOUSE_X10.bits    |
		                     MOUSE_MANY.bits,
	}
}

impl Default for Mode {
	fn default() -> Self {
		WRAP
	}
}
