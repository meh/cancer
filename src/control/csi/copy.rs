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

use nom;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Copy {
	ToPrimary,
	FromPrimary,
	ToSecondary,
	FromSecondary,
	StopPrimary,
	StartPrimary,
	StopSecondary,
	StartSecondary,
}

impl Copy {
	pub fn parse<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Copy::ToPrimary),
			1 => Ok(Copy::FromPrimary),
			2 => Ok(Copy::ToSecondary),
			3 => Ok(Copy::FromSecondary),
			4 => Ok(Copy::StopPrimary),
			5 => Ok(Copy::StartPrimary),
			6 => Ok(Copy::StopSecondary),
			7 => Ok(Copy::StartSecondary),
			_ => Err(nom::Err::Code(nom::ErrorKind::Custom(9005))),
		}
	}
}

impl Into<u32> for Copy {
	fn into(self) -> u32 {
		match self {
			Copy::ToPrimary      => 0,
			Copy::FromPrimary    => 1,
			Copy::ToSecondary    => 2,
			Copy::FromSecondary  => 3,
			Copy::StopPrimary    => 4,
			Copy::StartPrimary   => 5,
			Copy::StopSecondary  => 6,
			Copy::StartSecondary => 7,
		}
	}
}
