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

				if let Some(value) = iter.next() {
					if let Some(value) = value.clone() {
						try!(f.write_all(value.to_string().as_bytes()));
					}
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
				write!("P", [n]),

			DeleteLine(n) =>
				write!("M", [n]),

			DeviceStatusReport =>
				write!("n", [6u32]),

			DimensionTextArea(w, h) =>
				write!(" T", [w, h]),

			EraseArea(value) =>
				write!("O", [value]),

			EraseCharacter(n) =>
				write!("X", [n]),

			EraseField(value) =>
				write!("N", [value]),

			EraseDisplay(value) =>
				write!("J", [value]),

			EraseLine(value) =>
				write!("K", [value]),

			FunctionKey(n) =>
				write!(" W", [n]),

			SelectFont(a, b) =>
				write!(" D", [a, b]),

			GraphicCharacterCombination(value) =>
				write!(" _", [value]),

			GraphicSizeModification(w, h) =>
				write!(" B", [h, w]),

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
				write!(" e", [match n {
					0   => 0u32,
					45  => 1u32,
					90  => 2u32,
					135 => 3u32,
					180 => 4u32,
					225 => 5u32,
					270 => 6u32,
					315 => 7u32,
					_   => unreachable!(),
				}]),

			SaveCursor =>
				write!("s"),

			CharacterSpacing(n) =>
				write!(" b", [n]),

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
	map!(char!('P'), |_|
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
	map_res!(char!('O'), |_|
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
		Parallel::parse(arg!(args[0] => 0)).map(ParallelText)));

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
	map!(tag!(b" b"), |_|
		CharacterSpacing(arg!(args[0] => 1))));

with_args!(SD<1, args>,
	map!(char!('T'), |_|
		ScrollDown(arg!(args[0] => 1))));

with_args!(SIMD<1, args>,
	map_res!(char!('^'), |_|
		Direction::parse(arg!(args[0] => 0)).map(Movement)));

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
		Unit::parse(arg!(args[0] => 0)).map(SizeUnit)));

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
	pub use super::{Erase, Tabulation, Qualification, Combination, Copy};
	pub use super::{Expansion, Parallel, Disposition, Mode, Direction, Unit};
}

#[cfg(test)]
mod test {
	mod parse {
		pub use control::*;

		macro_rules! test {
			($string:expr => $item:expr) => (
				assert_eq!(Item::C1(C1::ControlSequenceIntroducer($item)),
					parse($string).unwrap().1);
			);
		}

		#[test]
		fn cbt() {
			test!(b"\x1B[Z" =>
				CSI::CursorBackTabulation(1));

			test!(b"\x1B[23Z" =>
				CSI::CursorBackTabulation(23));
		}

		#[test]
		fn cha() {
			test!(b"\x1B[G" =>
				CSI::CursorHorizontalPosition(0));

			test!(b"\x1B[43G" =>
				CSI::CursorHorizontalPosition(42));
		}

		#[test]
		fn cht() {
			test!(b"\x1B[I" =>
				CSI::CursorForwardTabulation(1));

			test!(b"\x1B[23I" =>
				CSI::CursorForwardTabulation(23));
		}

		#[test]
		fn cnl() {
			test!(b"\x1B[E" =>
				CSI::CursorNextLine(1));

			test!(b"\x1B[12E" =>
				CSI::CursorNextLine(12));
		}

		#[test]
		fn cpl() {
			test!(b"\x1B[F" =>
				CSI::CursorPreviousLine(1));

			test!(b"\x1B[43F" =>
				CSI::CursorPreviousLine(43));
		}

		#[test]
		fn cpr() {
			test!(b"\x1B[R" =>
				CSI::CursorPositionReport(0, 0));

			test!(b"\x1B[2R" =>
				CSI::CursorPositionReport(1, 0));

			test!(b"\x1B[;2R" =>
				CSI::CursorPositionReport(0, 1));

			test!(b"\x1B[2;2R" =>
				CSI::CursorPositionReport(1, 1));
		}

		#[test]
		fn ctc() {
			test!(b"\x1B[W" =>
				CSI::CursorTabulationControl(CSI::Tabulation::Character));

			test!(b"\x1B[0W" =>
				CSI::CursorTabulationControl(CSI::Tabulation::Character));

			test!(b"\x1B[1W" =>
				CSI::CursorTabulationControl(CSI::Tabulation::Line));

			test!(b"\x1B[2W" =>
				CSI::CursorTabulationControl(CSI::Tabulation::ClearCharacter));

			test!(b"\x1B[3W" =>
				CSI::CursorTabulationControl(CSI::Tabulation::ClearLine));

			test!(b"\x1B[4W" =>
				CSI::CursorTabulationControl(CSI::Tabulation::ClearLineAllCharacters));

			test!(b"\x1B[5W" =>
				CSI::CursorTabulationControl(CSI::Tabulation::ClearAllCharacters));

			test!(b"\x1B[6W" =>
				CSI::CursorTabulationControl(CSI::Tabulation::ClearAllLines));
		}

		#[test]
		fn cub() {
			test!(b"\x1B[D" =>
				CSI::CursorBack(1));

			test!(b"\x1B[37D" =>
				CSI::CursorBack(37));
		}

		#[test]
		fn cud() {
			test!(b"\x1B[B" =>
				CSI::CursorDown(1));

			test!(b"\x1B[42B" =>
				CSI::CursorDown(42));
		}

		#[test]
		fn cuf() {
			test!(b"\x1B[C" =>
				CSI::CursorForward(1));

			test!(b"\x1B[13C" =>
				CSI::CursorForward(13));
		}

