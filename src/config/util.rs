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

use picto::color::Rgba;

pub fn is_color(arg: &str) -> bool {
	arg.starts_with('#') &&
	(arg.len() == 4 || arg.len() == 5 || arg.len() == 7 || arg.len() == 9) &&
	arg.chars().skip(1).all(|c| c.is_digit(16))
}

pub fn to_color(arg: &str) -> Option<Rgba<f64>> {
	if !is_color(arg) {
		return None;
	}

	let (r, g, b, a) = if arg.len() == 4 {
		(u8::from_str_radix(&arg[1..2], 16).unwrap() * 0x11,
		 u8::from_str_radix(&arg[2..3], 16).unwrap() * 0x11,
		 u8::from_str_radix(&arg[3..4], 16).unwrap() * 0x11,
		 255)
	}
	else if arg.len() == 5 {
		(u8::from_str_radix(&arg[1..2], 16).unwrap() * 0x11,
		 u8::from_str_radix(&arg[2..3], 16).unwrap() * 0x11,
		 u8::from_str_radix(&arg[3..4], 16).unwrap() * 0x11,
		 u8::from_str_radix(&arg[4..5], 16).unwrap() * 0x11)
	}
	else if arg.len() == 7 {
		(u8::from_str_radix(&arg[1..3], 16).unwrap(),
		 u8::from_str_radix(&arg[3..5], 16).unwrap(),
		 u8::from_str_radix(&arg[5..7], 16).unwrap(),
		 255)
	}
	else if arg.len() == 9 {
		(u8::from_str_radix(&arg[1..3], 16).unwrap(),
		 u8::from_str_radix(&arg[3..5], 16).unwrap(),
		 u8::from_str_radix(&arg[5..7], 16).unwrap(),
		 u8::from_str_radix(&arg[7..9], 16).unwrap())
	}
	else {
		unreachable!()
	};

	Some(Rgba::new_u8(r, g, b, a))
}
