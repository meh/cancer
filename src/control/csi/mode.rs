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