		#[test]
		fn cup() {
			test!(b"\x1B[H" =>
				CSI::CursorPosition(0, 0));

			test!(b"\x1B[2;3H" =>
				CSI::CursorPosition(1, 2));

			test!(b"\x1B[;3H" =>
				CSI::CursorPosition(0, 2));

			test!(b"\x1B[2;H" =>
				CSI::CursorPosition(1, 0));

			test!(b"\x1B[;H" =>
				CSI::CursorPosition(0, 0));
		}

		#[test]
		fn cuu() {
			test!(b"\x1B[A" =>
				CSI::CursorUp(1));

			test!(b"\x1B[23A" =>
				CSI::CursorUp(23));
		}

		#[test]
		fn cvt() {
			test!(b"\x1B[Y" =>
				CSI::CursorLineTabulation(1));

			test!(b"\x1B[23Y" =>
				CSI::CursorLineTabulation(23));
		}

		#[test]
		fn da() {
			test!(b"\x1B[c" =>
				CSI::DeviceAttributes(0));

			test!(b"\x1B[23c" =>
				CSI::DeviceAttributes(23));
		}

		#[test]
		fn daq() {
			test!(b"\x1B[o" =>
				CSI::DefineAreaQualification(CSI::Qualification::UnprotectedUnguarded));

			test!(b"\x1B[0o" =>
				CSI::DefineAreaQualification(CSI::Qualification::UnprotectedUnguarded));

			test!(b"\x1B[1o" =>
				CSI::DefineAreaQualification(CSI::Qualification::ProtectedGuarded));

			test!(b"\x1B[2o" =>
				CSI::DefineAreaQualification(CSI::Qualification::GraphicCharacterInput));

			test!(b"\x1B[3o" =>
				CSI::DefineAreaQualification(CSI::Qualification::NumericInput));

			test!(b"\x1B[4o" =>
				CSI::DefineAreaQualification(CSI::Qualification::AlphabeticInput));

			test!(b"\x1B[5o" =>
				CSI::DefineAreaQualification(CSI::Qualification::AlignLast));

			test!(b"\x1B[6o" =>
				CSI::DefineAreaQualification(CSI::Qualification::ZeroFill));

			test!(b"\x1B[7o" =>
				CSI::DefineAreaQualification(CSI::Qualification::FieldStart));

			test!(b"\x1B[8o" =>
				CSI::DefineAreaQualification(CSI::Qualification::ProtectedUnguarded));

			test!(b"\x1B[9o" =>
				CSI::DefineAreaQualification(CSI::Qualification::SpaceFill));

			test!(b"\x1B[10o" =>
				CSI::DefineAreaQualification(CSI::Qualification::AlignFirst));

			test!(b"\x1B[11o" =>
				CSI::DefineAreaQualification(CSI::Qualification::Reverse));
		}

		#[test]
		fn dch() {
			test!(b"\x1B[P" =>
				CSI::DeleteCharacter(1));

			test!(b"\x1B[8P" =>
				CSI::DeleteCharacter(8));
		}

		#[test]
		fn dl() {
			test!(b"\x1B[M" =>
				CSI::DeleteLine(1));

			test!(b"\x1B[8M" =>
				CSI::DeleteLine(8));
		}

		#[test]
		fn dsr() {
			test!(b"\x1B[6n" =>
				CSI::DeviceStatusReport);
		}

		#[test]
		fn dta() {
			test!(b"\x1B[ T" =>
				CSI::DimensionTextArea(0, 0));

			test!(b"\x1B[1 T" =>
				CSI::DimensionTextArea(1, 0));

			test!(b"\x1B[;1 T" =>
				CSI::DimensionTextArea(0, 1));

			test!(b"\x1B[1;1 T" =>
				CSI::DimensionTextArea(1, 1));
		}

		#[test]
		fn ea() {
			test!(b"\x1B[O" =>
				CSI::EraseArea(CSI::Erase::ToEnd));

			test!(b"\x1B[0O" =>
				CSI::EraseArea(CSI::Erase::ToEnd));

			test!(b"\x1B[1O" =>
				CSI::EraseArea(CSI::Erase::ToStart));

			test!(b"\x1B[2O" =>
				CSI::EraseArea(CSI::Erase::All));
		}

		#[test]
		fn ech() {
			test!(b"\x1B[X" =>
				CSI::EraseCharacter(1));

			test!(b"\x1B[8X" =>
				CSI::EraseCharacter(8));
		}

		#[test]
		fn ed() {
			test!(b"\x1B[J" =>
				CSI::EraseDisplay(CSI::Erase::ToEnd));

			test!(b"\x1B[0J" =>
				CSI::EraseDisplay(CSI::Erase::ToEnd));

			test!(b"\x1B[1J" =>
				CSI::EraseDisplay(CSI::Erase::ToStart));

			test!(b"\x1B[2J" =>
				CSI::EraseDisplay(CSI::Erase::All));
		}

		#[test]
		fn ef() {
			test!(b"\x1B[N" =>
				CSI::EraseField(CSI::Erase::ToEnd));

			test!(b"\x1B[0N" =>
				CSI::EraseField(CSI::Erase::ToEnd));

			test!(b"\x1B[1N" =>
				CSI::EraseField(CSI::Erase::ToStart));

			test!(b"\x1B[2N" =>
				CSI::EraseField(CSI::Erase::All));
		}

		#[test]
		fn el() {
			test!(b"\x1B[K" =>
				CSI::EraseLine(CSI::Erase::ToEnd));

			test!(b"\x1B[0K" =>
				CSI::EraseLine(CSI::Erase::ToEnd));

			test!(b"\x1B[1K" =>
				CSI::EraseLine(CSI::Erase::ToStart));

			test!(b"\x1B[2K" =>
				CSI::EraseLine(CSI::Erase::All));
		}

		#[test]
		fn fnk() {
			test!(b"\x1B[ W" =>
				CSI::FunctionKey(0));

			test!(b"\x1B[13 W" =>
				CSI::FunctionKey(13));
		}

