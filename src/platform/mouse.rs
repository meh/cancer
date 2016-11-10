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

use platform::key;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Mouse {
	Click(Click),
	Motion(Motion),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Click {
	pub press:    bool,
	pub modifier: key::Modifier,
	pub button:   Button,
	pub position: Position,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Motion {
	pub modifier: key::Modifier,
	pub position: Position,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Button {
	Left,
	Middle,
	Right,
	Up,
	Down,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Position {
	pub x: u32,
	pub y: u32,
}
