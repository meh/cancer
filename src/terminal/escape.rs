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

#![allow(non_snake_case)]

use std::str;
use nom::{self, IResult, Needed, rest};

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Item<'a> {
	String(&'a str),

	C0(C0::T),
	C1(C1::T),
}

named!(pub parse<Item>,
	alt!(control | string));


fn string(i: &[u8]) -> IResult<&[u8], Item> {
	const WIDTH: [u8; 256] = [
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x1F
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x3F
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x5F
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x7F
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x9F
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xBF
		0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
		2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0xDF
		3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
		4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
	];

	if i.is_empty() {
		return IResult::Incomplete(Needed::Size(1));
	}

	let mut length = 0;
	let mut input  = i;

	while control(input).is_err() {
		let w = WIDTH[input[0] as usize] as usize;

		if input.len() < w {
			return IResult::Incomplete(Needed::Size(w - input.len()));
		}

		length += w;
		input   = &input[w..];

		if input.is_empty() {
			break;
		}
	}

	if let Ok(string) = str::from_utf8(&i[..length]) {
		IResult::Done(&i[length..], Item::String(string))
	}
	else {
		IResult::Error(nom::Err::Code(nom::ErrorKind::Custom(9001)))
	}
}

named!(control<Item>,
	alt!(
		map!(C1::parse, |c| Item::C1(c)) |
		map!(C0::parse, |c| Item::C0(c))));

macro_rules! arg {
	($args:ident[$index:tt] => $default:tt) => (
		$args.get($index).and_then(|v| *v).unwrap_or($default)
	);
}

pub mod C0 {
	#[derive(Eq, PartialEq, Copy, Clone, Debug)]
	pub enum T {
		Null,
		StartHeading,
		StartText,
		EndText,
		EndTransmission,
		Enquiry,
		Acknowledge,
		Bell,
		Backspace,
		HorizontalTabulation,
		LineFeed,
		VerticalTabulation,
		FormFeed,
		CarriageReturn,
		ShiftOut,
		ShiftIn,
		DataLinkEscape,
		DeviceControlOne,
		DeviceControlTwo,
		DeviceControlThree,
		DeviceControlFour,
		NegativeAcknowledge,
		SynchronousIdle,
		EndTransmissionBlock,
		Cancel,
		EndMedium,
		Substitute,
		Escape,
		FileSeparator,
		GroupSeparator,
		RecordSeparator,
		UnitSeparator,
	}

	pub use self::T::*;

	named!(pub parse<T>,
		alt!(NUL | SOH | STX | ETX | EOT | ENQ | ACK | BEL | BS | HT | LF | VT | FF |
		     CR | SS | SI | DLE | DC1 | DC2 | DC3 | DC4 | NAK | SYN | ETB | CAN | EM |
		     SUB | ESC | FS | GS | RS | US));

	named!(NUL<T>,
		value!(Null,
			char!(0x00)));
	
	named!(SOH<T>,
		value!(StartHeading,
			char!(0x01)));
	
	named!(STX<T>,
		value!(StartText,
			char!(0x02)));
	
	named!(ETX<T>,
		value!(EndText,
			char!(0x03)));
	
	named!(EOT<T>,
		value!(EndTransmission,
			char!(0x04)));
	
	named!(ENQ<T>,
		value!(Enquiry,
			char!(0x05)));
	
	named!(ACK<T>,
		value!(Acknowledge,
			char!(0x06)));
	
	named!(BEL<T>,
		value!(Bell,
			char!(0x07)));
	
	named!(BS<T>,
		value!(Backspace,
			char!(0x08)));
	
	named!(HT<T>,
		value!(HorizontalTabulation,
			char!(0x09)));
	
	named!(LF<T>,
		value!(LineFeed,
			char!(0x0A)));
	
	named!(VT<T>,
		value!(VerticalTabulation,
			char!(0x0B)));
	
	named!(FF<T>,
		value!(FormFeed,
			char!(0x0C)));
	
	named!(CR<T>,
		value!(CarriageReturn,
			char!(0x0D)));
	
	named!(SS<T>,
		value!(ShiftOut,
			char!(0x0E)));
	
	named!(SI<T>,
		value!(ShiftIn,
			char!(0x0F)));
	
	named!(DLE<T>,
		value!(DataLinkEscape,
			char!(0x10)));
	
	named!(DC1<T>,
		value!(DeviceControlOne,
			char!(0x11)));
	
	named!(DC2<T>,
		value!(DeviceControlTwo,
			char!(0x12)));
	
	named!(DC3<T>,
		value!(DeviceControlThree,
			char!(0x13)));
	
	named!(DC4<T>,
		value!(DeviceControlFour,
			char!(0x14)));
	
	named!(NAK<T>,
		value!(NegativeAcknowledge,
			char!(0x15)));
	
	named!(SYN<T>,
		value!(SynchronousIdle,
			char!(0x16)));
	
	named!(ETB<T>,
		value!(EndTransmissionBlock,
			char!(0x17)));
	
	named!(CAN<T>,
		value!(Cancel,
			char!(0x18)));
	