		#[test]
		fn fnt() {
			test!(b"\x1B[ D" =>
				CSI::SelectFont(0, 0));

			test!(b"\x1B[13 D" =>
				CSI::SelectFont(13, 0));

			test!(b"\x1B[;13 D" =>
				CSI::SelectFont(0, 13));

			test!(b"\x1B[13;13 D" =>
				CSI::SelectFont(13, 13));
		}

		#[test]
		fn gcc() {
			test!(b"\x1B[ _" =>
				CSI::GraphicCharacterCombination(CSI::Combination::Next));

			test!(b"\x1B[0 _" =>
				CSI::GraphicCharacterCombination(CSI::Combination::Next));

			test!(b"\x1B[1 _" =>
				CSI::GraphicCharacterCombination(CSI::Combination::Start));

			test!(b"\x1B[2 _" =>
				CSI::GraphicCharacterCombination(CSI::Combination::End));
		}

		#[test]
		fn gsm() {
			test!(b"\x1B[ B" =>
				CSI::GraphicSizeModification(100, 100));

			test!(b"\x1B[13 B" =>
				CSI::GraphicSizeModification(100, 13));

			test!(b"\x1B[;13 B" =>
				CSI::GraphicSizeModification(13, 100));

			test!(b"\x1B[13;13 B" =>
				CSI::GraphicSizeModification(13, 13));
		}

		#[test]
		fn hpa() {
			test!(b"\x1B[`" =>
				CSI::CursorHorizontalPosition(0));

			test!(b"\x1B[2`" =>
				CSI::CursorHorizontalPosition(1));
		}

		#[test]
		fn hpb() {
			test!(b"\x1B[j" =>
				CSI::CursorBack(1));

			test!(b"\x1B[2j" =>
				CSI::CursorBack(2));
		}

		#[test]
		fn hpr() {
			test!(b"\x1B[a" =>
				CSI::CursorForward(1));

			test!(b"\x1B[2a" =>
				CSI::CursorForward(2));
		}

		#[test]
		fn hvp() {
			test!(b"\x1B[f" =>
				CSI::CursorPosition(0, 0));

			test!(b"\x1B[13f" =>
				CSI::CursorPosition(12, 0));

			test!(b"\x1B[;13f" =>
				CSI::CursorPosition(0, 12));

			test!(b"\x1B[13;13f" =>
				CSI::CursorPosition(12, 12));
		}

		#[test]
		fn ich() {
			test!(b"\x1B[@" =>
				CSI::InsertBlankCharacter(1));

			test!(b"\x1B[23@" =>
				CSI::InsertBlankCharacter(23));
		}

		#[test]
		fn idcs() {
			test!(b"\x1B[ O" =>
				CSI::IdentifyDeviceControlString(None));

			test!(b"\x1B[1 O" =>
				CSI::IdentifyDeviceControlString(Some(1)));
		}

		#[test]
		fn igs() {
			test!(b"\x1B[ M" =>
				CSI::IdentifyGraphicSubrepertoire(None));

			test!(b"\x1B[1 M" =>
				CSI::IdentifyGraphicSubrepertoire(Some(1)));
		}

		#[test]
		fn il() {
			test!(b"\x1B[L" =>
				CSI::InsertBlankLine(1));

			test!(b"\x1B[2L" =>
				CSI::InsertBlankLine(2));
		}

		#[test]
		fn jfy() {
			test!(b"\x1B[1;2 F" =>
				CSI::Justify(vec![Some(1), Some(2)]));
		}

		#[test]
		fn mc() {
			test!(b"\x1B[i" =>
				CSI::MediaCopy(CSI::Copy::ToPrimary));

			test!(b"\x1B[0i" =>
				CSI::MediaCopy(CSI::Copy::ToPrimary));

			test!(b"\x1B[1i" =>
				CSI::MediaCopy(CSI::Copy::FromPrimary));

			test!(b"\x1B[2i" =>
				CSI::MediaCopy(CSI::Copy::ToSecondary));

			test!(b"\x1B[3i" =>
				CSI::MediaCopy(CSI::Copy::FromSecondary));

			test!(b"\x1B[4i" =>
				CSI::MediaCopy(CSI::Copy::StopPrimary));

			test!(b"\x1B[5i" =>
				CSI::MediaCopy(CSI::Copy::StartPrimary));

			test!(b"\x1B[6i" =>
				CSI::MediaCopy(CSI::Copy::StopSecondary));

			test!(b"\x1B[7i" =>
				CSI::MediaCopy(CSI::Copy::StartSecondary));
		}

		#[test]
		fn np() {
			test!(b"\x1B[U" =>
				CSI::NextPage(1));

			test!(b"\x1B[2U" =>
				CSI::NextPage(2));
		}

		#[test]
		fn pec() {
			test!(b"\x1B[ Z" =>
				CSI::Presentation(CSI::Expansion::Normal));

			test!(b"\x1B[0 Z" =>
				CSI::Presentation(CSI::Expansion::Normal));

			test!(b"\x1B[1 Z" =>
				CSI::Presentation(CSI::Expansion::Expanded));

			test!(b"\x1B[2 Z" =>
				CSI::Presentation(CSI::Expansion::Condensed));
		}

		#[test]
		fn pfs() {
			test!(b"\x1B[ J" =>
				CSI::PageFormat(0));

			test!(b"\x1B[3 J" =>
				CSI::PageFormat(3));
		}

		#[test]
		fn pp() {
			test!(b"\x1B[V" =>
				CSI::PrecedingPage(1));

			test!(b"\x1B[3V" =>
				CSI::PrecedingPage(3));
		}

