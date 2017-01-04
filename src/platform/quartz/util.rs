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

use std::ops::Deref;
use cocoa::base::{id, nil};

#[derive(Debug)]
pub struct IdRef(id);

impl IdRef {
	pub fn new(i: id) -> IdRef {
		IdRef(i)
	}

	#[allow(dead_code)]
	pub fn retain(i: id) -> IdRef {
		unsafe {
			if i != nil {
				msg_send![i, retain];
			}

			IdRef(i)
		}
	}

	pub fn non_nil(self) -> Option<IdRef> {
		if self.0 == nil {
			None
		}
		else {
			Some(self)
		}
	}
}

impl Deref for IdRef {
	type Target = id;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Clone for IdRef {
	fn clone(&self) -> IdRef {
		unsafe {
			if self.0 != nil {
				msg_send![self.0, retain];
			}

			IdRef(self.0)
		}
	}
}

impl Drop for IdRef {
	fn drop(&mut self) {
		unsafe {
			if self.0 != nil {
				msg_send![self.0, release];
			}
		}
	}
}
