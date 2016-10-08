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
use control::Format;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum C0 {
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

use self::C0::*;

impl Format for C0 {
	fn fmt<W: Write>(&self, mut f: W, wide: bool) -> io::Result<()> {
		macro_rules! write {
			($code:expr) => (
				f.write_all(&[$code])
			);
		}

		match *self {
			Null =>
				write!(0x00),

			StartHeading =>
				write!(0x01),

			StartText =>
				write!(0x02),

			EndText =>
				write!(0x03),

			EndTransmission =>
				write!(0x04),

			Enquiry =>
				write!(0x05),

			Acknowledge =>
				write!(0x06),

			Bell =>
				write!(0x07),

			Backspace =>
				write!(0x08),

			HorizontalTabulation =>
				write!(0x09),

			LineFeed =>
				write!(0x0A),

			VerticalTabulation =>
				write!(0x0B),

			FormFeed =>
				write!(0x0C),

			CarriageReturn =>
				write!(0x0D),

			ShiftOut =>
				write!(0x0E),

			ShiftIn =>
				write!(0x0F),

			DataLinkEscape =>
				write!(0x10),

			DeviceControlOne =>
				write!(0x11),

			DeviceControlTwo =>
				write!(0x12),

			DeviceControlThree =>
				write!(0x13),

			DeviceControlFour =>
				write!(0x14),

			NegativeAcknowledge =>
				write!(0x15),

			SynchronousIdle =>
				write!(0x16),

			EndTransmissionBlock =>
				write!(0x17),

			Cancel =>
				write!(0x18),

			EndMedium =>
				write!(0x19),

			Substitute =>
				write!(0x1A),

			Escape =>
				write!(0x1B),

			FileSeparator =>
				write!(0x1C),

			GroupSeparator =>
				write!(0x1D),

			RecordSeparator =>
				write!(0x1E),

			UnitSeparator =>
				write!(0x1F),
		}
	}
}

named!(pub parse<C0>,
	alt!(NUL | SOH | STX | ETX | EOT | ENQ | ACK | BEL | BS | HT | LF | VT | FF |
	     CR | SS | SI | DLE | DC1 | DC2 | DC3 | DC4 | NAK | SYN | ETB | CAN | EM |
	     SUB | ESC | FS | GS | RS | US));

named!(NUL<C0>,
	value!(Null,
		char!(0x00)));

named!(SOH<C0>,
	value!(StartHeading,
		char!(0x01)));

named!(STX<C0>,
	value!(StartText,
		char!(0x02)));

named!(ETX<C0>,
	value!(EndText,
		char!(0x03)));

named!(EOT<C0>,
	value!(EndTransmission,
		char!(0x04)));

named!(ENQ<C0>,
	value!(Enquiry,
		char!(0x05)));

named!(ACK<C0>,
	value!(Acknowledge,
		char!(0x06)));

named!(BEL<C0>,
	value!(Bell,
		char!(0x07)));

named!(BS<C0>,
	value!(Backspace,
		char!(0x08)));

named!(HT<C0>,
	value!(HorizontalTabulation,
		char!(0x09)));

named!(LF<C0>,
	value!(LineFeed,
		char!(0x0A)));

named!(VT<C0>,
	value!(VerticalTabulation,
		char!(0x0B)));

named!(FF<C0>,
	value!(FormFeed,
		char!(0x0C)));

named!(CR<C0>,
	value!(CarriageReturn,
		char!(0x0D)));

named!(SS<C0>,
	value!(ShiftOut,
		char!(0x0E)));

named!(SI<C0>,
	value!(ShiftIn,
		char!(0x0F)));

named!(DLE<C0>,
	value!(DataLinkEscape,
		char!(0x10)));

named!(DC1<C0>,
	value!(DeviceControlOne,
		char!(0x11)));

named!(DC2<C0>,
	value!(DeviceControlTwo,
		char!(0x12)));

named!(DC3<C0>,
	value!(DeviceControlThree,
		char!(0x13)));

named!(DC4<C0>,
	value!(DeviceControlFour,
		char!(0x14)));

named!(NAK<C0>,
	value!(NegativeAcknowledge,
		char!(0x15)));

named!(SYN<C0>,
	value!(SynchronousIdle,
		char!(0x16)));

named!(ETB<C0>,
	value!(EndTransmissionBlock,
		char!(0x17)));

named!(CAN<C0>,
	value!(Cancel,
		char!(0x18)));

named!(EM<C0>,
	value!(EndMedium,
		char!(0x19)));

named!(SUB<C0>,
	value!(Substitute,
		char!(0x1A)));

named!(ESC<C0>,
	value!(Escape,
		char!(0x1B)));

named!(FS<C0>,
	value!(FileSeparator,
		char!(0x1C)));

named!(GS<C0>,
	value!(GroupSeparator,
		char!(0x1D)));

named!(RS<C0>,
	value!(RecordSeparator,
		char!(0x1E)));

named!(US<C0>,
	value!(UnitSeparator,
		char!(0x1F)));

pub mod shim {
	pub use super::C0 as T;
	pub use super::C0::*;
	pub use super::parse;
}

#[cfg(test)]
mod test {
	mod parse {
		pub use control::*;

		macro_rules! test {
			($id:expr => $attr:expr) => (
				assert_eq!(Item::C0($attr),
					parse(&[$id]).unwrap().1);
			);
		}

		#[test]
		fn nul() {
			test!(0x00 =>
				C0::Null);
		}

		#[test]
		fn soh() {
			test!(0x01 =>
				C0::StartHeading);
		}

		#[test]
		fn stx() {
			test!(0x02 =>
				C0::StartText);
		}

		#[test]
		fn etx() {
			test!(0x03 =>
				C0::EndText);
		}

		#[test]
		fn eot() {
			test!(0x04 =>
				C0::EndTransmission);
		}

		#[test]
		fn enq() {
			test!(0x05 =>
				C0::Enquiry);
		}

		#[test]
		fn ack() {
			test!(0x06 =>
				C0::Acknowledge);
		}

		#[test]
		fn bel() {
			test!(0x07 =>
				C0::Bell);
		}

		#[test]
		fn bs() {
			test!(0x08 =>
				C0::Backspace);
		}

		#[test]
		fn ht() {
			test!(0x09 =>
				C0::HorizontalTabulation);
		}

		#[test]
		fn lf() {
			test!(0x0A =>
				C0::LineFeed);
		}

		#[test]
		fn vf() {
			test!(0x0B =>
				C0::VerticalTabulation);
		}

		#[test]
		fn ff() {
			test!(0x0C =>
				C0::FormFeed);
		}

		#[test]
		fn cr() {
			test!(0x0D =>
				C0::CarriageReturn);
		}

		#[test]
		fn ss() {
			test!(0x0E =>
				C0::ShiftOut);
		}

		#[test]
		fn si() {
			test!(0x0F =>
				C0::ShiftIn);
		}

		#[test]
		fn dle() {
			test!(0x10 =>
				C0::DataLinkEscape);
		}

		#[test]
		fn dc1() {
			test!(0x11 =>
				C0::DeviceControlOne);
		}

		#[test]
		fn dc2() {
			test!(0x12 =>
				C0::DeviceControlTwo);
		}

		#[test]
		fn dc3() {
			test!(0x13 =>
				C0::DeviceControlThree);
		}

		#[test]
		fn dc4() {
			test!(0x14 =>
				C0::DeviceControlFour);
		}

		#[test]
		fn nak() {
			test!(0x15 =>
				C0::NegativeAcknowledge);
		}

		#[test]
		fn syn() {
			test!(0x16 =>
				C0::SynchronousIdle);
		}

		#[test]
		fn etb() {
			test!(0x17 =>
				C0::EndTransmissionBlock);
		}

		#[test]
		fn can() {
			test!(0x18 =>
				C0::Cancel);
		}

		#[test]
		fn em() {
			test!(0x19 =>
				C0::EndMedium);
		}

		#[test]
		fn sub() {
			test!(0x1A =>
				C0::Substitute);
		}

		#[test]
		fn fs() {
			test!(0x1C =>
				C0::FileSeparator);
		}

		#[test]
		fn gs() {
			test!(0x1D =>
				C0::GroupSeparator);
		}

		#[test]
		fn rs() {
			test!(0x1E =>
				C0::RecordSeparator);
		}

		#[test]
		fn us() {
			test!(0x1F =>
				C0::UnitSeparator);
		}
	}

	mod format {
		pub use control::*;

		macro_rules! test {
			($code:expr) => (
				let item = Item::C0($code);

				let mut result = vec![];
				item.fmt(&mut result, true).unwrap();
				assert_eq!(item, parse(&result).unwrap().1);

				let mut result = vec![];
				item.fmt(&mut result, false).unwrap();
				assert_eq!(item, parse(&result).unwrap().1);
			);
		}

		#[test]
		fn nul() {
			test!(C0::Null);
		}

		#[test]
		fn soh() {
			test!(C0::StartHeading);
		}

		#[test]
		fn stx() {
			test!(C0::StartText);
		}

		#[test]
		fn etx() {
			test!(C0::EndText);
		}

		#[test]
		fn eot() {
			test!(C0::EndTransmission);
		}

		#[test]
		fn enq() {
			test!(C0::Enquiry);
		}

		#[test]
		fn ack() {
			test!(C0::Acknowledge);
		}

		#[test]
		fn bel() {
			test!(C0::Bell);
		}

		#[test]
		fn bs() {
			test!(C0::Backspace);
		}

		#[test]
		fn ht() {
			test!(C0::HorizontalTabulation);
		}

		#[test]
		fn lf() {
			test!(C0::LineFeed);
		}

		#[test]
		fn vf() {
			test!(C0::VerticalTabulation);
		}

		#[test]
		fn ff() {
			test!(C0::FormFeed);
		}

		#[test]
		fn cr() {
			test!(C0::CarriageReturn);
		}

		#[test]
		fn ss() {
			test!(C0::ShiftOut);
		}

		#[test]
		fn si() {
			test!(C0::ShiftIn);
		}

		#[test]
		fn dle() {
			test!(C0::DataLinkEscape);
		}

		#[test]
		fn dc1() {
			test!(C0::DeviceControlOne);
		}

		#[test]
		fn dc2() {
			test!(C0::DeviceControlTwo);
		}

		#[test]
		fn dc3() {
			test!(C0::DeviceControlThree);
		}

		#[test]
		fn dc4() {
			test!(C0::DeviceControlFour);
		}

		#[test]
		fn nak() {
			test!(C0::NegativeAcknowledge);
		}

		#[test]
		fn syn() {
			test!(C0::SynchronousIdle);
		}

		#[test]
		fn etb() {
			test!(C0::EndTransmissionBlock);
		}

		#[test]
		fn can() {
			test!(C0::Cancel);
		}

		#[test]
		fn em() {
			test!(C0::EndMedium);
		}

		#[test]
		fn sub() {
			test!(C0::Substitute);
		}

		#[test]
		fn fs() {
			test!(C0::FileSeparator);
		}

		#[test]
		fn gs() {
			test!(C0::GroupSeparator);
		}

		#[test]
		fn rs() {
			test!(C0::RecordSeparator);
		}

		#[test]
		fn us() {
			test!(C0::UnitSeparator);
		}
	}
}
