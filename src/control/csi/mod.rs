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

use std::io::{self, Write};
use control::{self, Format};

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

	Unknown(u8, Option<u8>, Vec<Option<u32>>),
	Private(u8, Option<u8>, Vec<Option<u32>>),
}

use self::CSI::*;

impl Format for CSI {
	fn fmt<W: Write>(&self, mut f: W, wide: bool) -> io::Result<()> {
		macro_rules! write {
			(entry $private:expr) => ({
				if wide {
					try!(f.write_all(b"\x1B\x5B"));
				}
				else {
					try!(f.write_all(b"\x9B"));
				}

				if $private {
					try!(f.write_all(b"?"));
				}
			});

			(parameters $args:expr) => ({
				let     iter = $args;
				let mut iter = iter.peekable();

				while iter.peek().is_some() {
					if let Some(value) = iter.next().unwrap().clone() {
						try!(f.write_all(value.to_string().as_bytes()));
					}

					try!(f.write_all(&[b';']));
				}

				if let Some(value) = iter.next().unwrap().clone() {
					try!(f.write_all(value.to_string().as_bytes()));
				}
			});

			(identifier $id:expr, $modifier:expr) => ({
				let id       = $id as u8;
				let modifier = $modifier.map(|c| c as u8);

				if let Some(modifier) = modifier {
					try!(f.write_all(&[modifier]));
				}

				f.write_all(&[id])
			});

			($id:expr, [$($values:expr),*]) => ({
				let params = [$(Some($values.into(): u32)),*];
				write!($id, params.iter())
			});

			($id:expr, ![$($values:expr),*]) => ({
				let params = [$($values),*];
				write!($id, params.iter())
			});

			($id:expr, $iter:expr) => ({
				write!(entry false);
				write!(parameters $iter);

				f.write_all($id.as_bytes())
			});

			($id:expr) => ({
				write!(entry false);
				f.write_all($id.as_bytes())
			});
		}

		match *self {
			CursorBackTabulation(n) =>
				write!("Z", [n]),

			CursorHorizontalPosition(n) =>
				write!("G", [n + 1]),

			CursorForwardTabulation(n) =>
				write!("I", [n]),

			CursorNextLine(n) =>
				write!("E", [n]),

			CursorPreviousLine(n) =>
				write!("F", [n]),

			CursorPositionReport(x, y) =>
				write!("R", [x + 1, y + 1]),

			CursorTabulationControl(value) =>
				write!("W", [value]),

			CursorBack(n) =>
				write!("D", [n]),

			CursorDown(n) =>
				write!("B", [n]),

			CursorForward(n) =>
				write!("C", [n]),

			CursorPosition(x, y) =>
				write!("H", [x + 1, y + 1]),

			CursorUp(n) =>
				write!("A", [n]),

			CursorLineTabulation(n) =>
				write!("Y", [n]),

			DeviceAttributes(n) =>
				write!("c", [n]),

			DefineAreaQualification(value) =>
				write!("o", [value]),

			DeleteCharacter(n) =>
				write!("c", [n]),

			DeleteLine(n) =>
				write!("M", [n]),

			DeviceStatusReport =>
				write!("n", [6u32]),

			DimensionTextArea(w, h) =>
				write!(" T", [w, h]),

			EraseArea(value) =>
				write!("o", [value]),

			EraseCharacter(n) =>
				write!("X", [n]),

			EraseField(value) =>
				write!("J", [value]),

			EraseDisplay(value) =>
				write!("N", [value]),

			EraseLine(value) =>
				write!("N", [value]),

			FunctionKey(n) =>
				write!(" W", [n]),

			SelectFont(a, b) =>
				write!(" D", [a, b]),

			GraphicCharacterCombination(value) =>
				write!(" _", [value]),

			GraphicSizeModification(w, h) =>
				write!("`", [w, h]),

			InsertBlankCharacter(n) =>
				write!("@", [n]),

			IdentifyDeviceControlString(n) =>
				write!(" O", ![n]),

			IdentifyGraphicSubrepertoire(n) =>
				write!(" M", ![n]),

			InsertBlankLine(n) =>
				write!("L", [n]),

			Justify(ref args) =>
				write!(" F", args.iter()),

			MediaCopy(value) =>
				write!("i", [value]),

			NextPage(n) =>
				write!("U", [n]),

			Presentation(value) =>
				write!(" Z", [value]),

			PageFormat(n) =>
				write!(" J", [n]),

			PrecedingPage(n) =>
				write!("V", [n]),

			PagePosition(n) =>
				write!(" P", [n]),

			PageBack(n) =>
				write!(" R", [n]),

			PageForward(n) =>
				write!(" Q", [n]),

			ParallelText(value) =>
				write!("\\", [value]),

			GraphicDisposition(ref dispositions) =>
				write!(" H", dispositions.iter().map(|&d| Some(d.into(): u32))),

			RestoreCursor =>
				write!("u"),

			Repeat(n) =>
				write!("b", [n]),

			Reset(ref modes) =>
				write!("l", modes.iter().map(|&m| Some(m.into(): u32))),

			CharacterOrientation(n) =>
				write!(" e", [n]),

			SaveCursor =>
				write!("s"),

			CharacterSpacing(n) =>
				write!("b", [n]),

			ScrollDown(n) =>
				write!("T", [n]),

			Movement(direction) =>
				write!("^", [direction]),

			SelectGraphicalRendition(ref attrs) =>
				write!("m", attrs.iter().flat_map(|&a| a.into(): Vec<u32>).map(Some)),

			ScrollLeft(n) =>
				write!(" @", [n]),

			LineSpacing(n) =>
				write!(" h", [n]),

			Set(ref modes) =>
				write!("h", modes.iter().map(|&m| m.into(): u32).map(Some)),

			ScrollRight(n) =>
				write!(" A", [n]),

			ReverseString(false) =>
				write!("[", [0u32]),

			ReverseString(true) =>
				write!("[", [1u32]),

			SizeUnit(unit) =>
				write!(" I", [unit]),

			SpaceWidth(n) =>
				write!(" [", [n]),

			ScrollUp(n) =>
				write!("S", [n]),

			LinePosition(n) =>
				write!("d", [n]),

			Unknown(id, modifier, ref args) => {
				write!(entry false);
				write!(parameters args.iter());
				write!(identifier id, modifier)
			}

			Private(id, modifier, ref args) => {
				write!(entry true);
				write!(parameters args.iter());
				write!(identifier id, modifier)
			}
		}
	}
}