		#[test]
		fn ppa() {
			test!(b"\x1B[ P" =>
				CSI::PagePosition(1));

			test!(b"\x1B[3 P" =>
				CSI::PagePosition(3));
		}

		#[test]
		fn ppb() {
			test!(b"\x1B[ R" =>
				CSI::PageBack(1));

			test!(b"\x1B[3 R" =>
				CSI::PageBack(3));
		}

		#[test]
		fn ppr() {
			test!(b"\x1B[ Q" =>
				CSI::PageForward(1));

			test!(b"\x1B[3 Q" =>
				CSI::PageForward(3));
		}

		#[test]
		fn ptx() {
			test!(b"\x1B[\\" =>
				CSI::ParallelText(CSI::Parallel::End));

			test!(b"\x1B[0\\" =>
				CSI::ParallelText(CSI::Parallel::End));

			test!(b"\x1B[1\\" =>
				CSI::ParallelText(CSI::Parallel::Start));

			test!(b"\x1B[2\\" =>
				CSI::ParallelText(CSI::Parallel::StartSupplementary));

			test!(b"\x1B[3\\" =>
				CSI::ParallelText(CSI::Parallel::StartPhoneticJapanese));

			test!(b"\x1B[4\\" =>
				CSI::ParallelText(CSI::Parallel::StartPhoneticChinese));

			test!(b"\x1B[5\\" =>
				CSI::ParallelText(CSI::Parallel::StopPhonetic));
		}

		#[test]
		fn rcp() {
			test!(b"\x1B[u" =>
				CSI::RestoreCursor);
		}

		#[test]
		fn rep() {
			test!(b"\x1B[b" =>
				CSI::Repeat(1));

			test!(b"\x1B[10b" =>
				CSI::Repeat(10));
		}

		#[test]
		fn rm() {
			test!(b"\x1B[1l" =>
				CSI::Reset(vec![CSI::Mode::GuardedAreaTransfer]));

			test!(b"\x1B[2l" =>
				CSI::Reset(vec![CSI::Mode::KeyboardAction]));

			test!(b"\x1B[3l" =>
				CSI::Reset(vec![CSI::Mode::ControlRepresentation]));

			test!(b"\x1B[4l" =>
				CSI::Reset(vec![CSI::Mode::InsertionReplacement]));

			test!(b"\x1B[5l" =>
				CSI::Reset(vec![CSI::Mode::StatusReportTransfer]));

			test!(b"\x1B[6l" =>
				CSI::Reset(vec![CSI::Mode::Erasure]));

			test!(b"\x1B[7l" =>
				CSI::Reset(vec![CSI::Mode::LineEditing]));

			test!(b"\x1B[8l" =>
				CSI::Reset(vec![CSI::Mode::BidirectionalSupport]));

			test!(b"\x1B[9l" =>
				CSI::Reset(vec![CSI::Mode::DeviceComponentSelect]));

			test!(b"\x1B[10l" =>
				CSI::Reset(vec![CSI::Mode::CharacterEditing]));

			test!(b"\x1B[11l" =>
				CSI::Reset(vec![CSI::Mode::PositioningUnit]));

			test!(b"\x1B[12l" =>
				CSI::Reset(vec![CSI::Mode::SendReceive]));

			test!(b"\x1B[13l" =>
				CSI::Reset(vec![CSI::Mode::FormatEffectorAction]));

			test!(b"\x1B[14l" =>
				CSI::Reset(vec![CSI::Mode::FormatEffectorTransfer]));

			test!(b"\x1B[15l" =>
				CSI::Reset(vec![CSI::Mode::MultipleAreaTransfer]));

			test!(b"\x1B[16l" =>
				CSI::Reset(vec![CSI::Mode::TransferTermination]));

			test!(b"\x1B[17l" =>
				CSI::Reset(vec![CSI::Mode::SelectedAreaTransfer]));

			test!(b"\x1B[18l" =>
				CSI::Reset(vec![CSI::Mode::TabulationStop]));

			test!(b"\x1B[21l" =>
				CSI::Reset(vec![CSI::Mode::GraphicRenditionCombination]));

			test!(b"\x1B[22l" =>
				CSI::Reset(vec![CSI::Mode::ZeroDefault]));
		}

		#[test]
		fn sco() {
			test!(b"\x1B[ e" =>
				CSI::CharacterOrientation(0));

			test!(b"\x1B[0 e" =>
				CSI::CharacterOrientation(0));

			test!(b"\x1B[1 e" =>
				CSI::CharacterOrientation(45));

			test!(b"\x1B[2 e" =>
				CSI::CharacterOrientation(90));

			test!(b"\x1B[3 e" =>
				CSI::CharacterOrientation(135));

			test!(b"\x1B[4 e" =>
				CSI::CharacterOrientation(180));

			test!(b"\x1B[5 e" =>
				CSI::CharacterOrientation(225));

			test!(b"\x1B[6 e" =>
				CSI::CharacterOrientation(270));

			test!(b"\x1B[7 e" =>
				CSI::CharacterOrientation(315));
		}

		#[test]
		fn scp() {
			test!(b"\x1B[s" =>
				CSI::SaveCursor);
		}

		#[test]
		fn scs() {
			test!(b"\x1B[ b" =>
				CSI::CharacterSpacing(1));

			test!(b"\x1B[23 b" =>
				CSI::CharacterSpacing(23));
		}

		#[test]
		fn sd() {
			test!(b"\x1B[T" =>
				CSI::ScrollDown(1));

			test!(b"\x1B[73T" =>
				CSI::ScrollDown(73));
		}

		#[test]
		fn simd() {
			test!(b"\x1B[^" =>
				CSI::Movement(CSI::Direction::Forward));

			test!(b"\x1B[0^" =>
				CSI::Movement(CSI::Direction::Forward));

			test!(b"\x1B[1^" =>
				CSI::Movement(CSI::Direction::Backward));
		}

