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

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

use toml;
use regex::Regex;
use config::util::{to_color, to_attributes};
use config::style::{Cursor, Shape};
use style::{self, Style};

#[derive(PartialEq, Clone, Debug)]
pub struct Overlay {
	pub(super) cursor:    Cursor,
	pub(super) status:    Option<Style>,
	pub(super) selection: Style,

	pub(super) hinter:  Hinter,
	pub(super) hinters: HashMap<u32, Hinter, BuildHasherDefault<FnvHasher>>,
}

impl Default for Overlay {
	fn default() -> Self {
		Overlay {
			cursor: Cursor::default(),

			status: Some(Style {
				foreground: to_color("#000"),
				background: to_color("#c0c0c0"),
				attributes: style::NONE,
			}),

			selection: Style {
				foreground: to_color("#000"),
				background: to_color("#c0c0c0"),
				attributes: style::NONE,
			},

			hinter:  Default::default(),
			hinters: Default::default(),
		}
	}
}

#[derive(PartialEq, Clone, Debug)]
pub struct Hinter {
	label:   Vec<char>,
	matcher: Regex,
	opener:  Option<String>,

	style: Style,
}

impl Default for Hinter {
	fn default() -> Self {
		Hinter {
			label:   vec!['g', 'h', 'f', 'j', 'd', 'k', 's', 'l', 'a', 'v', 'n', 'c', 'm', 'x', 'z'],
			matcher: Regex::new(r"(https?|ftp)://(-\.)?([^\s/?\.#]+\.?)+(/[^\s]*)?").unwrap(),
			opener:  None,

			style:  Style {
				foreground: to_color("#000"),
				background: to_color("#c0c0c0"),
				attributes: style::BOLD,
			},
		}
	}
}

impl Overlay {
	pub fn load(&mut self, table: &toml::Table) {
		if let Some(table) = table.get("cursor").and_then(|v| v.as_table()) {
			if let Some(value) = table.get("shape").and_then(|v| v.as_str()) {
				match &*value.to_lowercase() {
					"block" =>
						self.cursor.shape = Shape::Block,

					"beam" | "ibeam" =>
						self.cursor.shape = Shape::Beam,

					"underline" | "line" =>
						self.cursor.shape = Shape::Line,

					_ => ()
				}
			}

			if let Some(true) = table.get("blink").and_then(|v| v.as_bool()) {
				self.cursor.blink = true;
			}

			if let Some(value) = table.get("foreground").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
				self.cursor.foreground = value;
			}

			if let Some(value) = table.get("background").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
				self.cursor.background = value;
			}
		}

		if let Some(value) = table.get("status") {
			if let Some(table) = value.as_table() {
				let mut status = Style::default();

				if let Some(value) = table.get("foreground").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
					status.foreground = Some(value);
				}

				if let Some(value) = table.get("background").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
					status.background = Some(value);
				}

				if let Some(value) = table.get("attributes").and_then(|v| v.as_str()) {
					status.attributes = to_attributes(value);
				}

				self.status = Some(status);
			}
			else {
				self.status = None;
			}
		}

		if let Some(value) = table.get("selection") {
			if let Some(table) = value.as_table() {
				if let Some(value) = table.get("foreground").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
					self.selection.foreground = Some(value);
				}

				if let Some(value) = table.get("background").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
					self.selection.background = Some(value);
				}

				if let Some(value) = table.get("attributes").and_then(|v| v.as_str()) {
					self.selection.attributes = to_attributes(value);
				}
			}
		}

		if let Some(table) = table.get("hinter").and_then(|v| v.as_table()) {
			if let Some(value) = table.get("label").and_then(|v| v.as_str()) {
				self.hinter.label = value.chars().collect();
			}

			match table.get("matcher").and_then(|v| v.as_str()).map(|v| Regex::new(v)) {
				None => (),

				Some(Ok(value)) => {
					self.hinter.matcher = value;
				}

				Some(Err(err)) => {
					error!(target: "cancer::config", "[overlay.hinter.matcher]");
					error!(target: "cancer::config", "{}", err);
				}
			}

			if let Some(value) = table.get("opener").and_then(|v| v.as_str()) {
				self.hinter.opener = Some(value.into());
			}

			if let Some(table) = table.get("style").and_then(|v| v.as_table()) {
				if let Some(value) = table.get("foreground").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
					self.hinter.style.foreground = Some(value);
				}

				if let Some(value) = table.get("background").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
					self.hinter.style.background = Some(value);
				}

				if let Some(value) = table.get("attributes").and_then(|v| v.as_str()) {
					self.hinter.style.attributes = to_attributes(value);
				}
			}

			for (name, table) in table {
				let id    = try!(continue name.parse());
				let table = try!(continue option table.as_table());

				if id == 0 {
					continue;
				}

				let mut hinter = self.hinter.clone();

				if let Some(value) = table.get("label").and_then(|v| v.as_str()) {
					hinter.label = value.chars().collect();
				}

				match table.get("matcher").and_then(|v| v.as_str()).map(|v| Regex::new(v)) {
					None => (),

					Some(Ok(value)) => {
						hinter.matcher = value;
					}

					Some(Err(err)) => {
						error!(target: "cancer::config", "[overlay.hinter.{}.matcher]", id);
						error!(target: "cancer::config", "{}", err);
					}
				}

				if let Some(value) = table.get("opener").and_then(|v| v.as_str()) {
					hinter.opener = Some(value.into());
				}

				if let Some(table) = table.get("style").and_then(|v| v.as_table()) {
					if let Some(value) = table.get("foreground").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
						hinter.style.foreground = Some(value);
					}

					if let Some(value) = table.get("background").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
						hinter.style.background = Some(value);
					}

					if let Some(value) = table.get("attributes").and_then(|v| v.as_str()) {
						hinter.style.attributes = to_attributes(value);
					}
				}

				self.hinters.insert(id, hinter);
			}
		}
	}

	pub fn cursor(&self) -> &Cursor {
		&self.cursor
	}

	pub fn status(&self) -> Option<&Style> {
		self.status.as_ref()
	}

	pub fn selection(&self) -> &Style {
		&self.selection
	}

	pub fn hinter(&self, id: u32) -> &Hinter {
		self.hinters.get(&id).unwrap_or(&self.hinter)
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

	pub fn style(&self) -> &Style {
		&self.style
	}
}
