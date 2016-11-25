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

use libc::c_void;

#[repr(C)]
pub struct GList {
	pub data: *mut c_void,
	pub next: *mut GList,
	pub prev: *mut GList,
}

extern "C" {
	pub fn g_list_free(ptr: *mut GList);

	pub fn g_object_unref(ptr: *mut c_void);
	pub fn g_object_ref(ptr: *mut c_void) -> *mut c_void;
}