	named!(EM<T>,
		value!(EndMedium,
			char!(0x19)));
	
	named!(SUB<T>,
		value!(Substitute,
			char!(0x1A)));
	
	named!(ESC<T>,
		value!(Escape,
			char!(0x1B)));
	
	named!(FS<T>,
		value!(FileSeparator,
			char!(0x1C)));
	
	named!(GS<T>,
		value!(GroupSeparator,
			char!(0x1D)));
	
	named!(RS<T>,
		value!(RecordSeparator,
			char!(0x1E)));
	
	named!(US<T>,
		value!(UnitSeparator,
			char!(0x1F)));
}

pub mod C1 {
	#[derive(Eq, PartialEq, Clone, Debug)]
	pub enum T {
		Delete,
		PaddingCharacter,
		HighOctetPreset,
		BreakPermittedHere,
		NoBreakHere,
		Index,
		NextLine,
		StartSelectedArea,
		EndSelectedArea,
		HorizontalTabulationSet,
		HorizontalTabulationWithJustification,
		VerticalTabulationSet,
		PartialLineDown,
		PartialLineUp,
		ReverseIndex,
		SingleShiftTwo,
		SingleShiftThree,
		// TODO: Followed by a string of printable characters (0x20 through 0x7E) and
		//       format effectors (0x08 through 0x0D), terminated by ST (0x9C).
		DeviceControlString,
		PrivateUseOne,
		PrivateUseTwo,
		SetTransmitState,
		CancelCharacter,
		MessageWaiting,
		StartProtectedArea,
		EndProtectedArea,
		// TODO: Followed by a control string terminated by ST (0x9C) that may
		//       contain any character except SOS or ST. Not part of the first edition of
		//       ISO/IEC 6429.[3]
		StartString,
		SingleGraphicCharacterIntroducer,
		// TODO: To be followed by a single printable character (0x20 through 0x7E)
		//       or format effector (0x08 through 0x0D).
		SingleCharacterIntroducer,
		ControlSequenceIntroducer(super::CSI::T),
		StringTerminator,
		// TODO: Followed by a string of printable characters (0x20 through 0x7E) and
		//       format effectors (0x08 through 0x0D), terminated by ST (0x9C). 
		OperatingSystemCommand,
		// TODO: Followed by a string of printable characters (0x20 through 0x7E) and
		//       format effectors (0x08 through 0x0D), terminated by ST (0x9C). 
		PrivacyMessage,
		// TODO: Followed by a string of printable characters (0x20 through 0x7E) and
		//       format effectors (0x08 through 0x0D), terminated by ST (0x9C). 
		ApplicationProgramCommand,
	}

	pub use self::T::*;

	named!(pub parse<T>,
		alt!(DEL | PAD | HOP | BPH | NBH | IND | NEL | SSA | ESA | HTS | HTJ | VTS |
		     PLD | PLU | RI | SS2 | SS3 | DCS | PU1 | PU2 | STS | CCH | MW | SPA |
		     EPA | SOS | SGCI | SCI | CSI | ST | OSC | PM | APC));
	
	named!(DEL<T>,
		value!(Delete,
			char!(0x7F)));
	
	named!(PAD<T>,
		value!(PaddingCharacter,
			char!(0x80)));
	
	named!(HOP<T>,
		value!(HighOctetPreset,
			char!(0x81)));
	
	named!(BPH<T>,
		value!(BreakPermittedHere,
			alt!(tag!(b"\x82") | tag!(b"\x1B\x42"))));
	
	named!(NBH<T>,
		value!(NoBreakHere,
			alt!(tag!(b"\x83") | tag!(b"\x1B\x43"))));
	
	named!(IND<T>,
		value!(Index,
			char!(0x84)));
	
	named!(NEL<T>,
		value!(NextLine,
			alt!(tag!(b"\x85") | tag!(b"\x1B\x45"))));
	
	named!(SSA<T>,
		value!(StartSelectedArea,
			alt!(tag!(b"\x86") | tag!(b"\x1B\x46"))));
	
	named!(ESA<T>,
		value!(EndSelectedArea,
			alt!(tag!(b"\x87") | tag!(b"\x1B\x47"))));
	
	named!(HTS<T>,
		value!(HorizontalTabulationSet,
			char!(0x88)));
	
	named!(HTJ<T>,
		value!(HorizontalTabulationWithJustification,
			alt!(tag!(b"\x89") | tag!(b"\x1B\x49"))));
	
	named!(VTS<T>,
		value!(VerticalTabulationSet,
			alt!(tag!(b"\x8A") | tag!(b"\x1B\x4A"))));
	
	named!(PLD<T>,
		value!(PartialLineDown,
			alt!(tag!(b"\x8B") | tag!(b"\x1B\x4B"))));
	
	named!(PLU<T>,
		value!(PartialLineUp,
			alt!(tag!(b"\x8C") | tag!(b"\x1B\x4C"))));
	
	named!(RI<T>,
		value!(ReverseIndex,
			alt!(tag!(b"\x8D") | tag!(b"\x1B\x4D"))));
	
