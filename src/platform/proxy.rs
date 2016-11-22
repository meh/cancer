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

use sys::cairo;
use error;

pub trait Proxy: Send {
	/// Get the width.
	fn width(&self) -> u32;

	/// Get the height.
	fn height(&self) -> u32;

	/// Check if the window has focus.
	fn has_focus(&self) -> bool;

	/// Check if the window is visible.
	fn is_visible(&self) -> bool;

	/// Create a Cairo surface.
	fn surface(&self) -> cairo::Surface;

	/// Resize the window.
	fn resize(&mut self, width: u32, height: u32);

	/// Set the window title.
	fn set_title(&self, title: String);

	/// Change the clipboard contents.
	fn copy(&self, name: String, value: String);

	/// Request the clipboard contents.
	fn paste(&self, name: String);

	/// Ask senpai to notice you.
	fn urgent(&self);

	/// Flush whatever.
	fn flush(&self);

	/// Open the given item.
	fn open(&self, value: &str) -> error::Result<()>;
}