		#[test]
		fn sl() {
			test!(b"\x1B[ @" =>
				CSI::ScrollLeft(1));

			test!(b"\x1B[12 @" =>
				CSI::ScrollLeft(12));
		}

		#[test]
		fn sls() {
			test!(b"\x1B[ h" =>
				CSI::LineSpacing(1));

			test!(b"\x1B[12 h" =>
				CSI::LineSpacing(12));
		}

		#[test]
		fn sm() {
			test!(b"\x1B[1h" =>
				CSI::Set(vec![CSI::Mode::GuardedAreaTransfer]));

			test!(b"\x1B[2h" =>
				CSI::Set(vec![CSI::Mode::KeyboardAction]));

			test!(b"\x1B[3h" =>
				CSI::Set(vec![CSI::Mode::ControlRepresentation]));

			test!(b"\x1B[4h" =>
				CSI::Set(vec![CSI::Mode::InsertionReplacement]));

			test!(b"\x1B[5h" =>
				CSI::Set(vec![CSI::Mode::StatusReportTransfer]));

			test!(b"\x1B[6h" =>
				CSI::Set(vec![CSI::Mode::Erasure]));

			test!(b"\x1B[7h" =>
				CSI::Set(vec![CSI::Mode::LineEditing]));

			test!(b"\x1B[8h" =>
				CSI::Set(vec![CSI::Mode::BidirectionalSupport]));

			test!(b"\x1B[9h" =>
				CSI::Set(vec![CSI::Mode::DeviceComponentSelect]));

			test!(b"\x1B[10h" =>
				CSI::Set(vec![CSI::Mode::CharacterEditing]));

			test!(b"\x1B[11h" =>
				CSI::Set(vec![CSI::Mode::PositioningUnit]));

			test!(b"\x1B[12h" =>
				CSI::Set(vec![CSI::Mode::SendReceive]));

			test!(b"\x1B[13h" =>
				CSI::Set(vec![CSI::Mode::FormatEffectorAction]));

			test!(b"\x1B[14h" =>
				CSI::Set(vec![CSI::Mode::FormatEffectorTransfer]));

			test!(b"\x1B[15h" =>
				CSI::Set(vec![CSI::Mode::MultipleAreaTransfer]));

			test!(b"\x1B[16h" =>
				CSI::Set(vec![CSI::Mode::TransferTermination]));

			test!(b"\x1B[17h" =>
				CSI::Set(vec![CSI::Mode::SelectedAreaTransfer]));

			test!(b"\x1B[18h" =>
				CSI::Set(vec![CSI::Mode::TabulationStop]));

			test!(b"\x1B[21h" =>
				CSI::Set(vec![CSI::Mode::GraphicRenditionCombination]));

			test!(b"\x1B[22h" =>
				CSI::Set(vec![CSI::Mode::ZeroDefault]));
		}

		#[test]
		fn sr() {
			test!(b"\x1B[ A" =>
				CSI::ScrollRight(1));

			test!(b"\x1B[43 A" =>
				CSI::ScrollRight(43));
		}

		#[test]
		fn srs() {
			test!(b"\x1B[[" =>
				CSI::ReverseString(false));

			test!(b"\x1B[0[" =>
				CSI::ReverseString(false));

			test!(b"\x1B[1[" =>
				CSI::ReverseString(true));
		}

		#[test]
		fn ssu() {
			test!(b"\x1B[ I" =>
				CSI::SizeUnit(CSI::Unit::Character));

			test!(b"\x1B[0 I" =>
				CSI::SizeUnit(CSI::Unit::Character));

			test!(b"\x1B[1 I" =>
				CSI::SizeUnit(CSI::Unit::Millimeter));

			test!(b"\x1B[2 I" =>
				CSI::SizeUnit(CSI::Unit::ComputerDecipoint));

			test!(b"\x1B[3 I" =>
				CSI::SizeUnit(CSI::Unit::Decidot));

			test!(b"\x1B[4 I" =>
				CSI::SizeUnit(CSI::Unit::Mil));

			test!(b"\x1B[5 I" =>
				CSI::SizeUnit(CSI::Unit::BasicMeasuringUnit));

			test!(b"\x1B[6 I" =>
				CSI::SizeUnit(CSI::Unit::Micrometer));

			test!(b"\x1B[7 I" =>
				CSI::SizeUnit(CSI::Unit::Pixel));

			test!(b"\x1B[8 I" =>
				CSI::SizeUnit(CSI::Unit::Decipoint));
		}

		#[test]
		fn ssw() {
			test!(b"\x1B[ [" =>
				CSI::SpaceWidth(1));

			test!(b"\x1B[12 [" =>
				CSI::SpaceWidth(12));
		}

		#[test]
		fn su() {
			test!(b"\x1B[S" =>
				CSI::ScrollUp(1));

			test!(b"\x1B[37S" =>
				CSI::ScrollUp(37));
		}

		#[test]
		fn vpa() {
			test!(b"\x1B[d" =>
				CSI::LinePosition(1));

			test!(b"\x1B[42d" =>
				CSI::LinePosition(42));
		}

		#[test]
		fn vpb() {
			test!(b"\x1B[k" =>
				CSI::CursorUp(1));

			test!(b"\x1B[42k" =>
				CSI::CursorUp(42));
		}

		#[test]
		fn vpr() {
			test!(b"\x1B[e" =>
				CSI::CursorDown(1));

			test!(b"\x1B[42e" =>
				CSI::CursorDown(42));
		}
	}

	mod format {
		pub use control::*;

