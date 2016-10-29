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

use std::cmp;

macro_rules! try {
	(return $body:expr) => (
		if let Ok(value) = $body {
			value
		}
		else {
			return;
		}
	);

	(continue $body:expr) => (
		if let Ok(value) = $body {
			value
		}
		else {
			continue;
		}
	);

	(break $body:expr) => (
		if let Ok(value) = $body {
			value
		}
		else {
			break;
		}
	);

	($body:expr) => (
		$body?
	);
}

macro_rules! vec_deque {
	($value:expr; $size:expr) => ({
		let mut value = VecDeque::new();
		value.extend(::std::iter::repeat($value).take($size));
		value
	})
}

pub fn clamp<T: PartialOrd>(n: T, min: T, max: T) -> T {
	if n > max {
		max
	}
	else if n < min {
		min
	}
	else {
		n
	}
}
