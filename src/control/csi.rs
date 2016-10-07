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

use std::str;
use std::u32;
use nom::{self, ErrorKind, is_digit};

use control;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum CSI {
	CursorBackTabulation(u32),
	CursorHorizontalPosition(u32),
	CursorForwardTabulation(u32),
	CursorNextLine(u32),
	CursorPreviousLine(u32),
	CursorPositionReport(u32, u32),
	CursorTabulationControl(Tabulation),
	CursorBack(u32),
	CursorDown(u32),
	CursorForward(u32),
	CursorPosition(u32, u32),
	CursorUp(u32),
	CursorLineTabulation(u32),
	DeviceAttributes(u32),
	DefineAreaQualification(Qualification),
	DeleteCharacter(u32),
	DeleteLine(u32),
	DeviceStatusReport,
	DimensionTextArea(u32, u32),
	EraseArea(Erase),
	EraseCharacter(u32),
	EraseDisplay(Erase),
	EraseField(Erase),
	EraseLine(Erase),
	FunctionKey(u32),
	SelectFont(u32, u32),
	GraphicCharacterCombination(Combination),
	GraphicSizeModification(u32, u32),
	InsertBlankCharacter(u32),
	IdentifyDeviceControlString(Option<u32>),
	IdentifyGraphicSubrepertoire(Option<u32>),
	InsertBlankLine(u32),
	Justify(Vec<Option<u32>>),
	MediaCopy(Copy),
	NextPage(u32),
	Presentation(Expansion),
	PageFormat(u32),
	PrecedingPage(u32),
	PagePosition(u32),
	PageBack(u32),
	PageForward(u32),
	ParallelText(Parallel),
	GraphicDisposition(Vec<Disposition>),
	RestoreCursor,
	Repeat(u32),
	Reset(Vec<Mode>),
	CharacterOrientation(u16),
	SaveCursor,
	CharacterSpacing(u32),
	ScrollDown(u32),
	Movement(Direction),
	SelectGraphicalRendition(Vec<control::SGR::T>),
	ScrollLeft(u32),
	LineSpacing(u32),
	Set(Vec<Mode>),
	ScrollRight(u32),
	ReverseString(bool),
	SizeUnit(Unit),
	SpaceWidth(u32),
	ScrollUp(u32),
	LinePosition(u32),
	LineForward(u32),

	Unknown(char, Option<char>, Vec<Option<u32>>),
	Private(char, Option<char>, Vec<Option<u32>>),
}

use self::CSI::*;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Erase {
	ToEnd,
	ToStart,
	All,
}

impl Erase {
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Erase::ToEnd),
			1 => Ok(Erase::ToStart),
			2 => Ok(Erase::All),
			_ => Err(nom::Err::Code(ErrorKind::Custom(9002))),
		}
	}
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Tabulation {
	Character,
	Line,
	ClearCharacter,
	ClearLine,
	ClearLineAllCharacter,
	ClearAllCharacter,
	ClearAllLine,
}

impl Tabulation {
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Tabulation::Character),
			1 => Ok(Tabulation::Line),
			2 => Ok(Tabulation::ClearCharacter),
			3 => Ok(Tabulation::ClearLine),
			4 => Ok(Tabulation::ClearLineAllCharacter),
			5 => Ok(Tabulation::ClearAllCharacter),
			6 => Ok(Tabulation::ClearAllLine),
			_ => Err(nom::Err::Code(ErrorKind::Custom(9003))),
		}
	}
}

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
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
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
			_  => Err(nom::Err::Code(ErrorKind::Custom(9004))),
		}
	}
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Combination {
	Next,
	Start,
	End,
}

impl Combination {
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Combination::Next),
			1 => Ok(Combination::Start),
			2 => Ok(Combination::End),
			_ => Err(nom::Err::Code(ErrorKind::Custom(9005))),
		}
	}
}

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
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Copy::ToPrimary),
			1 => Ok(Copy::FromPrimary),
			2 => Ok(Copy::ToSecondary),
			3 => Ok(Copy::FromSecondary),
			4 => Ok(Copy::StopPrimary),
			5 => Ok(Copy::StartPrimary),
			6 => Ok(Copy::StopSecondary),
			7 => Ok(Copy::StartSecondary),
			_ => Err(nom::Err::Code(ErrorKind::Custom(9005))),
		}
	}
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Expansion {
	Normal,
	Expanded,
	Condensed,
}

