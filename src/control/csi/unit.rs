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
pub enum Unit {
	Character,
	Millimeter,
	ComputerDecipoint,
	Decidot,
	Mil,
	BasicMeasuringUnit,
	Micrometer,
	Pixel,
	Decipoint,
}

impl Unit {
	pub fn parse<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Unit::Character),
			1 => Ok(Unit::Millimeter),
			2 => Ok(Unit::ComputerDecipoint),
			3 => Ok(Unit::Decidot),
			4 => Ok(Unit::Mil),
			5 => Ok(Unit::BasicMeasuringUnit),
			6 => Ok(Unit::Micrometer),
			7 => Ok(Unit::Pixel),
			8 => Ok(Unit::Decipoint),
			_ => Err(nom::Err::Code(nom::ErrorKind::Custom(9002))),
		}
	}
}

impl Into<u32> for Unit {
	fn into(self) -> u32 {
		match self {
			Unit::Character          => 0,
			Unit::Millimeter         => 1,
			Unit::ComputerDecipoint  => 2,
			Unit::Decidot            => 3,
			Unit::Mil                => 4,
			Unit::BasicMeasuringUnit => 5,
			Unit::Micrometer         => 6,
			Unit::Pixel              => 7,
			Unit::Decipoint          => 8,
		}
	}
}
