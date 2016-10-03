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

use toml;

#[derive(PartialEq, Clone, Debug)]
pub struct Environment {
	display: Option<String>,
	program: Option<String>,
}

impl Default for Environment {
	fn default() -> Self {
		Environment {
			display: None,
			program: None,
		}
	}
}

impl Environment {
	pub fn load(&mut self, table: &toml::Table) {
		if let Some(value) = table.get("display").and_then(|v| v.as_str()) {
			self.display = Some(value.into());
		}

		if let Some(value) = table.get("program").and_then(|v| v.as_str()) {
			self.program = Some(value.into());
		}
	}

	pub fn display(&self) -> Option<&str> {
		self.display.as_ref().map(AsRef::as_ref)
	}

	pub fn program(&self) -> Option<&str> {
		self.program.as_ref().map(AsRef::as_ref)
	}
}
