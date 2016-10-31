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

use std::fmt;
use std::error;
use std::io;
use std::ffi;

use xcb;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	Message(String),
	Nul(ffi::NulError),
	Unknown,
	Config,
	X(X),
}

#[derive(Debug)]
pub enum X {
	MissingExtension,
	MissingDepth(u8),
	Request(u8, u8),
	Connection(xcb::ConnError),
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Error::Io(value)
	}
}

impl From<ffi::NulError> for Error {
	fn from(value: ffi::NulError) -> Self {
		Error::Nul(value)
	}
}

impl From<String> for Error {
	fn from(value: String) -> Self {
		Error::Message(value)
	}
}

impl From<()> for Error {
	fn from(_value: ()) -> Self {
		Error::Unknown
	}
}

impl From<X> for Error {
	fn from(value: X) -> Error {
		Error::X(value)
	}
}

impl From<xcb::ConnError> for Error {
	fn from(value: xcb::ConnError) -> Error {
		Error::X(X::Connection(value))
	}
}

impl<T> From<xcb::Error<T>> for Error {
	fn from(value: xcb::Error<T>) -> Error {
		Error::X(X::Request(value.response_type(), value.error_code()))
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
		f.write_str(error::Error::description(self))
	}
}

impl error::Error for Error {
	fn description(&self) -> &str {
		match *self {
			Error::Io(ref err) =>
				err.description(),

			Error::Nul(ref err) =>
				err.description(),

			Error::Message(ref msg) =>
				msg.as_ref(),

			Error::Unknown =>
				"Unknown error.",

			Error::Config =>
				"Configuration error.",

			Error::X(ref err) => match *err {
				X::Request(..) =>
					"An X request failed.",

				X::MissingExtension =>
					"A required X extension is missing.",

				X::MissingDepth(..) =>
					"Missing visual depth.",

				X::Connection(..) =>
					"Connection to the X display failed.",
			},
		}
	}
}
