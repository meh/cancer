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
use control::{self, Format};

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum C1 {
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
	ControlSequenceIntroducer(control::CSI::T),
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

use self::C1::*;

impl Format for C1 {
	fn fmt<W: Write>(&self, mut f: W, wide: bool) -> io::Result<()> {
		macro_rules! write {
			($code:expr) => (
				if wide {
					f.write_all(&[0x1B, $code - 0x40])
				}
				else {
					f.write_all(&[$code])
				}
			);
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

			DeviceControlString =>
				write!(0x90),

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

			StartString =>
				write!(0x98),

			SingleGraphicCharacterIntroducer =>
				write!(0x99),

			SingleCharacterIntroducer =>
				write!(0x9A),

			ControlSequenceIntroducer(ref value) =>
				value.fmt(f, wide),

			StringTerminator =>
				write!(0x9C),

			OperatingSystemCommand =>
				write!(0x9D),

			PrivacyMessage =>
				write!(0x9E),

			ApplicationProgramCommand =>
				write!(0x9F),
		}
	}
}

named!(pub parse<C1>,
	alt!(PAD | HOP | BPH | NBH | IND | NEL | SSA | ESA | HTS | HTJ | VTS |
	     PLD | PLU | RI | SS2 | SS3 | DCS | PU1 | PU2 | STS | CCH | MW | SPA |
	     EPA | SOS | SGCI | SCI | CSI | ST | OSC | PM | APC));

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
	value!(DeviceControlString,
		alt!(tag!(b"\x90") | tag!(b"\x1B\x50"))));

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
	value!(StartString,
		alt!(tag!(b"\x98") | tag!(b"\x1B\x58"))));

named!(SGCI<C1>,
	value!(SingleGraphicCharacterIntroducer,
		alt!(tag!(b"\x99") | tag!(b"\x1B\x59"))));

named!(SCI<C1>,
	value!(SingleCharacterIntroducer,
		alt!(tag!(b"\x9A") | tag!(b"\x1B\x5A"))));

named!(CSI<C1>,
	chain!(
		alt!(tag!(b"\x9B") | tag!(b"\x1B\x5B")) ~
		res: call!(control::CSI::parse),

		|| ControlSequenceIntroducer(res)));

named!(ST<C1>,
	value!(StringTerminator,
		alt!(tag!(b"\x9C") | tag!(b"\x1B\x5C"))));

named!(OSC<C1>,
	value!(OperatingSystemCommand,
		alt!(tag!(b"\x9D") | tag!(b"\x1B\x5D"))));

named!(PM<C1>,
	value!(PrivacyMessage,
		alt!(tag!(b"\x9E") | tag!(b"\x1B\x5E"))));

named!(APC<C1>,
	value!(ApplicationProgramCommand,
		alt!(tag!(b"\x9F") | tag!(b"\x1B\x5F"))));

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
			($id:expr => $attr:expr) => (
				assert_eq!(Item::C1($attr),
					parse(&[$id]).unwrap().1);

				assert_eq!(Item::C1($attr),
					parse(&[0x1B, $id - 0x40]).unwrap().1);
			);
		}

		#[test]
		fn pad() {
			test!(0x80 =>
				C1::PaddingCharacter);
		}

		#[test]
		fn hop() {
			test!(0x81 =>
				C1::HighOctetPreset);
		}

		#[test]
		fn bph() {
			test!(0x82 =>
				C1::BreakPermittedHere);
		}

		#[test]
		fn nbh() {
			test!(0x83 =>
				C1::NoBreakHere);
		}

		#[test]
		fn ind() {
			test!(0x84 =>
				C1::Index);
		}

		#[test]
		fn nel() {
			test!(0x85 =>
				C1::NextLine);
		}

		#[test]
		fn ssa() {
			test!(0x86 =>
				C1::StartSelectedArea);
		}

		#[test]
		fn esa() {
			test!(0x87 =>
				C1::EndSelectedArea);
		}

		#[test]
		fn hts() {
			test!(0x88 =>
				C1::HorizontalTabulationSet);
		}

		#[test]
		fn htj() {
			test!(0x89 =>
				C1::HorizontalTabulationWithJustification);
		}

		#[test]
		fn vts() {
			test!(0x8A =>
				C1::VerticalTabulationSet);
		}

		#[test]
		fn pld() {
			test!(0x8B =>
				C1::PartialLineDown);
		}

		#[test]
		fn plu() {
			test!(0x8C =>
				C1::PartialLineUp);
		}

		#[test]
		fn ri() {
			test!(0x8D =>
				C1::ReverseIndex);
		}

		#[test]
		fn ss2() {
			test!(0x8E =>
				C1::SingleShiftTwo);
		}

		#[test]
		fn ss3() {
			test!(0x8F =>
				C1::SingleShiftThree);
		}

		#[test]
		fn dcs() {
			test!(0x90 =>
				C1::DeviceControlString);
		}

		#[test]
		fn pu1() {
			test!(0x91 =>
				C1::PrivateUseOne);
		}

		#[test]
		fn pu2() {
			test!(0x92 =>
				C1::PrivateUseTwo);
		}

		#[test]
		fn sts() {
			test!(0x93 =>
				C1::SetTransmitState);
		}

		#[test]
		fn cch() {
			test!(0x94 =>
				C1::CancelCharacter);
		}

		#[test]
		fn mw() {
			test!(0x95 =>
				C1::MessageWaiting);
		}

		#[test]
		fn spa() {
			test!(0x96 =>
				C1::StartProtectedArea);
		}

		#[test]
		fn epa() {
			test!(0x97 =>
				C1::EndProtectedArea);
		}

		#[test]
		fn sos() {
			test!(0x98 =>
				C1::StartString);
		}

		#[test]
		fn sgci() {
			test!(0x99 =>
				C1::SingleGraphicCharacterIntroducer);
		}

		#[test]
		fn sci() {
			test!(0x9A =>
				C1::SingleCharacterIntroducer);
		}

		#[test]
		fn st() {
			test!(0x9C =>
				C1::StringTerminator);
		}

		#[test]
		fn osc() {
			test!(0x9D =>
				C1::OperatingSystemCommand);
		}

		#[test]
		fn pn() {
			test!(0x9E =>
				C1::PrivacyMessage);
		}

		#[test]
		fn apc() {
			test!(0x9F =>
				C1::ApplicationProgramCommand);
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
			test!(C1::DeviceControlString);
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
			test!(C1::StartString);
		}

		#[test]
		fn sgci() {
			test!(C1::SingleGraphicCharacterIntroducer);
		}

		#[test]
		fn sci() {
			test!(C1::SingleCharacterIntroducer);
		}

		#[test]
		fn st() {
			test!(C1::StringTerminator);
		}

		#[test]
		fn osc() {
			test!(C1::OperatingSystemCommand);
		}

		#[test]
		fn pn() {
			test!(C1::PrivacyMessage);
		}

		#[test]
		fn apc() {
			test!(C1::ApplicationProgramCommand);
		}
	}
}
