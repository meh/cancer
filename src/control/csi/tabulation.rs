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
pub enum Tabulation {
	Character,
	Line,
	ClearCharacter,
	ClearLine,
	ClearLineAllCharacters,
	ClearAllCharacters,
	ClearAllLines,
}

impl Tabulation {
	pub fn parse<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Tabulation::Character),
			1 => Ok(Tabulation::Line),
			2 => Ok(Tabulation::ClearCharacter),
			3 => Ok(Tabulation::ClearLine),
			4 => Ok(Tabulation::ClearLineAllCharacters),
			5 => Ok(Tabulation::ClearAllCharacters),
			6 => Ok(Tabulation::ClearAllLines),
			_ => Err(nom::Err::Code(nom::ErrorKind::Custom(9003))),
		}
	}
}

impl Into<u32> for Tabulation {
	fn into(self) -> u32 {
		match self {
			Tabulation::Character              => 0,
			Tabulation::Line                   => 1,
			Tabulation::ClearCharacter         => 2,
			Tabulation::ClearLine              => 3,
			Tabulation::ClearLineAllCharacters => 4,
			Tabulation::ClearAllCharacters     => 5,
			Tabulation::ClearAllLines          => 6,
		}
	}
}