	named!(SS2<T>,
		value!(SingleShiftTwo,
			alt!(tag!(b"\x8E") | tag!(b"\x1B\x4E"))));
	
	named!(SS3<T>,
		value!(SingleShiftThree,
			alt!(tag!(b"\x8F") | tag!(b"\x1B\x4F"))));
	
	named!(DCS<T>,
		value!(DeviceControlString,
			alt!(tag!(b"\x90") | tag!(b"\x1B\x50"))));
	
	named!(PU1<T>,
		value!(PrivateUseOne,
			char!(0x91)));
	
	named!(PU2<T>,
		value!(PrivateUseTwo,
			char!(0x92)));
	
	named!(STS<T>,
		value!(SetTransmitState,
			alt!(tag!(b"\x93") | tag!(b"\x1B\x53"))));
	
	named!(CCH<T>,
		value!(CancelCharacter,
			alt!(tag!(b"\x94") | tag!(b"\x1B\x54"))));
	
	named!(MW<T>,
		value!(MessageWaiting,
			char!(0x95)));
	
	named!(SPA<T>,
		value!(StartProtectedArea,
			alt!(tag!(b"\x96") | tag!(b"\x1B\x56"))));
	
	named!(EPA<T>,
		value!(EndProtectedArea,
			char!(0x97)));
	
	named!(SOS<T>,
		value!(StartString,
			alt!(tag!(b"\x98") | tag!(b"\x1B\x58"))));
	
	named!(SGCI<T>,
		value!(SingleGraphicCharacterIntroducer,
			char!(0x99)));
	
	named!(SCI<T>,
		value!(SingleCharacterIntroducer,
			alt!(tag!(b"\x9A") | tag!(b"\x1B\x5A"))));

	named!(CSI<T>,
		chain!(
			alt!(tag!(b"\x9B") | tag!(b"\x1B\x5B")) ~
			res: call!(super::CSI::parse),

			|| ControlSequenceIntroducer(res)));
	
	named!(ST<T>,
		value!(StringTerminator,
			alt!(tag!(b"\x9C") | tag!(b"\x1B\x5C"))));
	
	named!(OSC<T>,
		value!(OperatingSystemCommand,
			alt!(tag!(b"\x9D") | tag!(b"\x1B\x5D"))));
	
	named!(PM<T>,
		value!(PrivacyMessage,
			char!(0x9E)));
	
	named!(APC<T>,
		value!(ApplicationProgramCommand,
			alt!(tag!(b"\x9F") | tag!(b"\x1B\x5F"))));
}

pub mod CSI {
	use std::str;
	use std::u32;
	use nom::{self, ErrorKind, is_digit};

	#[derive(Eq, PartialEq, Clone, Debug)]
	pub enum T {
		CursorUp(u32),
		CursorDown(u32),
		CursorForward(u32),
		CursorBack(u32),
		CursorNextLine(u32),
		CursorPreviousLine(u32),
		CursorHorizontalPosition(u32),
		CursorPosition(u32, u32),
		EraseDisplay(Erase),
		EraseLine(Erase),
		ScrollUp(u32),
		ScrollDown(u32),
		MoveTo(u32, u32),
		SelectGraphicalRendition(Vec<super::SGR::T>),
		AuxPort(bool),
		DeviceStatusReport,
		SaveCursorPosition,
		RestoreCursorPosition,
		Private(Vec<Option<u32>>),
	}

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

	pub use self::T::*;

	const DIGIT: &[u8] = b"0123456789";

	named!(pub parse<T>,
		alt!(private | standard));

	named!(private<T>,
		chain!(
			char!('?') ~
			args: parameters,

			|| Private(args)));

	named!(standard<T>,
		chain!(
			args: parameters ~
			res:  alt!(apply!(CUU, &args) |
			           apply!(CUD, &args) |
			           apply!(CUF, &args) |
			           apply!(CUB, &args) |
			           apply!(CNL, &args) |
			           apply!(CPL, &args) |
			           apply!(CHA, &args) |
			           apply!(CUP, &args) |
			           apply!(ED,  &args) |
			           apply!(EL,  &args) |
			           apply!(SU,  &args) |
			           apply!(SD,  &args) |
			           apply!(HVP, &args) |
			           apply!(SGR, &args) |
			           apply!(AUX, &args) |
			           apply!(DSR, &args) |
			           apply!(SCP, &args) |
			           apply!(RCP, &args) |
			           apply!(RCP, &args)),

			|| res));

	named!(parameters<Vec<Option<u32>>>,
		many0!(parameter));

	named!(parameter<Option<u32> >,
		alt!(
			char!(';') => { |_| None } |
			chain!(
				number: is_a!(DIGIT) ~
				complete!(opt!(char!(';'))),
				|| number) => { |n|
					Some(u32::from_str_radix(unsafe { str::from_utf8_unchecked(n) }, 10).unwrap()) }));