impl Expansion {
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Expansion::Normal),
			1 => Ok(Expansion::Expanded),
			2 => Ok(Expansion::Condensed),
			_ => Err(nom::Err::Code(ErrorKind::Custom(9002))),
		}
	}
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Parallel {
	End,
	Start,
	StartSupplementary,
	StartPhoneticJapanese,
	StartPhoneticChinese,
	StopPhonetic,
}

impl Parallel {
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Parallel::End),
			1 => Ok(Parallel::Start),
			2 => Ok(Parallel::StartSupplementary),
			3 => Ok(Parallel::StartPhoneticJapanese),
			4 => Ok(Parallel::StartPhoneticChinese),
			5 => Ok(Parallel::StopPhonetic),
			_ => Err(nom::Err::Code(ErrorKind::Custom(9002))),
		}
	}
}

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
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Disposition::ToHome),
			1 => Ok(Disposition::ToHomeWithLeader),
			2 => Ok(Disposition::Center),
			3 => Ok(Disposition::CenterWithLeader),
			4 => Ok(Disposition::ToLimit),
			5 => Ok(Disposition::ToLimitWithLeader),
			6 => Ok(Disposition::ToBoth),
			_ => Err(nom::Err::Code(ErrorKind::Custom(9002))),
		}
	}
}

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
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
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
			_  => Err(nom::Err::Code(ErrorKind::Custom(9004))),
		}
	}
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Direction {
	Forward,
	Backward,
}

impl Direction {
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
		match value {
			0 => Ok(Direction::Forward),
			1 => Ok(Direction::Backward),
			_ => Err(nom::Err::Code(ErrorKind::Custom(9002))),
		}
	}
}

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
	fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
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
			_ => Err(nom::Err::Code(ErrorKind::Custom(9002))),
		}
	}
}

const DIGIT:    &[u8] = b"0123456789";
const LETTER:   &[u8] = b"@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
const MODIFIER: &[u8] = b" !\"\\#$%&'()*+,-./";

named!(pub parse<CSI>,
	alt!(private | standard | unknown));

named!(private<CSI>,
	chain!(
		char!('?') ~
		args:     parameters ~
		modifier: opt!(one_of!(MODIFIER)) ~
		id:       one_of!(LETTER),

		|| Private(id as char, modifier.map(|c| c as char), args)));

named!(unknown<CSI>,
	chain!(
		args:     parameters ~
		modifier: opt!(one_of!(MODIFIER)) ~
		id:       one_of!(LETTER),

		|| Unknown(id as char, modifier.map(|c| c as char), args)));

// TODO(meh): reorder them by most common occurrence
named!(standard<CSI>,
	chain!(
		args: parameters ~
		res:  alt!(apply!(CBT,  &args) |
		           apply!(CHA,  &args) |
		           apply!(CHT,  &args) |
		           apply!(CNL,  &args) |
		           apply!(CPL,  &args) |
		           apply!(CPR,  &args) |
		           apply!(CTC,  &args) |
		           apply!(CUB,  &args) |
		           apply!(CUD,  &args) |
		           apply!(CUF,  &args) |
		           apply!(CUP,  &args) |
		           apply!(CUU,  &args) |
		           apply!(CVT,  &args) |
		           apply!(DA,   &args) |
		           apply!(DAQ,  &args) |
		           apply!(DCH,  &args) |
		           apply!(DL,   &args) |
		           apply!(DSR,  &args) |
		           apply!(DTA,  &args) |
		           apply!(EA,   &args) |
		           apply!(ECH,  &args) |
		           apply!(ED,   &args) |
		           apply!(EF,   &args) |
		           apply!(EL,   &args) |
		           apply!(FNK,  &args) |
		           apply!(FNT,  &args) |
		           apply!(GCC,  &args) |
		           apply!(GSM,  &args) |
		           apply!(HPA,  &args) |
		           apply!(HPB,  &args) |
		           apply!(HPR,  &args) |
		           apply!(HVP,  &args) |
		           apply!(ICH,  &args) |
		           apply!(IDCS, &args) |
		           apply!(IGS,  &args) |
		           apply!(IL,   &args) |
		           apply!(JFY,  &args) |
		           apply!(MC,   &args) |
		           apply!(NP,   &args) |
		           apply!(PEC,  &args) |
		           apply!(PFS,  &args) |
		           apply!(PP,   &args) |
		           apply!(PPA,  &args) |
		           apply!(PPB,  &args) |
		           apply!(PPR,  &args) |
		           apply!(PTX,  &args) |
		           apply!(QUAD, &args) |
		           apply!(RCP,  &args) |
		           apply!(REP,  &args) |
		           apply!(RM,   &args) |
		           apply!(SCO,  &args) |
		           apply!(SCP,  &args) |
		           apply!(SCS,  &args) |
		           apply!(SD,   &args) |
		           apply!(SIMD, &args) |
		           apply!(SGR,  &args) |
		           apply!(SL,   &args) |
		           apply!(SLS,  &args) |
		           apply!(SM,   &args) |
		           apply!(SR,   &args) |
		           apply!(SRS,  &args) |
		           apply!(SSU,  &args) |
		           apply!(SSW,  &args) |
		           apply!(SU,   &args) |
		           apply!(VPA,  &args) |
		           apply!(VPB,  &args) |
		           apply!(VPR,  &args)),

		|| res));

