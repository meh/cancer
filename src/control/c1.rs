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
					f.write_all(&[0x1B, $code - 0x80])
				}
				else {
					f.write_all(&[$code])
				}
			);
		}

		match *self {
			Delete =>
				write!(0x7f),

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
	alt!(DEL | PAD | HOP | BPH | NBH | IND | NEL | SSA | ESA | HTS | HTJ | VTS |
	     PLD | PLU | RI | SS2 | SS3 | DCS | PU1 | PU2 | STS | CCH | MW | SPA |
	     EPA | SOS | SGCI | SCI | CSI | ST | OSC | PM | APC));

named!(DEL<C1>,
	value!(Delete,
		char!(0x7F)));

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
	pub use control::*;

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
