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

use std::io::{self, Write};
use std::str;
use control::{self, Format};

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum C1<'a> {
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
	DeviceControlString(&'a str),
	PrivateUseOne,
	PrivateUseTwo,
	SetTransmitState,
	CancelCharacter,
	MessageWaiting,
	StartProtectedArea,
	EndProtectedArea,
	String(&'a str),
	// TODO: this should contain the value.
	SingleGraphicCharacter,
	SingleCharacter(&'a str),
	ControlSequence(control::CSI::T),
	OperatingSystemCommand(&'a str),
	PrivacyMessage(&'a str),
	ApplicationProgramCommand(&'a str),
}

use self::C1::*;

impl<'a> Format for C1<'a> {
	fn fmt<W: Write>(&self, mut f: W, wide: bool) -> io::Result<()> {
		macro_rules! write {
			($code:expr) => (
				if wide {
					try!(f.write_all(&[0x1B, $code - 0x40]));
				}
				else {
					try!(f.write_all(&[$code]));
				}
			);
		}

		macro_rules! string {
			($string:expr) => (
				try!(f.write_all($string.as_bytes()));

				if wide {
					try!(f.write_all(&[0x1B, 0x9C - 0x40]));
				}
				else {
					try!(f.write_all(&[0x9C]));
				}
			)
		}

		match *self {
			PaddingCharacter =>
				write!(0x80),

			HighOctetPreset =>
				write!(0x81),

			BreakPermittedHere =>
				write!(0x82),

			NoBreakHere =>
				write!(0x83),

			Index =>
				write!(0x84),

			NextLine =>
				write!(0x85),

			StartSelectedArea =>
				write!(0x86),

			EndSelectedArea =>
				write!(0x87),

			HorizontalTabulationSet =>
				write!(0x88),

			HorizontalTabulationWithJustification =>
				write!(0x89),

			VerticalTabulationSet =>
				write!(0x8A),

			PartialLineDown =>
				write!(0x8B),

			PartialLineUp =>
				write!(0x8C),

			ReverseIndex =>
				write!(0x8D),

			SingleShiftTwo =>
				write!(0x8E),

			SingleShiftThree =>
				write!(0x8F),

			DeviceControlString(string) => {
				write!(0x90);
				string!(string);
			}

			PrivateUseOne =>
				write!(0x91),

			PrivateUseTwo =>
				write!(0x92),

			SetTransmitState =>
				write!(0x93),

			CancelCharacter =>
				write!(0x94),

			MessageWaiting =>
				write!(0x95),

			StartProtectedArea =>
				write!(0x96),

			EndProtectedArea =>
				write!(0x97),

			String(string) => {
				write!(0x98);
				string!(string);
			}

			SingleGraphicCharacter =>
				write!(0x99),

			SingleCharacter(string) => {
				write!(0x9A);
				string!(string);
			}

			ControlSequence(ref value) => {
				try!(value.fmt(f, wide));
			}

			OperatingSystemCommand(string) => {
				write!(0x9D);
				string!(string);
			}

			PrivacyMessage(string) => {
				write!(0x9E);
				string!(string);
			}

			ApplicationProgramCommand(string) => {
				write!(0x9F);
				string!(string);
			}
		}

		Ok(())
	}
}

named!(pub parse<C1>,
	alt!(PAD | HOP | BPH | NBH | IND | NEL | SSA | ESA | HTS | HTJ | VTS |
	     PLD | PLU | RI | SS2 | SS3 | DCS | PU1 | PU2 | STS | CCH | MW | SPA |
	     EPA | SOS | SGCI | SCI | CSI | OSC | PM | APC));

fn is_printable(b: u8) -> bool {
	(b >= 0x08 && b <= 0x0D) || (b >= 0x20 && b <= 0x7E)
}

named!(string<&str>,
	map!(terminated!(take_while!(is_printable), ST),
		|s| unsafe { str::from_utf8_unchecked(s) }));

named!(PAD<C1>,
	value!(PaddingCharacter,
		alt!(tag!(b"\x80") | tag!(b"\x1B\x40"))));

named!(HOP<C1>,
	value!(HighOctetPreset,
		alt!(tag!(b"\x81") | tag!(b"\x1B\x41"))));

named!(BPH<C1>,
	value!(BreakPermittedHere,
		alt!(tag!(b"\x82") | tag!(b"\x1B\x42"))));

named!(NBH<C1>,
	value!(NoBreakHere,
		alt!(tag!(b"\x83") | tag!(b"\x1B\x43"))));

named!(IND<C1>,
	value!(Index,
		alt!(tag!(b"\x84") | tag!(b"\x1B\x44"))));

named!(NEL<C1>,
	value!(NextLine,
		alt!(tag!(b"\x85") | tag!(b"\x1B\x45"))));

named!(SSA<C1>,
	value!(StartSelectedArea,
		alt!(tag!(b"\x86") | tag!(b"\x1B\x46"))));

named!(ESA<C1>,
	value!(EndSelectedArea,
		alt!(tag!(b"\x87") | tag!(b"\x1B\x47"))));

named!(HTS<C1>,
	value!(HorizontalTabulationSet,
		alt!(tag!(b"\x88") | tag!(b"\x1B\x48"))));

named!(HTJ<C1>,
	value!(HorizontalTabulationWithJustification,
		alt!(tag!(b"\x89") | tag!(b"\x1B\x49"))));

named!(VTS<C1>,
	value!(VerticalTabulationSet,
		alt!(tag!(b"\x8A") | tag!(b"\x1B\x4A"))));

named!(PLD<C1>,
	value!(PartialLineDown,
		alt!(tag!(b"\x8B") | tag!(b"\x1B\x4B"))));

named!(PLU<C1>,
	value!(PartialLineUp,
		alt!(tag!(b"\x8C") | tag!(b"\x1B\x4C"))));

named!(RI<C1>,
	value!(ReverseIndex,
		alt!(tag!(b"\x8D") | tag!(b"\x1B\x4D"))));

named!(SS2<C1>,
	value!(SingleShiftTwo,
		alt!(tag!(b"\x8E") | tag!(b"\x1B\x4E"))));

named!(SS3<C1>,
	value!(SingleShiftThree,
		alt!(tag!(b"\x8F") | tag!(b"\x1B\x4F"))));

named!(DCS<C1>,
	chain!(
		alt!(tag!(b"\x90") | tag!(b"\x1B\x50")) ~
		string: string,
		
		|| DeviceControlString(string)));

named!(PU1<C1>,
	value!(PrivateUseOne,
		alt!(tag!(b"\x91") | tag!(b"\x1B\x51"))));

named!(PU2<C1>,
	value!(PrivateUseTwo,
		alt!(tag!(b"\x92") | tag!(b"\x1B\x52"))));

named!(STS<C1>,
	value!(SetTransmitState,
		alt!(tag!(b"\x93") | tag!(b"\x1B\x53"))));

named!(CCH<C1>,
	value!(CancelCharacter,
		alt!(tag!(b"\x94") | tag!(b"\x1B\x54"))));

named!(MW<C1>,
	value!(MessageWaiting,
		alt!(tag!(b"\x95") | tag!(b"\x1B\x55"))));

named!(SPA<C1>,
	value!(StartProtectedArea,
		alt!(tag!(b"\x96") | tag!(b"\x1B\x56"))));

named!(EPA<C1>,
	value!(EndProtectedArea,
		alt!(tag!(b"\x97") | tag!(b"\x1B\x57"))));

named!(SOS<C1>,
	chain!(
		alt!(tag!(b"\x98") | tag!(b"\x1B\x58")) ~
		string: string,
		
		|| String(string)));

named!(SGCI<C1>,
	value!(SingleGraphicCharacter,
		alt!(tag!(b"\x99") | tag!(b"\x1B\x59"))));

named!(SCI<C1>,
	chain!(
		alt!(tag!(b"\x9A") | tag!(b"\x1B\x5A")) ~
		string: string,

		|| SingleCharacter(string)));

named!(CSI<C1>,
	chain!(
		alt!(tag!(b"\x9B") | tag!(b"\x1B\x5B")) ~
		res: call!(control::CSI::parse),

		|| ControlSequence(res)));

named!(ST,
	alt!(tag!(b"\x9C") | tag!(b"\x1B\x5C")));

named!(OSC<C1>,
	chain!(
		alt!(tag!(b"\x9D") | tag!(b"\x1B\x5D")) ~
		string: string,

		|| OperatingSystemCommand(string)));

named!(PM<C1>,
	chain!(
		alt!(tag!(b"\x9E") | tag!(b"\x1B\x5E")) ~
		string: string,

		|| PrivacyMessage(string)));

named!(APC<C1>,
	chain!(
		alt!(tag!(b"\x9F") | tag!(b"\x1B\x5F")) ~
		string: string,

		|| ApplicationProgramCommand(string)));

pub mod shim {
	pub use super::C1 as T;
	pub use super::C1::*;
	pub use super::parse;
}

#[cfg(test)]
mod test {
	mod parse {
		pub use control::*;

		macro_rules! test {
			($string:expr => $item:expr) => (
				assert_eq!(Item::C1($item),
					parse($string).unwrap().1);
			);
		}

		#[test]
		fn pad() {
			test!(b"\x80" =>
				C1::PaddingCharacter);

			test!(b"\x1B\x40" =>
				C1::PaddingCharacter);
		}

		#[test]
		fn hop() {
			test!(b"\x81" =>
				C1::HighOctetPreset);

			test!(b"\x1B\x41" =>
				C1::HighOctetPreset);
		}

		#[test]
		fn bph() {
			test!(b"\x82" =>
				C1::BreakPermittedHere);

			test!(b"\x1B\x42" =>
				C1::BreakPermittedHere);
		}

		#[test]
		fn nbh() {
			test!(b"\x83" =>
				C1::NoBreakHere);

			test!(b"\x1B\x43" =>
				C1::NoBreakHere);
		}

		#[test]
		fn ind() {
			test!(b"\x84" =>
				C1::Index);

			test!(b"\x1B\x44" =>
				C1::Index);
		}

		#[test]
		fn nel() {
			test!(b"\x85" =>
				C1::NextLine);

			test!(b"\x1B\x45" =>
				C1::NextLine);
		}

		#[test]
		fn ssa() {
			test!(b"\x86" =>
				C1::StartSelectedArea);

			test!(b"\x1B\x46" =>
				C1::StartSelectedArea);
		}

		#[test]
		fn esa() {
			test!(b"\x87" =>
				C1::EndSelectedArea);

			test!(b"\x1B\x47" =>
				C1::EndSelectedArea);
		}

		#[test]
		fn hts() {
			test!(b"\x88" =>
				C1::HorizontalTabulationSet);

			test!(b"\x1B\x48" =>
				C1::HorizontalTabulationSet);
		}

		#[test]
		fn htj() {
			test!(b"\x89" =>
				C1::HorizontalTabulationWithJustification);

			test!(b"\x1B\x49" =>
				C1::HorizontalTabulationWithJustification);
		}

		#[test]
		fn vts() {
			test!(b"\x8A" =>
				C1::VerticalTabulationSet);

			test!(b"\x1B\x4A" =>
				C1::VerticalTabulationSet);
		}

		#[test]
		fn pld() {
			test!(b"\x8B" =>
				C1::PartialLineDown);

			test!(b"\x1B\x4B" =>
				C1::PartialLineDown);
		}

		#[test]
		fn plu() {
			test!(b"\x8C" =>
				C1::PartialLineUp);

			test!(b"\x1B\x4C" =>
				C1::PartialLineUp);
		}

		#[test]
		fn ri() {
			test!(b"\x8D" =>
				C1::ReverseIndex);

			test!(b"\x1B\x4D" =>
				C1::ReverseIndex);
		}

		#[test]
		fn ss2() {
			test!(b"\x8E" =>
				C1::SingleShiftTwo);

			test!(b"\x1B\x4E" =>
				C1::SingleShiftTwo);
		}

		#[test]
		fn ss3() {
			test!(b"\x8F" =>
				C1::SingleShiftThree);

			test!(b"\x1B\x4F" =>
				C1::SingleShiftThree);
		}

		#[test]
		fn dcs() {
			test!(b"\x90foo\x9C" =>
				C1::DeviceControlString("foo"));

			test!(b"\x1B\x50foo\x1B\x5C" =>
				C1::DeviceControlString("foo"));
		}

		#[test]
		fn pu1() {
			test!(b"\x91" =>
				C1::PrivateUseOne);

			test!(b"\x1B\x51" =>
				C1::PrivateUseOne);
		}

		#[test]
		fn pu2() {
			test!(b"\x92" =>
				C1::PrivateUseTwo);

			test!(b"\x1B\x52" =>
				C1::PrivateUseTwo);
		}

		#[test]
		fn sts() {
			test!(b"\x93" =>
				C1::SetTransmitState);

			test!(b"\x1B\x53" =>
				C1::SetTransmitState);
		}

		#[test]
		fn cch() {
			test!(b"\x94" =>
				C1::CancelCharacter);

			test!(b"\x1B\x54" =>
				C1::CancelCharacter);
		}

		#[test]
		fn mw() {
			test!(b"\x95" =>
				C1::MessageWaiting);

			test!(b"\x1B\x55" =>
				C1::MessageWaiting);
		}

		#[test]
		fn spa() {
			test!(b"\x96" =>
				C1::StartProtectedArea);

			test!(b"\x1B\x56" =>
				C1::StartProtectedArea);
		}

		#[test]
		fn epa() {
			test!(b"\x97" =>
				C1::EndProtectedArea);

			test!(b"\x1B\x57" =>
				C1::EndProtectedArea);
		}

		#[test]
		fn sos() {
			test!(b"\x98foo\x9C" =>
				C1::String("foo"));

			test!(b"\x1B\x58foo\x1B\x5C" =>
				C1::String("foo"));
		}

		#[test]
		fn sgci() {
			test!(b"\x99" =>
				C1::SingleGraphicCharacter);

			test!(b"\x1B\x59" =>
				C1::SingleGraphicCharacter);
		}

		#[test]
		fn sci() {
			test!(b"\x9Afoo\x9C" =>
				C1::SingleCharacter("foo"));

			test!(b"\x1B\x5Afoo\x1B\x5C" =>
				C1::SingleCharacter("foo"));
		}

		#[test]
		fn osc() {
			test!(b"\x9Dfoo\x9C" =>
				C1::OperatingSystemCommand("foo"));

			test!(b"\x1B\x5Dfoo\x1B\x5C" =>
				C1::OperatingSystemCommand("foo"));
		}

		#[test]
		fn pn() {
			test!(b"\x9Efoo\x9C" =>
				C1::PrivacyMessage("foo"));

			test!(b"\x1B\x5Efoo\x1B\x5C" =>
				C1::PrivacyMessage("foo"));
		}

		#[test]
		fn apc() {
			test!(b"\x9Ffoo\x9C" =>
				C1::ApplicationProgramCommand("foo"));

			test!(b"\x1B\x5Ffoo\x1B\x5C" =>
				C1::ApplicationProgramCommand("foo"));
		}
	}

	mod format {
		pub use control::*;

		macro_rules! test {
			($code:expr) => (
				let item = Item::C1($code);

				let mut result = vec![];
				item.fmt(&mut result, true).unwrap();
				assert_eq!(item, parse(&result).unwrap().1);

				let mut result = vec![];
				item.fmt(&mut result, false).unwrap();
				assert_eq!(item, parse(&result).unwrap().1);
			);
		}

		#[test]
		fn pad() {
			test!(C1::PaddingCharacter);
		}

		#[test]
		fn hop() {
			test!(C1::HighOctetPreset);
		}

		#[test]
		fn bph() {
			test!(C1::BreakPermittedHere);
		}

		#[test]
		fn nbh() {
			test!(C1::NoBreakHere);
		}

		#[test]
		fn ind() {
			test!(C1::Index);
		}

		#[test]
		fn nel() {
			test!(C1::NextLine);
		}

		#[test]
		fn ssa() {
			test!(C1::StartSelectedArea);
		}

		#[test]
		fn esa() {
			test!(C1::EndSelectedArea);
		}

		#[test]
		fn hts() {
			test!(C1::HorizontalTabulationSet);
		}

		#[test]
		fn htj() {
			test!(C1::HorizontalTabulationWithJustification);
		}

		#[test]
		fn vts() {
			test!(C1::VerticalTabulationSet);
		}

		#[test]
		fn pld() {
			test!(C1::PartialLineDown);
		}

		#[test]
		fn plu() {
			test!(C1::PartialLineUp);
		}

		#[test]
		fn ri() {
			test!(C1::ReverseIndex);
		}

		#[test]
		fn ss2() {
			test!(C1::SingleShiftTwo);
		}

		#[test]
		fn ss3() {
			test!(C1::SingleShiftThree);
		}

		#[test]
		fn dcs() {
			test!(C1::DeviceControlString("foo"));
		}

		#[test]
		fn pu1() {
			test!(C1::PrivateUseOne);
		}

		#[test]
		fn pu2() {
			test!(C1::PrivateUseTwo);
		}

		#[test]
		fn sts() {
			test!(C1::SetTransmitState);
		}

		#[test]
		fn cch() {
			test!(C1::CancelCharacter);
		}

		#[test]
		fn mw() {
			test!(C1::MessageWaiting);
		}

		#[test]
		fn spa() {
			test!(C1::StartProtectedArea);
		}

		#[test]
		fn epa() {
			test!(C1::EndProtectedArea);
		}

		#[test]
		fn sos() {
			test!(C1::String("foo"));
		}

		#[test]
		fn sgci() {
			test!(C1::SingleGraphicCharacter);
		}

		#[test]
		fn sci() {
			test!(C1::SingleCharacter("foo"));
		}

		#[test]
		fn osc() {
			test!(C1::OperatingSystemCommand("foo"));
		}

		#[test]
		fn pn() {
			test!(C1::PrivacyMessage("foo"));
		}

		#[test]
		fn apc() {
			test!(C1::ApplicationProgramCommand("foo"));
		}
	}
}