named!(parameters<Vec<Option<u32>>>,
	many0!(parameter));

named!(parameter<Option<u32> >,
	alt!(
		char!(';') => { |_| None } |
		chain!(
			number: is_a!(DIGIT) ~
			opt!(char!(';')),

			|| number) => { |n|
				Some(u32::from_str_radix(unsafe { str::from_utf8_unchecked(n) }, 10).unwrap()) }));

macro_rules! with_args {
	($name:ident<$n:tt, $params:ident>, $submac:ident!( $($args:tt)* )) => (
		fn $name<'a, 'b>(i: &'a [u8], $params: &'b [Option<u32>]) -> nom::IResult<&'a [u8], CSI> {
			if $params.len() <= $n {
				$submac!(i, $($args)*)
			}
			else {
				nom::IResult::Error(nom::Err::Code(ErrorKind::Custom(9001)))
			}
		}
	);

	($name:ident<$params:ident>, $submac:ident!( $($args:tt)* )) => (
		fn $name<'a, 'b>(i: &'a [u8], $params: &'b [Option<u32>]) -> nom::IResult<&'a [u8], CSI> {
			$submac!(i, $($args)*)
		}
	);

	($name:ident, $submac:ident!( $($args:tt)* )) => (
		fn $name<'a, 'b>(i: &'a [u8], args: &'b [Option<u32>]) -> nom::IResult<&'a [u8], CSI> {
			if args.is_empty() {
				$submac!(i, $($args)*)
			}
			else {
				nom::IResult::Error(nom::Err::Code(ErrorKind::Custom(9001)))
			}
		}
	);
}

with_args!(CBT<1, args>,
	map!(char!('Z'), |_|
		CursorBackTabulation(arg!(args[0] => 1))));

with_args!(CHA<1, args>,
	map!(char!('G'), |_|
		CursorHorizontalPosition(arg!(args[0] => 1) - 1)));

with_args!(CHT<1, args>,
	map!(char!('I'), |_|
		CursorForwardTabulation(arg!(args[0] => 1))));

with_args!(CNL<1, args>,
	map!(char!('E'), |_|
		CursorNextLine(arg!(args[0] => 1))));

with_args!(CPL<1, args>,
	map!(char!('F'), |_|
		CursorPreviousLine(arg!(args[0] => 1))));

with_args!(CPR<2, args>,
	map!(char!('R'), |_|
		CursorPositionReport(arg!(args[0] => 1) - 1, arg!(args[1] => 1) - 1)));

with_args!(CTC<1, args>,
	map_res!(char!('W'), |_|
		Tabulation::from(arg!(args[0] => 0)).map(CursorTabulationControl)));

with_args!(CUB<1, args>,
	map!(char!('D'), |_|
		CursorBack(arg!(args[0] => 1))));

with_args!(CUD<1, args>,
	map!(char!('B'), |_|
		CursorDown(arg!(args[0] => 1))));

with_args!(CUF<1, args>,
	map!(char!('C'), |_|
		CursorForward(arg!(args[0] => 1))));

with_args!(CUP<2, args>,
	map!(char!('H'), |_|
		CursorPosition(arg!(args[0] => 1) - 1, arg!(args[1] => 1) - 1)));

with_args!(CUU<1, args>,
	map!(char!('A'), |_|
		CursorUp(arg!(args[0] => 1))));