	macro_rules! with_args {
		($name:ident<$n:tt, $params:ident>, $submac:ident!( $($args:tt)* )) => (
			fn $name<'a, 'b>(i: &'a [u8], $params: &'b [Option<u32>]) -> nom::IResult<&'a [u8], T> {
				if $params.len() <= $n {
					$submac!(i, $($args)*)
				}
				else {
					nom::IResult::Error(nom::Err::Code(ErrorKind::Custom(9001)))
				}
			}
		);

		($name:ident<$params:ident>, $submac:ident!( $($args:tt)* )) => (
			fn $name<'a, 'b>(i: &'a [u8], $params: &'b [Option<u32>]) -> nom::IResult<&'a [u8], T> {
				$submac!(i, $($args)*)
			}
		);
	
		($name:ident, $submac:ident!( $($args:tt)* )) => (
			fn $name<'a, 'b>(i: &'a [u8], args: &'b [Option<u32>]) -> nom::IResult<&'a [u8], T> {
				if args.is_empty() {
					$submac!(i, $($args)*)
				}
				else {
					nom::IResult::Error(nom::Err::Code(ErrorKind::Custom(9001)))
				}
			}
		);
	}

	with_args!(CUU<1, args>,
		map!(char!('A'), |_|
			CursorUp(arg!(args[0] => 1))));

	with_args!(CUD<1, args>,
		map!(char!('B'), |_|
			CursorDown(arg!(args[0] => 1))));

	with_args!(CUF<1, args>,
		map!(char!('C'), |_|
			CursorForward(arg!(args[0] => 1))));

	with_args!(CUB<1, args>,
		map!(char!('D'), |_|
			CursorBack(arg!(args[0] => 1))));

	with_args!(CNL<1, args>,
		map!(char!('E'), |_|
			CursorNextLine(arg!(args[0] => 1))));

	with_args!(CPL<1, args>,
		map!(char!('F'), |_|
			CursorPreviousLine(arg!(args[0] => 1))));

	with_args!(CHA<1, args>,
		map!(char!('G'), |_|
			CursorHorizontalPosition(arg!(args[0] => 1) - 1)));

	with_args!(CUP<2, args>,
		map!(char!('H'), |_|
			CursorPosition(arg!(args[0] => 1) - 1, arg!(args[1] => 1) - 1)));

	with_args!(ED<1, args>,
		map_res!(char!('J'), |_|
			Erase::from(arg!(args[0] => 0)).map(EraseDisplay)));

	with_args!(EL<1, args>,
		map_res!(char!('K'), |_|
			Erase::from(arg!(args[0] => 0)).map(EraseLine)));

	with_args!(SU<1, args>,
		map!(char!('S'), |_|
			ScrollUp(arg!(args[0] => 1))));

	with_args!(SD<1, args>,
		map!(char!('T'), |_|
			ScrollDown(arg!(args[0] => 1))));

	with_args!(HVP<2, args>,
		map!(char!('f'), |_|
			MoveTo(arg!(args[0] => 1) - 1, arg!(args[1] => 1) - 1)));

	with_args!(SGR<args>,
		map_res!(char!('m'), |_|
			super::SGR::parse(args).map(|v| SelectGraphicalRendition(v))));

	with_args!(AUX<1, args>,
		map_res!(char!('i'), |_|
			match arg!(args[0] => 0) {
				5 =>
					Ok(AuxPort(true)),

				6 =>
					Ok(AuxPort(false)),

				_ =>
					Err(nom::Err::Code::<&[u8], u32>(ErrorKind::Custom(9003)))
			}));

	with_args!(DSR<1, args>,
		map_res!(char!('n'), |_|
			match arg!(args[0] => 0) {
				6 =>
					Ok(DeviceStatusReport),

				_ =>
					Err(nom::Err::Code::<&[u8], u32>(ErrorKind::Custom(9004)))
			}));

	with_args!(SCP,
		map!(char!('s'), |_|
			SaveCursorPosition));

	with_args!(RCP,
		map!(char!('u'), |_|
			RestoreCursorPosition));
}

pub mod SGR {
	use nom::{self, ErrorKind};

	#[derive(Eq, PartialEq, Copy, Clone, Debug)]
	pub enum T {
		Reset,
		Font(Weight),
		Italic(bool),
		Underline(bool),
		Blink(bool),
		Reverse(bool),
		Invisible(bool),
		Struck(bool),
		Foreground(Color),
		Background(Color),
	}

	pub use self::T::*;

	#[derive(Eq, PartialEq, Copy, Clone, Debug)]
	pub enum Weight {
		Normal,
		Bold,
		Faint,
	}

	#[derive(Eq, PartialEq, Copy, Clone, Debug)]
	pub enum Color {
		Default,
		Transparent,
		Index(u8),
		Cmy(u8, u8, u8),
		Cmyk(u8, u8, u8, u8),
		Rgb(u8, u8, u8),
	}

	macro_rules! pop {
		($args:ident, $n:expr) => ({
			let count = $n;

			if $args.len() >= count {
				$args = &$args[count..];
			}
			else {
				$args = &[];
			}
		});
	}