		macro_rules! test {
			($code:expr) => (
				let item = Item::C1(C1::ControlSequenceIntroducer($code));

				let mut result = vec![];
				item.fmt(&mut result, true).unwrap();
				assert_eq!(item, parse(&result).unwrap().1);

				let mut result = vec![];
				item.fmt(&mut result, false).unwrap();
				assert_eq!(item, parse(&result).unwrap().1);
			);
		}

		#[test]
		fn cbt() {
			test!(CSI::CursorBackTabulation(1));
			test!(CSI::CursorBackTabulation(23));
		}

		#[test]
		fn cha() {
			test!(CSI::CursorHorizontalPosition(0));
			test!(CSI::CursorHorizontalPosition(42));
		}

		#[test]
		fn cht() {
			test!(CSI::CursorForwardTabulation(1));
			test!(CSI::CursorForwardTabulation(23));
		}

		#[test]
		fn cnl() {
			test!(CSI::CursorNextLine(1));
			test!(CSI::CursorNextLine(12));
		}

		#[test]
		fn cpl() {
			test!(CSI::CursorPreviousLine(1));
			test!(CSI::CursorPreviousLine(43));
		}

		#[test]
		fn cpr() {
			test!(CSI::CursorPositionReport(0, 0));
			test!(CSI::CursorPositionReport(1, 0));
			test!(CSI::CursorPositionReport(0, 1));
			test!(CSI::CursorPositionReport(1, 1));
		}

		#[test]
		fn ctc() {
			test!(CSI::CursorTabulationControl(CSI::Tabulation::Character));
			test!(CSI::CursorTabulationControl(CSI::Tabulation::Line));
			test!(CSI::CursorTabulationControl(CSI::Tabulation::ClearCharacter));
			test!(CSI::CursorTabulationControl(CSI::Tabulation::ClearLine));
			test!(CSI::CursorTabulationControl(CSI::Tabulation::ClearLineAllCharacters));
			test!(CSI::CursorTabulationControl(CSI::Tabulation::ClearAllCharacters));
			test!(CSI::CursorTabulationControl(CSI::Tabulation::ClearAllLines));
		}

		#[test]
		fn cub() {
			test!(CSI::CursorBack(1));
			test!(CSI::CursorBack(37));
		}

		#[test]
		fn cud() {
			test!(CSI::CursorDown(1));
			test!(CSI::CursorDown(42));
		}

		#[test]
		fn cuf() {
			test!(CSI::CursorForward(1));
			test!(CSI::CursorForward(13));
		}

		#[test]
		fn cup() {
			test!(CSI::CursorPosition(0, 0));
			test!(CSI::CursorPosition(1, 2));
			test!(CSI::CursorPosition(0, 2));
			test!(CSI::CursorPosition(1, 0));
		}

		#[test]
		fn cuu() {
			test!(CSI::CursorUp(1));
			test!(CSI::CursorUp(23));
		}

		#[test]
		fn cvt() {
			test!(CSI::CursorLineTabulation(1));
			test!(CSI::CursorLineTabulation(23));
		}

		#[test]
		fn da() {
			test!(CSI::DeviceAttributes(0));
			test!(CSI::DeviceAttributes(23));
		}

		#[test]
		fn daq() {
			test!(CSI::DefineAreaQualification(CSI::Qualification::UnprotectedUnguarded));
			test!(CSI::DefineAreaQualification(CSI::Qualification::ProtectedGuarded));
			test!(CSI::DefineAreaQualification(CSI::Qualification::GraphicCharacterInput));
			test!(CSI::DefineAreaQualification(CSI::Qualification::NumericInput));
			test!(CSI::DefineAreaQualification(CSI::Qualification::AlphabeticInput));
			test!(CSI::DefineAreaQualification(CSI::Qualification::AlignLast));
			test!(CSI::DefineAreaQualification(CSI::Qualification::ZeroFill));
			test!(CSI::DefineAreaQualification(CSI::Qualification::FieldStart));
			test!(CSI::DefineAreaQualification(CSI::Qualification::ProtectedUnguarded));
			test!(CSI::DefineAreaQualification(CSI::Qualification::SpaceFill));
			test!(CSI::DefineAreaQualification(CSI::Qualification::AlignFirst));
			test!(CSI::DefineAreaQualification(CSI::Qualification::Reverse));
		}

		#[test]
		fn dch() {
			test!(CSI::DeleteCharacter(1));
			test!(CSI::DeleteCharacter(8));
		}

		#[test]
		fn dl() {
			test!(CSI::DeleteLine(1));
			test!(CSI::DeleteLine(8));
		}

		#[test]
		fn dsr() {
			test!(CSI::DeviceStatusReport);
		}

		#[test]
		fn dta() {
			test!(CSI::DimensionTextArea(0, 0));
			test!(CSI::DimensionTextArea(1, 0));
			test!(CSI::DimensionTextArea(0, 1));
			test!(CSI::DimensionTextArea(1, 1));
		}

		#[test]
		fn ea() {
			test!(CSI::EraseArea(CSI::Erase::ToEnd));
			test!(CSI::EraseArea(CSI::Erase::ToStart));
			test!(CSI::EraseArea(CSI::Erase::All));
		}

		#[test]
		fn ech() {
			test!(CSI::EraseCharacter(1));
			test!(CSI::EraseCharacter(8));
		}

		#[test]
		fn ed() {
			test!(CSI::EraseDisplay(CSI::Erase::ToEnd));
			test!(CSI::EraseDisplay(CSI::Erase::ToStart));
			test!(CSI::EraseDisplay(CSI::Erase::All));
		}

		#[test]
		fn ef() {
			test!(CSI::EraseField(CSI::Erase::ToEnd));
			test!(CSI::EraseField(CSI::Erase::ToEnd));
			test!(CSI::EraseField(CSI::Erase::ToStart));
			test!(CSI::EraseField(CSI::Erase::All));
		}