with_args!(CVT<1, args>,
	map!(char!('Y'), |_|
		CursorLineTabulation(arg!(args[0] => 1))));

with_args!(DA<1, args>,
	map!(char!('c'), |_|
		DeviceAttributes(arg!(args[0] => 0))));

with_args!(DAQ<1, args>,
	map_res!(char!('o'), |_|
		Qualification::from(arg!(args[0] => 0)).map(DefineAreaQualification)));

with_args!(DCH<1, args>,
	map!(char!('c'), |_|
		DeleteCharacter(arg!(args[0] => 1))));

with_args!(DL<1, args>,
	map!(char!('M'), |_|
		DeleteLine(arg!(args[0] => 1))));

with_args!(DSR<1, args>,
	map_res!(char!('n'), |_|
		match arg!(args[0] => 0) {
			6 =>
				Ok(DeviceStatusReport),

			_ =>
				Err(nom::Err::Code::<&[u8], u32>(ErrorKind::Custom(9004)))
		}));

with_args!(DTA<2, args>,
	map!(tag!(b" T"), |_|
		DimensionTextArea(arg!(args[0] => 0), arg!(args[1] => 0))));

with_args!(EA<1, args>,
	map_res!(char!('o'), |_|
		Erase::from(arg!(args[0] => 0)).map(EraseArea)));

with_args!(ECH<1, args>,
	map!(char!('X'), |_|
		EraseCharacter(arg!(args[0] => 1))));

with_args!(ED<1, args>,
	map_res!(char!('J'), |_|
		Erase::from(arg!(args[0] => 0)).map(EraseDisplay)));

with_args!(EF<1, args>,
	map_res!(char!('N'), |_|
		Erase::from(arg!(args[0] => 0)).map(EraseField)));

with_args!(EL<1, args>,
	map_res!(char!('K'), |_|
		Erase::from(arg!(args[0] => 0)).map(EraseLine)));

with_args!(FNK<1, args>,
	map!(tag!(b" W"), |_|
		FunctionKey(arg!(args[0] => 0))));

with_args!(FNT<2, args>,
	map!(tag!(" D"), |_|
		SelectFont(arg!(args[0] => 0), arg!(args[1] => 0))));

with_args!(GCC<1, args>,
	map_res!(tag!(b" _"), |_|
		Combination::from(arg!(args[0] => 0)).map(GraphicCharacterCombination)));

with_args!(GSM<2, args>,
	map!(tag!(b" B"), |_|
		GraphicSizeModification(arg!(args[1] => 100), arg!(args[0] => 100))));

with_args!(HPA<1, args>,
	map!(char!('`'), |_|
		CursorHorizontalPosition(arg!(args[0] => 1) - 1)));

with_args!(HPB<1, args>,
	map!(char!('j'), |_|
		CursorBack(arg!(args[0] => 1))));

with_args!(HPR<1, args>,
	map!(char!('a'), |_|
		CursorForward(arg!(args[0] => 1))));

with_args!(HVP<2, args>,
	map!(char!('f'), |_|
		CursorPosition(arg!(args[0] => 1) - 1, arg!(args[1] => 1) - 1)));

with_args!(ICH<1, args>,
	map!(char!('@'), |_|
		InsertBlankCharacter(arg!(args[0] => 1))));

with_args!(IDCS<1, args>,
	map!(tag!(b" O"), |_|
		IdentifyDeviceControlString(arg!(args[0]))));

with_args!(IGS<1, args>,
	map!(tag!(b" M"), |_|
		IdentifyGraphicSubrepertoire(arg!(args[0]))));

with_args!(IL<1, args>,
	map!(char!('L'), |_|
		InsertBlankLine(arg!(args[0] => 1))));

with_args!(JFY<args>,
	map!(tag!(b" F"), |_|
		Justify(args.to_vec())));

with_args!(MC<1, args>,
	map_res!(char!('i'), |_|
		Copy::from(arg!(args[0] => 0)).map(MediaCopy)));

with_args!(NP<1, args>,
	map!(char!('U'), |_|
		NextPage(arg!(args[0] => 1))));

with_args!(PEC<1, args>,
	map_res!(tag!(b" Z"), |_|
		Expansion::from(arg!(args[0] => 0)).map(Presentation)));

with_args!(PFS<1, args>,
	map!(tag!(b" J"), |_|
		PageFormat(arg!(args[0] => 0))));

