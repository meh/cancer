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
use nom::{IResult, rest};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Item<'a> {
	Insert(&'a str),

	CSI(CSI::T),
	SGR(SGR::T),
	C0(C0::T),
	C1(C1::T),
}

named!(pub parse<Item>,
	alt!(control | insert));

named!(insert<Item>,
	map!(rest, |s| Item::Insert(str::from_utf8(s).unwrap().into())));

named!(pub control<Item>,
	alt!(
		map!(CSI::parse, |c| match c {
			CSI::SelectGraphicalRendition(value) =>
				Item::SGR(value),

			value =>
				Item::CSI(value)
		})
		| map!(C0::parse, |c| Item::C0(c))
		| map!(C1::parse, |c| Item::C1(c))));

pub mod SGR {
	use nom::{self, ErrorKind};

	#[derive(Eq, PartialEq, Copy, Clone, Debug)]
	pub enum T {
		Reset,
	}

	pub use self::T::*;

	pub fn parse<'a, 'b>(args: &'b [Option<u32>]) -> Result<T, nom::Err<&'a [u8]>> {
		Err(nom::Err::Code(ErrorKind::Custom(9004)))
	}
}

pub mod CSI {
	use std::str;
	use std::u32;
	use nom::{self, ErrorKind, is_digit};

	#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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
		SelectGraphicalRendition(super::SGR::T),
		AuxPort(bool),
		DeviceStatusReport,
		SaveCursorPosition,
		RestoreCursorPosition,
		// TODO: Private ones.
	}

	#[derive(Eq, PartialEq, Copy, Clone, Debug)]
	pub enum Erase {
		ToEnd,
		ToBeginning,
		All,
	}

	impl Erase {
		fn from<'a>(value: u32) -> Result<Self, nom::Err<&'a [u8]>> {
			match value {
				0 => Ok(Erase::ToEnd),
				1 => Ok(Erase::ToBeginning),
				2 => Ok(Erase::All),
				_ => Err(nom::Err::Code(ErrorKind::Custom(9002))),
			}
		}
	}

	pub use self::T::*;

	const DIGIT: &[u8] = b"0123456789";

	named!(pub parse<T>,
		chain!(
			tag!(b"\x1B[") ~
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
			fn $name<'a, 'b>(i: &'a [u8], $params: &'b [Option<u32>]) -> nom::IResult<&'a [u8], T, u32> {
				if $params.len() <= $n {
					$submac!(i, $($args)*)
				}
				else {
					nom::IResult::Error(nom::Err::Code(ErrorKind::Custom(9001)))
				}
			}
		);

		($name:ident, $submac:ident!( $($args:tt)* )) => (
			fn $name<'a, 'b>(i: &'a [u8], args: &'b [Option<u32>]) -> nom::IResult<&'a [u8], T, u32> {
				if args.is_empty() {
					$submac!(i, $($args)*)
				}
				else {
					nom::IResult::Error(nom::Err::Code(ErrorKind::Custom(9001)))
				}
			}
		);
	}

	macro_rules! arg {
		($args:ident[$index:tt] => $default:tt) => (
			$args.get($index).and_then(|v| *v).unwrap_or($default)
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

	with_args!(SGR<9, args>,
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
		map!(char!('s'), |_|
			RestoreCursorPosition));
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
		value!(Null, char!(0x00)));
	
	named!(SOH<T>,
		value!(StartHeading, char!(0x01)));
	
	named!(STX<T>,
		value!(StartText, char!(0x02)));
	
	named!(ETX<T>,
		value!(EndText, char!(0x03)));
	
	named!(EOT<T>,
		value!(EndTransmission, char!(0x04)));
	
	named!(ENQ<T>,
		value!(Enquiry, char!(0x05)));
	
	named!(ACK<T>,
		value!(Acknowledge, char!(0x06)));
	
	named!(BEL<T>,
		value!(Bell, char!(0x07)));
	
	named!(BS<T>,
		value!(Backspace, char!(0x08)));
	
	named!(HT<T>,
		value!(HorizontalTabulation, char!(0x09)));
	
	named!(LF<T>,
		value!(LineFeed, char!(0x0A)));
	
	named!(VT<T>,
		value!(VerticalTabulation, char!(0x0B)));
	
	named!(FF<T>,
		value!(FormFeed, char!(0x0C)));
	
	named!(CR<T>,
		value!(CarriageReturn, char!(0x0D)));
	
	named!(SS<T>,
		value!(ShiftOut, char!(0x0E)));
	
	named!(SI<T>,
		value!(ShiftIn, char!(0x0F)));
	
	named!(DLE<T>,
		value!(DataLinkEscape, char!(0x10)));
	
	named!(DC1<T>,
		value!(DeviceControlOne, char!(0x11)));
	
	named!(DC2<T>,
		value!(DeviceControlTwo, char!(0x12)));
	
	named!(DC3<T>,
		value!(DeviceControlThree, char!(0x13)));
	
	named!(DC4<T>,
		value!(DeviceControlFour, char!(0x14)));
	
	named!(NAK<T>,
		value!(NegativeAcknowledge, char!(0x15)));
	
	named!(SYN<T>,
		value!(SynchronousIdle, char!(0x16)));
	
	named!(ETB<T>,
		value!(EndTransmissionBlock, char!(0x17)));
	
	named!(CAN<T>,
		value!(Cancel, char!(0x18)));
	
	named!(EM<T>,
		value!(EndMedium, char!(0x19)));
	
	named!(SUB<T>,
		value!(Substitute, char!(0x1A)));
	
	named!(ESC<T>,
		value!(Escape, char!(0x1B)));
	
	named!(FS<T>,
		value!(FileSeparator, char!(0x1C)));
	
	named!(GS<T>,
		value!(GroupSeparator, char!(0x1D)));
	
	named!(RS<T>,
		value!(RecordSeparator, char!(0x1E)));
	
	named!(US<T>,
		value!(UnitSeparator, char!(0x1F)));
}

