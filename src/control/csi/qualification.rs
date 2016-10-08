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
pub enum Qualification {
	UnprotectedUnguarded,
	ProtectedGuarded,
	GraphicCharacterInput,
	NumericInput,
	AlphabeticInput,
	AlignLast,
	ZeroFill,
	FieldStart,
	ProtectedUnguarded,
	SpaceFill,
	AlignFirst,
	Reverse,
}

impl Qualification {
	pub fn parse<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0  => Ok(Qualification::UnprotectedUnguarded),
			1  => Ok(Qualification::ProtectedGuarded),
			2  => Ok(Qualification::GraphicCharacterInput),
			3  => Ok(Qualification::NumericInput),
			4  => Ok(Qualification::AlphabeticInput),
			5  => Ok(Qualification::AlignLast),
			6  => Ok(Qualification::ZeroFill),
			7  => Ok(Qualification::FieldStart),
			8  => Ok(Qualification::ProtectedUnguarded),
			9  => Ok(Qualification::SpaceFill),
			10 => Ok(Qualification::AlignFirst),
			11 => Ok(Qualification::Reverse),
			_  => Err(nom::Err::Code(nom::ErrorKind::Custom(9004))),
		}
	}
}

impl Into<u32> for Qualification {
	fn into(self) -> u32 {
		match self {
			Qualification::UnprotectedUnguarded  => 0,
			Qualification::ProtectedGuarded      => 1,
			Qualification::GraphicCharacterInput => 2,
			Qualification::NumericInput          => 3,
			Qualification::AlphabeticInput       => 4,
			Qualification::AlignLast             => 5,
			Qualification::ZeroFill              => 6,
			Qualification::FieldStart            => 7,
			Qualification::ProtectedUnguarded    => 8,
			Qualification::SpaceFill             => 9,
			Qualification::AlignFirst            => 10,
			Qualification::Reverse               => 11,
		}
	}
}