with_args!(PP<1, args>,
	map!(char!('V'), |_|
		PrecedingPage(arg!(args[0] => 1))));

with_args!(PPA<1, args>,
	map!(tag!(b" P"), |_|
		PagePosition(arg!(args[0] => 1))));

with_args!(PPB<1, args>,
	map!(tag!(b" R"), |_|
		PageBack(arg!(args[0] => 1))));

with_args!(PPR<1, args>,
	map!(tag!(b" Q"), |_|
		PageForward(arg!(args[0] => 1))));

with_args!(PTX<1, args>,
	map_res!(char!('\\'), |_|
		Parallel::from(arg!(args[0] => 1)).map(ParallelText)));

with_args!(QUAD<args>,
	map_res!(tag!(b" H"), |_|
		args.iter().map(|d| d.unwrap_or(0))
			.map(Disposition::from)
			.collect::<Result<Vec<_>, _>>()
			.map(GraphicDisposition)));

with_args!(RCP,
	map!(char!('u'), |_|
		RestoreCursor));

with_args!(REP<1, args>,
	map!(char!('b'), |_|
		Repeat(arg!(args[0] => 1))));

with_args!(RM<args>,
	map_res!(char!('l'), |_|
		args.iter().map(|d| d.unwrap_or(0))
			.map(Mode::from)
			.collect::<Result<Vec<_>, _>>()
			.map(Reset)));

with_args!(SCO<1, args>,
	map_res!(tag!(b" e"), |_|
		match arg!(args[0] => 0) {
			0 => Ok(CharacterOrientation(0)),
			1 => Ok(CharacterOrientation(45)),
			2 => Ok(CharacterOrientation(90)),
			3 => Ok(CharacterOrientation(135)),
			4 => Ok(CharacterOrientation(180)),
			5 => Ok(CharacterOrientation(225)),
			6 => Ok(CharacterOrientation(270)),
			7 => Ok(CharacterOrientation(315)),
			_ => Err(nom::Err::Code::<&[u8], u32>(ErrorKind::Custom(9002))),
		}));

with_args!(SCP,
	map!(char!('s'), |_|
		SaveCursor));

with_args!(SCS<1, args>,
	map!(char!('b'), |_|
		CharacterSpacing(arg!(args[0] => 1))));

with_args!(SD<1, args>,
	map!(char!('T'), |_|
		ScrollDown(arg!(args[0] => 1))));

with_args!(SIMD<1, args>,
	map_res!(char!('^'), |_|
		Direction::from(arg!(args[0] => 1)).map(Movement)));

with_args!(SGR<args>,
	map_res!(char!('m'), |_|
		control::SGR::parse(args).map(|v| SelectGraphicalRendition(v))));

with_args!(SL<1, args>,
	map!(tag!(b" @"), |_|
		ScrollLeft(arg!(args[0] => 1))));

with_args!(SLS<1, args>,
	map!(tag!(b" h"), |_|
		LineSpacing(arg!(args[0] => 1))));

with_args!(SM<args>,
	map_res!(char!('h'), |_|
		args.iter().map(|d| d.unwrap_or(0))
			.map(Mode::from)
			.collect::<Result<Vec<_>, _>>()
			.map(Set)));

with_args!(SR<1, args>,
	map!(tag!(b" A"), |_|
		ScrollRight(arg!(args[0] => 1))));

with_args!(SRS<1, args>,
	map_res!(tag!(b"["), |_|
		match arg!(args[0] => 0) {
			0 => Ok(ReverseString(false)),
			1 => Ok(ReverseString(true)),
			_ => Err(nom::Err::Code::<&[u8], u32>(ErrorKind::Custom(9002))),
		}));

with_args!(SSU<1, args>,
	map_res!(tag!(b" I"), |_|
		Unit::from(arg!(args[0] => 1)).map(SizeUnit)));

with_args!(SSW<1, args>,
	map!(tag!(b" ["), |_|
		SpaceWidth(arg!(args[0] => 1))));

with_args!(SU<1, args>,
	map!(char!('S'), |_|
		ScrollUp(arg!(args[0] => 1))));

with_args!(VPA<1, args>,
	map!(char!('d'), |_|
		LinePosition(arg!(args[0] => 1))));

with_args!(VPB<1, args>,
	map!(char!('k'), |_|
		CursorUp(arg!(args[0] => 1))));

with_args!(VPR<1, args>,
	map!(char!('e'), |_|
		CursorDown(arg!(args[0] => 1))));