		#[test]
		fn el() {
			test!(CSI::EraseLine(CSI::Erase::ToEnd));
			test!(CSI::EraseLine(CSI::Erase::ToEnd));
			test!(CSI::EraseLine(CSI::Erase::ToStart));
			test!(CSI::EraseLine(CSI::Erase::All));
		}

		#[test]
		fn fnk() {
			test!(CSI::FunctionKey(0));
			test!(CSI::FunctionKey(13));
		}

		#[test]
		fn fnt() {
			test!(CSI::SelectFont(0, 0));
			test!(CSI::SelectFont(13, 0));
			test!(CSI::SelectFont(0, 13));
			test!(CSI::SelectFont(13, 13));
		}

		#[test]
		fn gcc() {
			test!(CSI::GraphicCharacterCombination(CSI::Combination::Next));
			test!(CSI::GraphicCharacterCombination(CSI::Combination::Start));
			test!(CSI::GraphicCharacterCombination(CSI::Combination::End));
		}

		#[test]
		fn gsm() {
			test!(CSI::GraphicSizeModification(100, 100));
			test!(CSI::GraphicSizeModification(100, 13));
			test!(CSI::GraphicSizeModification(13, 100));
			test!(CSI::GraphicSizeModification(13, 13));
		}

		#[test]
		fn hpa() {
			test!(CSI::CursorHorizontalPosition(0));
			test!(CSI::CursorHorizontalPosition(1));
		}

		#[test]
		fn hpb() {
			test!(CSI::CursorBack(1));
			test!(CSI::CursorBack(2));
		}

		#[test]
		fn hpr() {
			test!(CSI::CursorForward(1));
			test!(CSI::CursorForward(2));
		}

		#[test]
		fn hvp() {
			test!(CSI::CursorPosition(0, 0));
			test!(CSI::CursorPosition(12, 0));
			test!(CSI::CursorPosition(0, 12));
			test!(CSI::CursorPosition(12, 12));
		}

		#[test]
		fn ich() {
			test!(CSI::InsertBlankCharacter(1));
			test!(CSI::InsertBlankCharacter(23));
		}

		#[test]
		fn idcs() {
			test!(CSI::IdentifyDeviceControlString(None));
			test!(CSI::IdentifyDeviceControlString(Some(1)));
		}

		#[test]
		fn igs() {
			test!(CSI::IdentifyGraphicSubrepertoire(None));
			test!(CSI::IdentifyGraphicSubrepertoire(Some(1)));
		}

		#[test]
		fn il() {
			test!(CSI::InsertBlankLine(1));
			test!(CSI::InsertBlankLine(2));
		}

		#[test]
		fn jfy() {
			test!(CSI::Justify(vec![Some(1), Some(2)]));
		}

		#[test]
		fn mc() {
			test!(CSI::MediaCopy(CSI::Copy::ToPrimary));
			test!(CSI::MediaCopy(CSI::Copy::FromPrimary));
			test!(CSI::MediaCopy(CSI::Copy::ToSecondary));
			test!(CSI::MediaCopy(CSI::Copy::FromSecondary));
			test!(CSI::MediaCopy(CSI::Copy::StopPrimary));
			test!(CSI::MediaCopy(CSI::Copy::StartPrimary));
			test!(CSI::MediaCopy(CSI::Copy::StopSecondary));
			test!(CSI::MediaCopy(CSI::Copy::StartSecondary));
		}

		#[test]
		fn np() {
			test!(CSI::NextPage(1));
			test!(CSI::NextPage(2));
		}

		#[test]
		fn pec() {
			test!(CSI::Presentation(CSI::Expansion::Normal));
			test!(CSI::Presentation(CSI::Expansion::Expanded));
			test!(CSI::Presentation(CSI::Expansion::Condensed));
		}

		#[test]
		fn pfs() {
			test!(CSI::PageFormat(0));
			test!(CSI::PageFormat(3));
		}

		#[test]
		fn pp() {
			test!(CSI::PrecedingPage(1));
			test!(CSI::PrecedingPage(3));
		}

		#[test]
		fn ppa() {
			test!(CSI::PagePosition(1));
			test!(CSI::PagePosition(3));
		}

		#[test]
		fn ppb() {
			test!(CSI::PageBack(1));
			test!(CSI::PageBack(3));
		}

		#[test]
		fn ppr() {
			test!(CSI::PageForward(1));
			test!(CSI::PageForward(3));
		}

		#[test]
		fn ptx() {
			test!(CSI::ParallelText(CSI::Parallel::End));
			test!(CSI::ParallelText(CSI::Parallel::Start));
			test!(CSI::ParallelText(CSI::Parallel::StartSupplementary));
			test!(CSI::ParallelText(CSI::Parallel::StartPhoneticJapanese));
			test!(CSI::ParallelText(CSI::Parallel::StartPhoneticChinese));
			test!(CSI::ParallelText(CSI::Parallel::StopPhonetic));
		}

		#[test]
		fn rcp() {
			test!(CSI::RestoreCursor);
		}

		#[test]
		fn rep() {
			test!(CSI::Repeat(1));
			test!(CSI::Repeat(10));
		}

