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

use std::process::Command;
use std::sync::Arc;

use sys::cairo;
use error;
use platform;
use config::Config;

#[derive(Debug)]
pub struct Proxy {
	pub(super) config: Arc<Config>,
}

impl platform::Proxy for Proxy {
	fn dimensions(&self) -> (u32, u32) {
		(0, 0)
	}

	fn surface(&self) -> error::Result<cairo::Surface> {
		Err(error::Error::Message("wut".into()))
	}

	fn resize(&mut self, width: u32, height: u32) {

	}

	fn set_title(&self, title: String) {

	}

	fn copy(&self, name: String, value: String) {

	}

	fn paste(&self, name: String) {

	}

	fn urgent(&self) {

	}

	fn flush(&self) {

	}

	fn open(&self, value: &str) -> error::Result<()> {
		Command::new(self.config.environment().hinter().opener().unwrap_or("open"))
			.arg(value).spawn()?;

		Ok(())
	}
}