pub mod shim {
	pub use super::CSI as T;
	pub use super::CSI::*;
	pub use super::parse;
	pub use super::{Erase, Tabulation, Qualification, Combination};
}

#[cfg(test)]
mod test {
	pub use control::*;

	#[test]
	fn ich() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::InsertBlankCharacter(1))),
			parse(b"\x1B[@").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::InsertBlankCharacter(23))),
			parse(b"\x1B[23@").unwrap().1);
	}

	#[test]
	fn cuu() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorUp(1))),
			parse(b"\x1B[A").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorUp(23))),
			parse(b"\x1B[23A").unwrap().1);
	}

	#[test]
	fn cud() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorDown(1))),
			parse(b"\x1B[B").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorDown(42))),
			parse(b"\x1B[42B").unwrap().1);
	}

	#[test]
	fn vpr() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorDown(1))),
			parse(b"\x1B[e").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorDown(42))),
			parse(b"\x1B[42e").unwrap().1);
	}

	#[test]
	fn cuf() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorForward(1))),
			parse(b"\x1B[C").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorForward(13))),
			parse(b"\x1B[13C").unwrap().1);
	}

	#[test]
	fn cub() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorBack(1))),
			parse(b"\x1B[D").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorBack(37))),
			parse(b"\x1B[37D").unwrap().1);
	}

	#[test]
	fn cnl() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorNextLine(1))),
			parse(b"\x1B[E").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorNextLine(12))),
			parse(b"\x1B[12E").unwrap().1);
	}

	#[test]
	fn cpl() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPreviousLine(1))),
			parse(b"\x1B[F").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPreviousLine(43))),
			parse(b"\x1B[43F").unwrap().1);
	}

	#[test]
	fn cha() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorHorizontalPosition(0))),
			parse(b"\x1B[G").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorHorizontalPosition(42))),
			parse(b"\x1B[43G").unwrap().1);
	}

	#[test]
	fn cup() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPosition(0, 0))),
			parse(b"\x1B[H").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPosition(1, 2))),
			parse(b"\x1B[2;3H").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPosition(0, 2))),
			parse(b"\x1B[;3H").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPosition(1, 0))),
			parse(b"\x1B[2;H").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPosition(0, 0))),
			parse(b"\x1B[;H").unwrap().1);
	}

	#[test]
	fn hvp() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPosition(0, 0))),
			parse(b"\x1B[f").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPosition(1, 2))),
			parse(b"\x1B[2;3f").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPosition(0, 2))),
			parse(b"\x1B[;3f").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPosition(1, 0))),
			parse(b"\x1B[2;f").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::CursorPosition(0, 0))),
			parse(b"\x1B[;f").unwrap().1);
	}

	#[test]
	fn ed() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::EraseDisplay(CSI::Erase::ToEnd))),
			parse(b"\x1B[J").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::EraseDisplay(CSI::Erase::ToEnd))),
			parse(b"\x1B[0J").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::EraseDisplay(CSI::Erase::ToStart))),
			parse(b"\x1B[1J").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::EraseDisplay(CSI::Erase::All))),
			parse(b"\x1B[2J").unwrap().1);
	}

	#[test]
	fn el() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::EraseLine(CSI::Erase::ToEnd))),
			parse(b"\x1B[K").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::EraseLine(CSI::Erase::ToEnd))),
			parse(b"\x1B[0K").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::EraseLine(CSI::Erase::ToStart))),
			parse(b"\x1B[1K").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::EraseLine(CSI::Erase::All))),
			parse(b"\x1B[2K").unwrap().1);
	}

	#[test]
	fn su() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::ScrollUp(1))),
			parse(b"\x1B[S").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::ScrollUp(37))),
			parse(b"\x1B[37S").unwrap().1);
	}

	#[test]
	fn sd() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::ScrollDown(1))),
			parse(b"\x1B[T").unwrap().1);

		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::ScrollDown(73))),
			parse(b"\x1B[73T").unwrap().1);
	}

	#[test]
	fn dsr() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::DeviceStatusReport)),
			parse(b"\x1B[6n").unwrap().1);
	}

	#[test]
	fn scp() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::SaveCursor)),
			parse(b"\x1B[s").unwrap().1);
	}

	#[test]
	fn rcp() {
		assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
			CSI::RestoreCursor)),
			parse(b"\x1B[u").unwrap().1);
	}
}
