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

impl Into<Vec<u32>> for SGR {
	fn into(self) -> Vec<u32> {
		match self {
			Reset =>
				vec![0],

			Font(Weight::Bold) =>
				vec![1],

			Font(Weight::Faint) =>
				vec![2],

			Italic(true) =>
				vec![3],

			Underline(true) =>
				vec![4],

			Blink(true) =>
				vec![5],

			Reverse(true) =>
				vec![7],

			Invisible(true) =>
				vec![8],

			Struck(true) =>
				vec![9],

			Font(Weight::Normal) =>
				vec![22],

			Italic(false) =>
				vec![23],

			Underline(false) =>
				vec![24],

			Blink(false) =>
				vec![25],

			Reverse(false) =>
				vec![27],

			Invisible(false) =>
				vec![28],

			Struck(false) =>
				vec![29],

			Foreground(Color::Index(index)) if index < 8 =>
				vec![index as u32 + 30],

			Foreground(Color::Index(index)) if index < 16 =>
				vec![index as u32 - 8 + 90],

			Foreground(Color::Index(index)) =>
				vec![38, 5, index as u32],

			Foreground(Color::Default) =>
				vec![38, 0],

			Foreground(Color::Transparent) =>
				vec![38, 1],

			Foreground(Color::Rgb(r, g, b)) =>
				vec![38, 2, r as u32, g as u32, b as u32],

			Foreground(Color::Cmy(c, m, y)) =>
				vec![38, 3, c as u32, m as u32, y as u32],

			Foreground(Color::Cmyk(c, m, y, k)) =>
				vec![38, 4, c as u32, m as u32, y as u32, k as u32],

			Background(Color::Index(index)) if index < 8 =>
				vec![index as u32 + 40],

			Background(Color::Index(index)) if index < 16 =>
				vec![index as u32 - 8 + 100],

			Background(Color::Index(index)) =>
				vec![48, 5, index as u32],

			Background(Color::Default) =>
				vec![48, 0],

			Background(Color::Transparent) =>
				vec![48, 1],

			Background(Color::Rgb(r, g, b)) =>
				vec![48, 2, r as u32, g as u32, b as u32],

			Background(Color::Cmy(c, m, y)) =>
				vec![48, 3, c as u32, m as u32, y as u32],

			Background(Color::Cmyk(c, m, y, k)) =>
				vec![48, 4, c as u32, m as u32, y as u32, k as u32],
		}
	}
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
	mod parse {
		pub use control::*;

		macro_rules! test {
			($string:expr => $($attrs:expr),+) => (
				assert_eq!(Item::C1(C1::ControlSequence(CSI::SelectGraphicalRendition(vec![$($attrs),*]))),
					parse($string).unwrap().1);
			);
		}

		#[test]
		fn reset() {
			test!(b"\x1B[0m" =>
				SGR::Reset);

			test!(b"\x1B[m" =>
				SGR::Reset);
		}

		#[test]
		fn font() {
			test!(b"\x1B[1m" =>
				SGR::Font(SGR::Weight::Bold));

			test!(b"\x1B[2m" =>
				SGR::Font(SGR::Weight::Faint));

			test!(b"\x1B[22m" =>
				SGR::Font(SGR::Weight::Normal));
		}

		#[test]
		fn italic() {
			test!(b"\x1B[3m" =>
				SGR::Italic(true));

			test!(b"\x1B[23m" =>
				SGR::Italic(false));
		}

		#[test]
		fn underline() {
			test!(b"\x1B[4m" =>
				SGR::Underline(true));

			test!(b"\x1B[24m" =>
				SGR::Underline(false));
		}

		#[test]
		fn blink() {
			test!(b"\x1B[5m" =>
				SGR::Blink(true));

			test!(b"\x1B[6m" =>
				SGR::Blink(true));

			test!(b"\x1B[25m" =>
				SGR::Blink(false));
		}

		#[test]
		fn reverse() {
			test!(b"\x1B[7m" =>
				SGR::Reverse(true));

			test!(b"\x1B[27m" =>
				SGR::Reverse(false));
		}

		#[test]
		fn invisible() {
			test!(b"\x1B[8m" =>
				SGR::Invisible(true));

			test!(b"\x1B[28m" =>
				SGR::Invisible(false));
		}

		#[test]
		fn struck() {
			test!(b"\x1B[9m" =>
				SGR::Struck(true));

			test!(b"\x1B[29m" =>
				SGR::Struck(false));
		}

