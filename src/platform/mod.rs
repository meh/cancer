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

mod tty;
pub use self::tty::Tty;

mod proxy;
pub use self::proxy::Proxy;

pub mod event;
pub use self::event::Event;

pub mod key;
pub use self::key::Key;

pub mod mouse;
pub use self::mouse::Mouse;

mod clipboard;
pub use self::clipboard::Clipboard;

#[cfg(all(unix, not(target_os = "macos")))]
mod x11;
#[cfg(all(unix, not(target_os = "macos")))]
pub use self::x11::Window;

#[cfg(target_os = "macos")]
mod quartz;
#[cfg(target_os = "macos")]
pub use self::quartz::Window;