	macro_rules! color {
		($args:ident) => ({
			let id = arg!($args[0] => 0);
			pop!($args, 1);

			match id {
				0 =>
					Color::Default,

				1 =>
					Color::Transparent,

				2 => {
					let r = arg!($args[0] => 0) as u8;
					let g = arg!($args[1] => 0) as u8;
					let b = arg!($args[2] => 0) as u8;
					pop!($args, 3);

					Color::Rgb(r, g, b)
				}

				3 => {
					let c = arg!($args[0] => 0) as u8;
					let m = arg!($args[1] => 0) as u8;
					let y = arg!($args[2] => 0) as u8;
					pop!($args, 3);

					Color::Cmy(c, m, y)
				}

				4 => {
					let c = arg!($args[0] => 0) as u8;
					let m = arg!($args[1] => 0) as u8;
					let y = arg!($args[2] => 0) as u8;
					let k = arg!($args[3] => 0) as u8;
					pop!($args, 4);

					Color::Cmyk(c, m, y, k)
				}

				5 => {
					let index = arg!($args[0] => 0) as u8;
					pop!($args, 1);

					Color::Index(index)
				}

				_ =>
					return Err(nom::Err::Code(ErrorKind::Custom(9006)))
			}
		})
	}

	pub fn parse<'a, 'b>(args: &'b [Option<u32>]) -> Result<Vec<T>, nom::Err<&'a [u8]>> {
		if args.is_empty() {
			return Ok(vec![Reset]);
		}

		let mut result = Vec::new();
		let mut args   = args;

		while !args.is_empty() {
			let id = arg!(args[0] => 0);
			pop!(args, 1);

			result.push(match id {
				0 =>
					Reset,

				1 =>
					Font(Weight::Bold),

				2 =>
					Font(Weight::Faint),

				3 =>
					Italic(true),

				4 =>
					Underline(true),

				5 | 6 =>
					Blink(true),

				7 =>
					Reverse(true),

				8 =>
					Invisible(true),

				9 =>
					Struck(true),

				22 =>
					Font(Weight::Normal),

				23 =>
					Italic(false),

				24 =>
					Underline(false),

				25 =>
					Blink(false),

				27 =>
					Reverse(false),

				28 =>
					Invisible(false),

				29 =>
					Struck(false),

				c if c >= 30 && c <= 37 =>
					Foreground(Color::Index(c as u8 - 30)),

				38 =>
					Foreground(color!(args)),

				39 =>
					Foreground(Color::Default),

				c if c >= 40 && c <= 47 =>
					Background(Color::Index(c as u8 - 40)),

				48 =>
					Background(color!(args)),

				49 =>
					Background(Color::Default),

				c if c >= 90 && c <= 97 =>
					Foreground(Color::Index(c as u8 - 90 + 8)),

				c if c >= 100 && c <= 107 =>
					Background(Color::Index(c as u8 - 100 + 8)),

				_ =>
					return Err(nom::Err::Code(ErrorKind::Custom(9001)))
			});
		}

		Ok(result)
	}
}

#[cfg(test)]
mod test {
	mod c0 {
		pub use terminal::escape::*;

		#[test]
		fn nul() {
			assert_eq!(Item::C0(C0::Null),
				parse(b"\x00").unwrap().1);
		}

		#[test]
		fn soh() {
			assert_eq!(Item::C0(C0::StartHeading),
				parse(b"\x01").unwrap().1);
		}

		#[test]
		fn stx() {
			assert_eq!(Item::C0(C0::StartText),
				parse(b"\x02").unwrap().1);
		}

		#[test]
		fn etx() {
			assert_eq!(Item::C0(C0::EndText),
				parse(b"\x03").unwrap().1);
		}

		#[test]
		fn eot() {
			assert_eq!(Item::C0(C0::EndTransmission),
				parse(b"\x04").unwrap().1);
		}

		#[test]
		fn enq() {
			assert_eq!(Item::C0(C0::Enquiry),
				parse(b"\x05").unwrap().1);
		}

		#[test]
		fn ack() {
			assert_eq!(Item::C0(C0::Acknowledge),
				parse(b"\x06").unwrap().1);
		}

		#[test]
		fn bel() {
			assert_eq!(Item::C0(C0::Bell),
				parse(b"\x07").unwrap().1);
		}

		#[test]
		fn bs() {
			assert_eq!(Item::C0(C0::Backspace),
				parse(b"\x08").unwrap().1);
		}

		#[test]
		fn ht() {
			assert_eq!(Item::C0(C0::HorizontalTabulation),
				parse(b"\x09").unwrap().1);
		}

		#[test]
		fn lf() {
			assert_eq!(Item::C0(C0::LineFeed),
				parse(b"\x0A").unwrap().1);
		}

		#[test]
		fn vf() {
			assert_eq!(Item::C0(C0::VerticalTabulation),
				parse(b"\x0B").unwrap().1);
		}

		#[test]
		fn ff() {
			assert_eq!(Item::C0(C0::FormFeed),
				parse(b"\x0C").unwrap().1);
		}

		#[test]
		fn cr() {
			assert_eq!(Item::C0(C0::CarriageReturn),
				parse(b"\x0D").unwrap().1);
		}