		#[test]
		fn rm() {
			test!(CSI::Reset(vec![CSI::Mode::GuardedAreaTransfer]));
			test!(CSI::Reset(vec![CSI::Mode::KeyboardAction]));
			test!(CSI::Reset(vec![CSI::Mode::ControlRepresentation]));
			test!(CSI::Reset(vec![CSI::Mode::InsertionReplacement]));
			test!(CSI::Reset(vec![CSI::Mode::StatusReportTransfer]));
			test!(CSI::Reset(vec![CSI::Mode::Erasure]));
			test!(CSI::Reset(vec![CSI::Mode::LineEditing]));
			test!(CSI::Reset(vec![CSI::Mode::BidirectionalSupport]));
			test!(CSI::Reset(vec![CSI::Mode::DeviceComponentSelect]));
			test!(CSI::Reset(vec![CSI::Mode::CharacterEditing]));
			test!(CSI::Reset(vec![CSI::Mode::PositioningUnit]));
			test!(CSI::Reset(vec![CSI::Mode::SendReceive]));
			test!(CSI::Reset(vec![CSI::Mode::FormatEffectorAction]));
			test!(CSI::Reset(vec![CSI::Mode::FormatEffectorTransfer]));
			test!(CSI::Reset(vec![CSI::Mode::MultipleAreaTransfer]));
			test!(CSI::Reset(vec![CSI::Mode::TransferTermination]));
			test!(CSI::Reset(vec![CSI::Mode::SelectedAreaTransfer]));
			test!(CSI::Reset(vec![CSI::Mode::TabulationStop]));
			test!(CSI::Reset(vec![CSI::Mode::GraphicRenditionCombination]));
			test!(CSI::Reset(vec![CSI::Mode::ZeroDefault]));
		}

		#[test]
		fn sco() {
			test!(CSI::CharacterOrientation(0));
			test!(CSI::CharacterOrientation(45));
			test!(CSI::CharacterOrientation(90));
			test!(CSI::CharacterOrientation(135));
			test!(CSI::CharacterOrientation(180));
			test!(CSI::CharacterOrientation(225));
			test!(CSI::CharacterOrientation(270));
			test!(CSI::CharacterOrientation(315));
		}

		#[test]
		fn scp() {
			test!(CSI::SaveCursor);
		}

		#[test]
		fn scs() {
			test!(CSI::CharacterSpacing(1));
			test!(CSI::CharacterSpacing(23));
		}

		#[test]
		fn sd() {
			test!(CSI::ScrollDown(1));
			test!(CSI::ScrollDown(73));
		}

		#[test]
		fn simd() {
			test!(CSI::Movement(CSI::Direction::Forward));
			test!(CSI::Movement(CSI::Direction::Forward));
			test!(CSI::Movement(CSI::Direction::Backward));
		}

		#[test]
		fn sl() {
			test!(CSI::ScrollLeft(1));
			test!(CSI::ScrollLeft(12));
		}

		#[test]
		fn sls() {
			test!(CSI::LineSpacing(1));
			test!(CSI::LineSpacing(12));
		}

		#[test]
		fn sm() {
			test!(CSI::Set(vec![CSI::Mode::GuardedAreaTransfer]));
			test!(CSI::Set(vec![CSI::Mode::KeyboardAction]));
			test!(CSI::Set(vec![CSI::Mode::ControlRepresentation]));
			test!(CSI::Set(vec![CSI::Mode::InsertionReplacement]));
			test!(CSI::Set(vec![CSI::Mode::StatusReportTransfer]));
			test!(CSI::Set(vec![CSI::Mode::Erasure]));
			test!(CSI::Set(vec![CSI::Mode::LineEditing]));
			test!(CSI::Set(vec![CSI::Mode::BidirectionalSupport]));
			test!(CSI::Set(vec![CSI::Mode::DeviceComponentSelect]));
			test!(CSI::Set(vec![CSI::Mode::CharacterEditing]));
			test!(CSI::Set(vec![CSI::Mode::PositioningUnit]));
			test!(CSI::Set(vec![CSI::Mode::SendReceive]));
			test!(CSI::Set(vec![CSI::Mode::FormatEffectorAction]));
			test!(CSI::Set(vec![CSI::Mode::FormatEffectorTransfer]));
			test!(CSI::Set(vec![CSI::Mode::MultipleAreaTransfer]));
			test!(CSI::Set(vec![CSI::Mode::TransferTermination]));
			test!(CSI::Set(vec![CSI::Mode::SelectedAreaTransfer]));
			test!(CSI::Set(vec![CSI::Mode::TabulationStop]));
			test!(CSI::Set(vec![CSI::Mode::GraphicRenditionCombination]));
			test!(CSI::Set(vec![CSI::Mode::ZeroDefault]));
		}

		#[test]
		fn sr() {
			test!(CSI::ScrollRight(1));
			test!(CSI::ScrollRight(43));
		}

		#[test]
		fn srs() {
			test!(CSI::ReverseString(false));
			test!(CSI::ReverseString(true));
		}

		#[test]
		fn ssu() {
			test!(CSI::SizeUnit(CSI::Unit::Character));
			test!(CSI::SizeUnit(CSI::Unit::Millimeter));
			test!(CSI::SizeUnit(CSI::Unit::ComputerDecipoint));
			test!(CSI::SizeUnit(CSI::Unit::Decidot));
			test!(CSI::SizeUnit(CSI::Unit::Mil));
			test!(CSI::SizeUnit(CSI::Unit::BasicMeasuringUnit));
			test!(CSI::SizeUnit(CSI::Unit::Micrometer));
			test!(CSI::SizeUnit(CSI::Unit::Pixel));
			test!(CSI::SizeUnit(CSI::Unit::Decipoint));
		}

		#[test]
		fn ssw() {
			test!(CSI::SpaceWidth(1));
			test!(CSI::SpaceWidth(12));
		}

		#[test]
		fn su() {
			test!(CSI::ScrollUp(1));
			test!(CSI::ScrollUp(37));
		}

		#[test]
		fn vpa() {
			test!(CSI::LinePosition(1));
			test!(CSI::CursorDown(42));
		}

		#[test]
		fn vpb() {
			test!(CSI::CursorUp(1));
			test!(CSI::CursorUp(42));
		}

		#[test]
		fn vpr() {
			test!(CSI::CursorDown(1));
			test!(CSI::CursorDown(42));
		}
	}
}
