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
pub enum Disposition {
	ToHome,
	ToHomeWithLeader,
	Center,
	CenterWithLeader,
	ToLimit,
	ToLimitWithLeader,
	ToBoth,
}

impl Disposition {
	pub fn parse<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Disposition::ToHome),
			1 => Ok(Disposition::ToHomeWithLeader),
			2 => Ok(Disposition::Center),
			3 => Ok(Disposition::CenterWithLeader),
			4 => Ok(Disposition::ToLimit),
			5 => Ok(Disposition::ToLimitWithLeader),
			6 => Ok(Disposition::ToBoth),
			_ => Err(nom::Err::Code(nom::ErrorKind::Custom(9002))),
		}
	}
}

impl Into<u32> for Disposition {
	fn into(self) -> u32 {
		match self {
			Disposition::ToHome            => 0,
			Disposition::ToHomeWithLeader  => 1,
			Disposition::Center            => 2,
			Disposition::CenterWithLeader  => 3,
			Disposition::ToLimit           => 4,
			Disposition::ToLimitWithLeader => 5,
			Disposition::ToBoth            => 6,
		}
	}
}