		#[test]
		fn ss() {
			assert_eq!(Item::C0(C0::ShiftOut),
				parse(b"\x0E").unwrap().1);
		}

		#[test]
		fn si() {
			assert_eq!(Item::C0(C0::ShiftIn),
				parse(b"\x0F").unwrap().1);
		}

		#[test]
		fn dle() {
			assert_eq!(Item::C0(C0::DataLinkEscape),
				parse(b"\x10").unwrap().1);
		}

		#[test]
		fn dc1() {
			assert_eq!(Item::C0(C0::DeviceControlOne),
				parse(b"\x11").unwrap().1);
		}

		#[test]
		fn dc2() {
			assert_eq!(Item::C0(C0::DeviceControlTwo),
				parse(b"\x12").unwrap().1);
		}

		#[test]
		fn dc3() {
			assert_eq!(Item::C0(C0::DeviceControlThree),
				parse(b"\x13").unwrap().1);
		}

		#[test]
		fn dc4() {
			assert_eq!(Item::C0(C0::DeviceControlFour),
				parse(b"\x14").unwrap().1);
		}

		#[test]
		fn nak() {
			assert_eq!(Item::C0(C0::NegativeAcknowledge),
				parse(b"\x15").unwrap().1);
		}

		#[test]
		fn syn() {
			assert_eq!(Item::C0(C0::SynchronousIdle),
				parse(b"\x16").unwrap().1);
		}

		#[test]
		fn etb() {
			assert_eq!(Item::C0(C0::EndTransmissionBlock),
				parse(b"\x17").unwrap().1);
		}

		#[test]
		fn can() {
			assert_eq!(Item::C0(C0::Cancel),
				parse(b"\x18").unwrap().1);
		}

		#[test]
		fn em() {
			assert_eq!(Item::C0(C0::EndMedium),
				parse(b"\x19").unwrap().1);
		}

		#[test]
		fn sub() {
			assert_eq!(Item::C0(C0::Substitute),
				parse(b"\x1A").unwrap().1);
		}

		#[test]
		fn fs() {
			assert_eq!(Item::C0(C0::FileSeparator),
				parse(b"\x1C").unwrap().1);
		}

		#[test]
		fn gs() {
			assert_eq!(Item::C0(C0::GroupSeparator),
				parse(b"\x1D").unwrap().1);
		}

		#[test]
		fn rs() {
			assert_eq!(Item::C0(C0::RecordSeparator),
				parse(b"\x1E").unwrap().1);
		}

		#[test]
		fn us() {
			assert_eq!(Item::C0(C0::UnitSeparator),
				parse(b"\x1F").unwrap().1);
		}
	}

	mod c1 {
		pub use terminal::escape::*;

		#[test]
		fn del() {
			assert_eq!(Item::C1(C1::Delete),
				parse(b"\x7F").unwrap().1);
		}

		#[test]
		fn pad() {
			assert_eq!(Item::C1(C1::PaddingCharacter),
				parse(b"\x80").unwrap().1);
		}

		#[test]
		fn hop() {
			assert_eq!(Item::C1(C1::HighOctetPreset),
				parse(b"\x81").unwrap().1);
		}

		#[test]
		fn bph() {
			assert_eq!(Item::C1(C1::BreakPermittedHere),
				parse(b"\x82").unwrap().1);

			assert_eq!(Item::C1(C1::BreakPermittedHere),
				parse(b"\x1B\x42").unwrap().1);
		}

		#[test]
		fn nbh() {
			assert_eq!(Item::C1(C1::NoBreakHere),
				parse(b"\x83").unwrap().1);

			assert_eq!(Item::C1(C1::NoBreakHere),
				parse(b"\x1B\x43").unwrap().1);
		}

		#[test]
		fn ind() {
			assert_eq!(Item::C1(C1::Index),
				parse(b"\x84").unwrap().1);
		}

		#[test]
		fn nel() {
			assert_eq!(Item::C1(C1::NextLine),
				parse(b"\x85").unwrap().1);

			assert_eq!(Item::C1(C1::NextLine),
				parse(b"\x1B\x45").unwrap().1);
		}


		#[test]
		fn ssa() {
			assert_eq!(Item::C1(C1::StartSelectedArea),
				parse(b"\x86").unwrap().1);

			assert_eq!(Item::C1(C1::StartSelectedArea),
				parse(b"\x1B\x46").unwrap().1);
		}

		#[test]
		fn esa() {
			assert_eq!(Item::C1(C1::EndSelectedArea),
				parse(b"\x87").unwrap().1);

			assert_eq!(Item::C1(C1::EndSelectedArea),
				parse(b"\x1B\x47").unwrap().1);
		}

		#[test]
		fn hts() {
			assert_eq!(Item::C1(C1::HorizontalTabulationSet),
				parse(b"\x88").unwrap().1);
		}

		#[test]
		fn htj() {
			assert_eq!(Item::C1(C1::HorizontalTabulationWithJustification),
				parse(b"\x89").unwrap().1);

			assert_eq!(Item::C1(C1::HorizontalTabulationWithJustification),
				parse(b"\x1B\x49").unwrap().1);
		}

