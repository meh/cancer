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

use nom::{self, ErrorKind};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum SGR {
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

use self::SGR::*;

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

pub fn parse<'a, 'b>(args: &'b [Option<u32>]) -> Result<Vec<SGR>, nom::Err<&'a [u8]>> {
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

pub mod shim {
	pub use super::SGR as T;
	pub use super::SGR::*;
	pub use super::parse;
	pub use super::{Weight, Color};
}

#[cfg(test)]
mod test {
	pub use control::*;

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
