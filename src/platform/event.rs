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

use picto::Region;
use platform::{Key, Mouse};

#[allow(dead_code)]
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Event {
	/// The window has been closed.
	Closed,

	/// The window is being shown or hidden.
	Show(bool),

	/// Redraw the whole screen.
	Redraw,

	/// Redraw the specified region.
	Damaged(Region),

	/// Focus change.
	Focus(bool),

	/// Window resize.
	Resize(u32, u32),

	/// Paste request.
	Paste(Vec<u8>),

	/// Key press.
	Key(Key),

	/// Mouse event.
	Mouse(Mouse),
}