		#[test]
		fn vts() {
			assert_eq!(Item::C1(C1::VerticalTabulationSet),
				parse(b"\x8A").unwrap().1);

			assert_eq!(Item::C1(C1::VerticalTabulationSet),
				parse(b"\x1B\x4A").unwrap().1);
		}

		#[test]
		fn pld() {
			assert_eq!(Item::C1(C1::PartialLineDown),
				parse(b"\x8B").unwrap().1);

			assert_eq!(Item::C1(C1::PartialLineDown),
				parse(b"\x1B\x4B").unwrap().1);
		}

		#[test]
		fn plu() {
			assert_eq!(Item::C1(C1::PartialLineUp),
				parse(b"\x8C").unwrap().1);

			assert_eq!(Item::C1(C1::PartialLineUp),
				parse(b"\x1B\x4C").unwrap().1);
		}

		#[test]
		fn ri() {
			assert_eq!(Item::C1(C1::ReverseIndex),
				parse(b"\x8D").unwrap().1);

			assert_eq!(Item::C1(C1::ReverseIndex),
				parse(b"\x1B\x4D").unwrap().1);
		}

		#[test]
		fn ss2() {
			assert_eq!(Item::C1(C1::SingleShiftTwo),
				parse(b"\x8E").unwrap().1);

			assert_eq!(Item::C1(C1::SingleShiftTwo),
				parse(b"\x1B\x4E").unwrap().1);
		}

		#[test]
		fn ss3() {
			assert_eq!(Item::C1(C1::SingleShiftThree),
				parse(b"\x8F").unwrap().1);

			assert_eq!(Item::C1(C1::SingleShiftThree),
				parse(b"\x1B\x4F").unwrap().1);
		}

		#[test]
		fn dcs() {
			assert_eq!(Item::C1(C1::DeviceControlString),
				parse(b"\x90").unwrap().1);

			assert_eq!(Item::C1(C1::DeviceControlString),
				parse(b"\x1B\x50").unwrap().1);
		}

		#[test]
		fn pu1() {
			assert_eq!(Item::C1(C1::PrivateUseOne),
				parse(b"\x91").unwrap().1);
		}

		#[test]
		fn pu2() {
			assert_eq!(Item::C1(C1::PrivateUseTwo),
				parse(b"\x92").unwrap().1);
		}

		#[test]
		fn sts() {
			assert_eq!(Item::C1(C1::SetTransmitState),
				parse(b"\x93").unwrap().1);

			assert_eq!(Item::C1(C1::SetTransmitState),
				parse(b"\x1B\x53").unwrap().1);
		}

		#[test]
		fn cch() {
			assert_eq!(Item::C1(C1::CancelCharacter),
				parse(b"\x94").unwrap().1);

			assert_eq!(Item::C1(C1::CancelCharacter),
				parse(b"\x1B\x54").unwrap().1);
		}

		#[test]
		fn mw() {
			assert_eq!(Item::C1(C1::MessageWaiting),
				parse(b"\x95").unwrap().1);
		}

		#[test]
		fn spa() {
			assert_eq!(Item::C1(C1::StartProtectedArea),
				parse(b"\x96").unwrap().1);

			assert_eq!(Item::C1(C1::StartProtectedArea),
				parse(b"\x1B\x56").unwrap().1);
		}

		#[test]
		fn epa() {
			assert_eq!(Item::C1(C1::EndProtectedArea),
				parse(b"\x97").unwrap().1);
		}

		#[test]
		fn sos() {
			assert_eq!(Item::C1(C1::StartString),
				parse(b"\x98").unwrap().1);

			assert_eq!(Item::C1(C1::StartString),
				parse(b"\x1B\x58").unwrap().1);
		}

		#[test]
		fn sgci() {
			assert_eq!(Item::C1(C1::SingleGraphicCharacterIntroducer),
				parse(b"\x99").unwrap().1);
		}

		#[test]
		fn sci() {
			assert_eq!(Item::C1(C1::SingleCharacterIntroducer),
				parse(b"\x9A").unwrap().1);

			assert_eq!(Item::C1(C1::SingleCharacterIntroducer),
				parse(b"\x1B\x5A").unwrap().1);
		}

		#[test]
		fn st() {
			assert_eq!(Item::C1(C1::StringTerminator),
				parse(b"\x9C").unwrap().1);

			assert_eq!(Item::C1(C1::StringTerminator),
				parse(b"\x1B\x5C").unwrap().1);
		}

		#[test]
		fn osc() {
			assert_eq!(Item::C1(C1::OperatingSystemCommand),
				parse(b"\x9D").unwrap().1);

			assert_eq!(Item::C1(C1::OperatingSystemCommand),
				parse(b"\x1B\x5D").unwrap().1);
		}

		#[test]
		fn pn() {
			assert_eq!(Item::C1(C1::PrivacyMessage),
				parse(b"\x9E").unwrap().1);
		}