mod C1 {
	#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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
		ControlSequenceIntroducer,
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
		alt!(DEL | PAD | BPH | NBH | IND | NEL | SSA | ESA | HTS | HTJ | VTS | PLD |
		     PLU | RI | SS2 | SS3 | DCS | PU1 | PU2 | STS | CCH | MW | SPA | EPA |
		     SOS | SGCI | SCI | CSI | ST | OSC | PM | APC));
	
	named!(DEL<T>,
		value!(Delete, char!(0x7F)));
	
	named!(PAD<T>,
		value!(PaddingCharacter, char!(0x80)));
	
	named!(HOP<T>,
		value!(HighOctetPreset, char!(0x81)));
	
	named!(BPH<T>,
		value!(BreakPermittedHere, char!(0x82)));
	
	named!(NBH<T>,
		value!(NoBreakHere, char!(0x83)));
	
	named!(IND<T>,
		value!(Index, char!(0x84)));
	
	named!(NEL<T>,
		value!(NextLine, char!(0x85)));
	
	named!(SSA<T>,
		value!(StartSelectedArea, char!(0x86)));
	
	named!(ESA<T>,
		value!(EndSelectedArea, char!(0x87)));
	
	named!(HTS<T>,
		value!(HorizontalTabulationSet, char!(0x88)));
	
	named!(HTJ<T>,
		value!(HorizontalTabulationWithJustification, char!(0x89)));
	
	named!(VTS<T>,
		value!(VerticalTabulationSet, char!(0x8A)));
	
	named!(PLD<T>,
		value!(PartialLineDown, char!(0x8B)));
	
	named!(PLU<T>,
		value!(PartialLineUp, char!(0x8C)));
	
	named!(RI<T>,
		value!(ReverseIndex, char!(0x8D)));
	
	named!(SS2<T>,
		value!(SingleShiftTwo, char!(0x8E)));
	
	named!(SS3<T>,
		value!(SingleShiftThree, char!(0x8F)));
	
	named!(DCS<T>,
		value!(DeviceControlString, char!(0x90)));
	
	named!(PU1<T>,
		value!(PrivateUseOne, char!(0x91)));
	
	named!(PU2<T>,
		value!(PrivateUseTwo, char!(0x92)));
	
	named!(STS<T>,
		value!(SetTransmitState, char!(0x93)));
	
	named!(CCH<T>,
		value!(CancelCharacter, char!(0x94)));
	
	named!(MW<T>,
		value!(MessageWaiting, char!(0x95)));
	
	named!(SPA<T>,
		value!(StartProtectedArea, char!(0x96)));
	
	named!(EPA<T>,
		value!(EndProtectedArea, char!(0x97)));
	
	named!(SOS<T>,
		value!(StartString, char!(0x98)));
	
	named!(SGCI<T>,
		value!(SingleGraphicCharacterIntroducer, char!(0x99)));
	
	named!(SCI<T>,
		value!(SingleCharacterIntroducer, char!(0x9A)));
	
	named!(CSI<T>,
		value!(ControlSequenceIntroducer, char!(0x9B)));
	
	named!(ST<T>,
		value!(StringTerminator, char!(0x9C)));
	
	named!(OSC<T>,
		value!(OperatingSystemCommand, char!(0x9D)));
	
	named!(PM<T>,
		value!(PrivacyMessage, char!(0x9E)));
	
	named!(APC<T>,
		value!(ApplicationProgramCommand, char!(0x9F)));
}

#[cfg(test)]
mod test {
	pub use super::*;

	#[test]
	fn csi() {
		println!("{:?}", CSI::parse(b"\x1B[23A"));
	}
}
