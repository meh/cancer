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

use toml::{self, Value};
use regex::Regex;

#[derive(PartialEq, Clone, Debug)]
pub struct Environment {
	display: Option<String>,
	program: Option<String>,
	term:    Option<String>,
	bell:    i8,
	hinter:  Hinter,

	cache:  usize,
	scroll: usize,
	batch:  Option<u32>,
}

impl Default for Environment {
	fn default() -> Self {
		Environment {
			display: None,
			program: None,
			term:    None,
			bell:    0,
			hinter:  Default::default(),

			cache:  4096,
			scroll: 4096,
			batch:  Some(16),
		}
	}
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Hinter {
	label:   Vec<char>,
	matcher: Regex,
	opener:  Option<String>,
}

impl Default for Hinter {
	fn default() -> Self {
		Hinter {
			label:   vec!['g', 'h', 'f', 'j', 'd', 'k', 's', 'l', 'a', 'v', 'n', 'c', 'm', 'x', 'z'],
			matcher: Regex::new(r"\b(https?|ftp)://(-\.)?([^\s/?#]+\.?)+(/[^\s]*)?\b").unwrap(),
			opener:  None,
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

		if let Some(value) = table.get("term").and_then(|v| v.as_str()) {
			self.term = Some(value.into());
		}

		if let Some(value) = table.get("bell").and_then(|v| v.as_integer()) {
			self.bell = value as i8;
		}

		if let Some(table) = table.get("hinter").and_then(|v| v.as_table()) {
			if let Some(value) = table.get("label").and_then(|v| v.as_str()) {
				self.hinter.label = value.chars().collect();
			}

			if let Some(value) = table.get("matcher").and_then(|v| v.as_str()) {
				if let Ok(value) = Regex::new(value) {
					self.hinter.matcher = value;
				}
			}

			if let Some(value) = table.get("opener").and_then(|v| v.as_str()) {
				self.hinter.opener = Some(value.into());
			}
		}

		if let Some(value) = table.get("cache") {
			match *value {
				Value::Integer(value) =>
					self.cache = value as usize,

				Value::Boolean(false) =>
					self.cache = 0,

				_ => ()
			}
		}

		if let Some(value) = table.get("scroll") {
			match *value {
				Value::Integer(value) =>
					self.scroll = value as usize,

				Value::Boolean(false) =>
					self.scroll = 0,

				_ => ()
			}
		}

		if let Some(value) = table.get("batch") {
			match *value {
				Value::Boolean(false) =>
					self.batch = None,

				Value::Integer(value) =>
					self.batch = Some(value as u32),

				_ => ()
			}
		}
	}

	pub fn display(&self) -> Option<&str> {
		self.display.as_ref().map(AsRef::as_ref)
	}

	pub fn program(&self) -> Option<&str> {
		self.program.as_ref().map(AsRef::as_ref)
	}

	pub fn term(&self) -> Option<&str> {
		self.term.as_ref().map(AsRef::as_ref)
	}

	pub fn bell(&self) -> i8 {
		self.bell
	}

	pub fn hinter(&self) -> &Hinter {
		&self.hinter
	}

	pub fn cache(&self) -> usize {
		self.cache
	}

	pub fn scroll(&self) -> usize {
		self.scroll
	}

	pub fn batch(&self) -> Option<u32> {
		self.batch
	}
}

impl Hinter {
	pub fn label(&self) -> &[char] {
		&self.label
	}

	pub fn matcher(&self) -> &Regex {
		&self.matcher
	}

	pub fn opener(&self) -> Option<&str> {
		self.opener.as_ref().map(AsRef::as_ref)
	}
}