		#[test]
		fn foreground() {
			test!(b"\x1B[38;0m" =>
				SGR::Foreground(SGR::Color::Default));

			test!(b"\x1B[39m" =>
				SGR::Foreground(SGR::Color::Default));

			test!(b"\x1B[38;1m" =>
				SGR::Foreground(SGR::Color::Transparent));

			test!(b"\x1B[30m" =>
				SGR::Foreground(SGR::Color::Index(0)));

			test!(b"\x1B[37m" =>
				SGR::Foreground(SGR::Color::Index(7)));

			test!(b"\x1B[38;2;255;;127m" =>
				SGR::Foreground(SGR::Color::Rgb(255, 0, 127)));

			test!(b"\x1B[38;5;235m" =>
				SGR::Foreground(SGR::Color::Index(235)));

			test!(b"\x1B[90m" =>
				SGR::Foreground(SGR::Color::Index(8)));

			test!(b"\x1B[97m" =>
				SGR::Foreground(SGR::Color::Index(15)));
		}

		#[test]
		fn background() {
			test!(b"\x1B[48;0m" =>
				SGR::Background(SGR::Color::Default));

			test!(b"\x1B[49m" =>
				SGR::Background(SGR::Color::Default));

			test!(b"\x1B[48;1m" =>
				SGR::Background(SGR::Color::Transparent));

			test!(b"\x1B[40m" =>
				SGR::Background(SGR::Color::Index(0)));

			test!(b"\x1B[47m" =>
				SGR::Background(SGR::Color::Index(7)));

			test!(b"\x1B[48;2;255;;127m" =>
				SGR::Background(SGR::Color::Rgb(255, 0, 127)));

			test!(b"\x1B[48;5;235m" =>
				SGR::Background(SGR::Color::Index(235)));

			test!(b"\x1B[100m" =>
				SGR::Background(SGR::Color::Index(8)));

			test!(b"\x1B[107m" =>
				SGR::Background(SGR::Color::Index(15)));
		}

		#[test]
		fn sequence() {
			test!(b"\x1B[38;2;0;255;127;48;2;127;255;0m" =>
				SGR::Foreground(SGR::Color::Rgb(0, 255, 127)),
				SGR::Background(SGR::Color::Rgb(127, 255, 0)));
		}
	}

	mod format {
		pub use control::*;

		macro_rules! test {
			($($attr:expr),+) => (
				let item = Item::C1(C1::ControlSequence(CSI::SelectGraphicalRendition(
					vec![$($attr),*])));

				let mut result = vec![];
				item.fmt(&mut result, true).unwrap();
				assert_eq!(item, parse(&result).unwrap().1);

				let mut result = vec![];
				item.fmt(&mut result, false).unwrap();
				assert_eq!(item, parse(&result).unwrap().1);
			);
		}

		#[test]
		fn reset() {
			test!(SGR::Reset);
		}

		#[test]
		fn font() {
			test!(SGR::Font(SGR::Weight::Bold));
			test!(SGR::Font(SGR::Weight::Faint));
			test!(SGR::Font(SGR::Weight::Normal));
		}

		#[test]
		fn italic() {
			test!(SGR::Italic(true));
			test!(SGR::Italic(false));
		}

		#[test]
		fn underline() {
			test!(SGR::Underline(true));
			test!(SGR::Underline(false));
		}

		#[test]
		fn blink() {
			test!(SGR::Blink(true));
			test!(SGR::Blink(false));
		}

		#[test]
		fn reverse() {
			test!(SGR::Reverse(true));
			test!(SGR::Reverse(false));
		}

		#[test]
		fn invisible() {
			test!(SGR::Invisible(true));
			test!(SGR::Invisible(false));
		}

		#[test]
		fn struck() {
			test!(SGR::Struck(true));
			test!(SGR::Struck(false));
		}

		#[test]
		fn foreground() {
			test!(SGR::Foreground(SGR::Color::Default));
			test!(SGR::Foreground(SGR::Color::Transparent));
			test!(SGR::Foreground(SGR::Color::Index(0)));
			test!(SGR::Foreground(SGR::Color::Index(7)));
			test!(SGR::Foreground(SGR::Color::Rgb(255, 0, 127)));
			test!(SGR::Foreground(SGR::Color::Index(235)));
			test!(SGR::Foreground(SGR::Color::Index(8)));
			test!(SGR::Foreground(SGR::Color::Index(15)));
		}

		#[test]
		fn background() {
			test!(SGR::Background(SGR::Color::Default));
			test!(SGR::Background(SGR::Color::Transparent));
			test!(SGR::Background(SGR::Color::Index(0)));
			test!(SGR::Background(SGR::Color::Index(7)));
			test!(SGR::Background(SGR::Color::Rgb(255, 0, 127)));
			test!(SGR::Background(SGR::Color::Index(235)));
			test!(SGR::Background(SGR::Color::Index(8)));
			test!(SGR::Background(SGR::Color::Index(15)));
		}

		#[test]
		fn sequence() {
			test!(SGR::Foreground(SGR::Color::Rgb(0, 255, 127)),
			      SGR::Background(SGR::Color::Rgb(127, 255, 0)));
		}
	}
}