		#[test]
		fn apc() {
			assert_eq!(Item::C1(C1::ApplicationProgramCommand),
				parse(b"\x9F").unwrap().1);

			assert_eq!(Item::C1(C1::ApplicationProgramCommand),
				parse(b"\x1B\x5F").unwrap().1);
		}
	}

	mod csi {
		pub use terminal::escape::*;

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
		fn hsv() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
				CSI::MoveTo(0, 0))),
				parse(b"\x1B[f").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
				CSI::MoveTo(1, 2))),
				parse(b"\x1B[2;3f").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
				CSI::MoveTo(0, 2))),
				parse(b"\x1B[;3f").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
				CSI::MoveTo(1, 0))),
				parse(b"\x1B[2;f").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
				CSI::MoveTo(0, 0))),
				parse(b"\x1B[;f").unwrap().1);
		}

		#[test]
		fn aux() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
				CSI::AuxPort(true))),
				parse(b"\x1B[5i").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
				CSI::AuxPort(false))),
				parse(b"\x1B[6i").unwrap().1);
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
				CSI::SaveCursorPosition)),
				parse(b"\x1B[s").unwrap().1);
		}

		#[test]
		fn rcp() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(
				CSI::RestoreCursorPosition)),
				parse(b"\x1B[u").unwrap().1);
		}
	}

	mod sgr {
		pub use terminal::escape::*;

		#[test]
		fn reset() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Reset]))),
				parse(b"\x1B[0m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Reset]))),
				parse(b"\x1B[m").unwrap().1);
		}

		#[test]
		fn font() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Font(SGR::Weight::Bold)]))),
				parse(b"\x1B[1m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Font(SGR::Weight::Faint)]))),
				parse(b"\x1B[2m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Font(SGR::Weight::Normal)]))),
				parse(b"\x1B[22m").unwrap().1);
		}

		#[test]
		fn italic() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Italic(true)]))),
				parse(b"\x1B[3m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Italic(false)]))),
				parse(b"\x1B[23m").unwrap().1);
		}

		#[test]
		fn underline() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Underline(true)]))),
				parse(b"\x1B[4m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Underline(false)]))),
				parse(b"\x1B[24m").unwrap().1);
		}

		#[test]
		fn blink() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Blink(true)]))),
				parse(b"\x1B[5m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Blink(true)]))),
				parse(b"\x1B[6m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Blink(false)]))),
				parse(b"\x1B[25m").unwrap().1);
		}

		#[test]
		fn reverse() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Reverse(true)]))),
				parse(b"\x1B[7m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Reverse(false)]))),
				parse(b"\x1B[27m").unwrap().1);
		}

		#[test]
		fn invisible() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Invisible(true)]))),
				parse(b"\x1B[8m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Invisible(false)]))),
				parse(b"\x1B[28m").unwrap().1);
		}

		#[test]
		fn struck() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Struck(true)]))),
				parse(b"\x1B[9m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Struck(false)]))),
				parse(b"\x1B[29m").unwrap().1);
		}

		#[test]
		fn foreground() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Foreground(SGR::Color::Default)]))),
				parse(b"\x1B[38;0m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Foreground(SGR::Color::Default)]))),
				parse(b"\x1B[39m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Foreground(SGR::Color::Transparent)]))),
				parse(b"\x1B[38;1m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Foreground(SGR::Color::Index(0))]))),
				parse(b"\x1B[30m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Foreground(SGR::Color::Index(7))]))),
				parse(b"\x1B[37m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Foreground(SGR::Color::Rgb(255, 0, 127))]))),
				parse(b"\x1B[38;2;255;;127m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Foreground(SGR::Color::Index(235))]))),
				parse(b"\x1B[38;5;235m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Foreground(SGR::Color::Index(8))]))),
				parse(b"\x1B[90m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Foreground(SGR::Color::Index(15))]))),
				parse(b"\x1B[97m").unwrap().1);
		}

		#[test]
		fn background() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Background(SGR::Color::Default)]))),
				parse(b"\x1B[48;0m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Background(SGR::Color::Default)]))),
				parse(b"\x1B[49m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Background(SGR::Color::Transparent)]))),
				parse(b"\x1B[48;1m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Background(SGR::Color::Index(0))]))),
				parse(b"\x1B[40m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Background(SGR::Color::Index(7))]))),
				parse(b"\x1B[47m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Background(SGR::Color::Rgb(255, 0, 127))]))),
				parse(b"\x1B[48;2;255;;127m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Background(SGR::Color::Index(235))]))),
				parse(b"\x1B[48;5;235m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Background(SGR::Color::Index(8))]))),
				parse(b"\x1B[100m").unwrap().1);

			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Background(SGR::Color::Index(15))]))),
				parse(b"\x1B[107m").unwrap().1);
		}

		#[test]
		fn sequence() {
			assert_eq!(Item::C1(C1::ControlSequenceIntroducer(CSI::SelectGraphicalRendition(vec![
				SGR::Foreground(SGR::Color::Rgb(0, 255, 127)),
				SGR::Background(SGR::Color::Rgb(127, 255, 0))
			]))),

			parse(b"\x1B[38;2;0;255;127;48;2;127;255;0m").unwrap().1);
		}
	}
}
