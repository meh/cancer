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
	pub use control::*;

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