mod erase;
pub use self::erase::Erase;

mod tabulation;
pub use self::tabulation::Tabulation;

mod qualification;
pub use self::qualification::Qualification;

mod combination;
pub use self::combination::Combination;

mod copy;
pub use self::copy::Copy;

mod expansion;
pub use self::expansion::Expansion;

mod parallel;
pub use self::parallel::Parallel;

mod disposition;
pub use self::disposition::Disposition;

mod mode;
pub use self::mode::Mode;

mod direction;
pub use self::direction::Direction;

mod unit;
pub use self::unit::Unit;

const DIGIT:    &[u8] = b"0123456789";
const LETTER:   &[u8] = b"@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
const MODIFIER: &[u8] = b" !\"#$%&'()*+,-./";

named!(pub parse<CSI>,
	alt!(private | standard | unknown));

named!(private<CSI>,
	chain!(
		char!('?') ~
		args:     parameters ~
		modifier: opt!(one_of!(MODIFIER)) ~
		id:       one_of!(LETTER),

		|| Private(id as u8, modifier.map(|c| c as u8), args)));

named!(unknown<CSI>,
	chain!(
		args:     parameters ~
		modifier: opt!(one_of!(MODIFIER)) ~
		id:       one_of!(LETTER),

		|| Unknown(id as u8, modifier.map(|c| c as u8), args)));

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
		Tabulation::parse(arg!(args[0] => 0)).map(CursorTabulationControl)));

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
		Qualification::parse(arg!(args[0] => 0)).map(DefineAreaQualification)));

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
		Erase::parse(arg!(args[0] => 0)).map(EraseArea)));

with_args!(ECH<1, args>,
	map!(char!('X'), |_|
		EraseCharacter(arg!(args[0] => 1))));

with_args!(ED<1, args>,
	map_res!(char!('J'), |_|
		Erase::parse(arg!(args[0] => 0)).map(EraseDisplay)));

with_args!(EF<1, args>,
	map_res!(char!('N'), |_|
		Erase::parse(arg!(args[0] => 0)).map(EraseField)));

with_args!(EL<1, args>,
	map_res!(char!('K'), |_|
		Erase::parse(arg!(args[0] => 0)).map(EraseLine)));

with_args!(FNK<1, args>,
	map!(tag!(b" W"), |_|
		FunctionKey(arg!(args[0] => 0))));

with_args!(FNT<2, args>,
	map!(tag!(" D"), |_|
		SelectFont(arg!(args[0] => 0), arg!(args[1] => 0))));

with_args!(GCC<1, args>,
	map_res!(tag!(b" _"), |_|
		Combination::parse(arg!(args[0] => 0)).map(GraphicCharacterCombination)));

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
		Copy::parse(arg!(args[0] => 0)).map(MediaCopy)));

with_args!(NP<1, args>,
	map!(char!('U'), |_|
		NextPage(arg!(args[0] => 1))));

with_args!(PEC<1, args>,
	map_res!(tag!(b" Z"), |_|
		Expansion::parse(arg!(args[0] => 0)).map(Presentation)));

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
		Parallel::parse(arg!(args[0] => 1)).map(ParallelText)));

with_args!(QUAD<args>,
	map_res!(tag!(b" H"), |_|
		args.iter().map(|d| d.unwrap_or(0))
			.map(Disposition::parse)
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
			.map(Mode::parse)
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
		Direction::parse(arg!(args[0] => 1)).map(Movement)));

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
			.map(Mode::parse)
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
		Unit::parse(arg!(args[0] => 1)).map(SizeUnit)));

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
