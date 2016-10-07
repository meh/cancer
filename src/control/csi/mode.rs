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
pub enum Mode {
	GuardedAreaTransfer,
	KeyboardAction,
	ControlRepresentation,
	InsertionReplacement,
	StatusReportTransfer,
	Erasure,
	LineEditing,
	BidirectionalSupport,
	DeviceComponentSelect,
	CharacterEditing,
	PositioningUnit,
	SendReceive,
	FormatEffectorAction,
	FormatEffectorTransfer,
	MultipleAreaTransfer,
	TransferTermination,
	SelectedAreaTransfer,
	TabulationStop,
	GraphicRenditionCombination,
	ZeroDefault,
}

impl Mode {
	pub fn parse<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			1  => Ok(Mode::GuardedAreaTransfer),
			2  => Ok(Mode::KeyboardAction),
			3  => Ok(Mode::ControlRepresentation),
			4  => Ok(Mode::InsertionReplacement),
			5  => Ok(Mode::StatusReportTransfer),
			6  => Ok(Mode::Erasure),
			7  => Ok(Mode::LineEditing),
			8  => Ok(Mode::BidirectionalSupport),
			9  => Ok(Mode::DeviceComponentSelect),
			10 => Ok(Mode::CharacterEditing),
			11 => Ok(Mode::PositioningUnit),
			12 => Ok(Mode::SendReceive),
			13 => Ok(Mode::FormatEffectorAction),
			14 => Ok(Mode::FormatEffectorTransfer),
			15 => Ok(Mode::MultipleAreaTransfer),
			16 => Ok(Mode::TransferTermination),
			17 => Ok(Mode::SelectedAreaTransfer),
			18 => Ok(Mode::TabulationStop),
			21 => Ok(Mode::GraphicRenditionCombination),
			22 => Ok(Mode::ZeroDefault),
			_  => Err(nom::Err::Code(nom::ErrorKind::Custom(9004))),
		}
	}
}

impl Into<u32> for Mode {
	fn into(self) -> u32 {
		match self {
			Mode::GuardedAreaTransfer         => 1,
			Mode::KeyboardAction              => 2,
			Mode::ControlRepresentation       => 3,
			Mode::InsertionReplacement        => 4,
			Mode::StatusReportTransfer        => 5,
			Mode::Erasure                     => 6,
			Mode::LineEditing                 => 7,
			Mode::BidirectionalSupport        => 8,
			Mode::DeviceComponentSelect       => 9,
			Mode::CharacterEditing            => 10,
			Mode::PositioningUnit             => 11,
			Mode::SendReceive                 => 12,
			Mode::FormatEffectorAction        => 13,
			Mode::FormatEffectorTransfer      => 14,
			Mode::MultipleAreaTransfer        => 15,
			Mode::TransferTermination         => 16,
			Mode::SelectedAreaTransfer        => 17,
			Mode::TabulationStop              => 18,
			Mode::GraphicRenditionCombination => 21,
			Mode::ZeroDefault                 => 22,
		}
	}
}
