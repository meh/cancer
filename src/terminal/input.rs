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

use control;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Input<'a> {
	Done(&'a [u8], Kind<'a>),
	Incomplete(Option<usize>),
	Error(usize),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Kind<'a> {
	Unicode(&'a str),
	Ascii(&'a [u8]),
}

pub fn parse(i: &[u8]) -> Input {
	use std::str;

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

	let mut length = WIDTH[i[0] as usize] as usize;
	let mut ascii  = length == 1;

	if length == 0 {
		return Input::Error(0);
	}
	else if i.len() < length {
		return Input::Incomplete(None);
	}
	else if !ascii && str::from_utf8(&i[..length]).is_err() {
		return Input::Error(length);
	}

	let mut rest = &i[length..];

	while !rest.is_empty() && control::parse(rest).is_err() {
		let w = WIDTH[rest[0] as usize] as usize;

		if w > 1 {
			ascii = false;
		}

		if w == 0 {
			break;
		}
		else if rest.len() < w {
			return Input::Incomplete(Some(w - rest.len()));
		}
		else if !ascii && str::from_utf8(&rest[..w]).is_err() {
			break;
		}

		length += w;
		rest    = &rest[w..];
	}

	if ascii {
		Input::Done(&i[length..], Kind::Ascii(&i[..length]))
	}
	else {
		Input::Done(&i[length..], Kind::Unicode(unsafe { str::from_utf8_unchecked(&i[..length]) }))
	}
}
