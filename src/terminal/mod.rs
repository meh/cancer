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

pub trait Access {
	fn access(&self, x: u32, y: u32) -> &Cell;
}

mod iter;
pub use self::iter::Iter;

pub mod cell;
pub use self::cell::Cell;

mod row;
pub use self::row::Row;

mod free;
pub use self::free::Free;

pub mod touched;
pub use self::touched::Touched;

pub mod mode;
pub use self::mode::Mode;

pub mod cursor;
pub use self::cursor::Cursor;

pub mod grid;
pub use self::grid::Grid;

mod tabs;
pub use self::tabs::Tabs;

mod input;
pub use self::input::Input;

mod terminal;
pub use self::terminal::Terminal;
