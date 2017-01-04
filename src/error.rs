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
use std::sync::mpsc::{RecvError, SendError};

use app_dirs;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	Message(String),
	Nul(ffi::NulError),
	Directory(app_dirs::AppDirsError),
	Platform(Platform),
	Unknown,
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

impl From<app_dirs::AppDirsError> for Error {
	fn from(value: app_dirs::AppDirsError) -> Self {
		Error::Directory(value)
	}
}

impl From<()> for Error {
	fn from(_value: ()) -> Self {
		Error::Unknown
	}
}

impl<T> From<SendError<T>> for Error {
	fn from(_value: SendError<T>) -> Self {
		Error::Message("send failed on closed channel".into())
	}
}

impl From<RecvError> for Error {
	fn from(_value: RecvError) -> Self {
		Error::Message("recv failed on closed channel".into())
	}
}

#[derive(Debug)]
pub enum Platform {
	#[cfg(all(feature = "x11", unix))]
	X11(platform::x11::Error),

	#[cfg(all(feature = "quartz", target_os = "macos"))]
	Quartz(platform::quartz::Error),
}

#[cfg(all(feature = "x11", unix))]
pub mod platform {
	pub mod x11 {
		use super::super::{Error as Err, Platform};
		use xcb;

		#[derive(Debug)]
		pub enum Error {
			MissingExtension,
			MissingDepth(u8),
			Request(u8, u8),
			Connection(xcb::ConnError),
		}

		impl From<Error> for Err {
			fn from(value: Error) -> Err {
				Err::Platform(Platform::X11(value))
			}
		}

		impl From<xcb::ConnError> for Err {
			fn from(value: xcb::ConnError) -> Err {
				Err::Platform(Platform::X11(Error::Connection(value)))
			}
		}

		impl<T> From<xcb::Error<T>> for Err {
			fn from(value: xcb::Error<T>) -> Err {
				Err::Platform(Platform::X11(Error::Request(value.response_type(), value.error_code())))
			}
		}
	}
}

#[cfg(all(feature = "quartz", target_os = "macos"))]
pub mod platform {
	pub mod quartz {
		pub type Error = ();
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

			Error::Directory(ref err) =>
				err.description(),

			Error::Unknown =>
				"Unknown error.",

			#[cfg(all(feature = "x11", unix))]
			Error::Platform(Platform::X11(ref err)) => match *err {
				platform::x11::Error::Request(..) =>
					"An X request failed.",

				platform::x11::Error::MissingExtension =>
					"A required X extension is missing.",

				platform::x11::Error::MissingDepth(..) =>
					"Missing visual depth.",

				platform::x11::Error::Connection(..) =>
					"Connection to the X display failed.",
			},

			#[cfg(all(feature = "quartz", target_os = "macos"))]
			Error::Platform(Platform::Quartz(_)) =>
				"Something happened :(",
		}
	}
}
