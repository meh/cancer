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
use picto::color::Rgba;
use config::util::to_color;

#[derive(PartialEq, Clone, Debug)]
pub struct Style {
	pub(super) font:      String,
	pub(super) ligatures: bool,
	pub(super) blink:     u32,
	pub(super) bold:      Bold,

	pub(super) margin:  u8,
	pub(super) spacing: u8,

	pub(super) color:  Color,
	pub(super) cursor: Cursor,
}

impl Default for Style {
	fn default() -> Self {
		Style {
			font:      "monospace 16px".into(),
			ligatures: false,
			blink:     500,
			bold:      Bold::default(),

			margin:  0,
			spacing: 0,

			color:  Default::default(),
			cursor: Default::default(),
		}
	}
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Bold {
	Normal,
	Bright,
}

impl Default for Bold {
	fn default() -> Self {
		Bold::Bright
	}
}

impl Bold {
	pub fn is_bright(&self) -> bool {
		*self == Bold::Bright
	}
}

#[derive(PartialEq, Clone, Debug)]
pub struct Color {
	pub(super) foreground: Rgba<f64>,
	pub(super) background: Rgba<f64>,

	pub(super) underline:     Option<Rgba<f64>>,
	pub(super) strikethrough: Option<Rgba<f64>>,
}

impl Default for Color {
	fn default() -> Self {
		Color {
			foreground:    to_color("#c0c0c0").unwrap(),
			background:    to_color("#000").unwrap(),
			underline:     None,
			strikethrough: None,
		}
	}
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Shape {
	Block,
	Line,
	Beam,
}

impl Default for Shape {
	fn default() -> Shape {
		Shape::Block
	}
}

#[derive(PartialEq, Clone, Debug)]
pub struct Cursor {
	pub(super) shape: Shape,
	pub(super) blink: bool,

	pub(super) foreground: Rgba<f64>,
	pub(super) background: Rgba<f64>,
}

impl Default for Cursor {
	fn default() -> Self {
		Cursor {
			shape: Shape::default(),
			blink: false,

			foreground: to_color("#000").unwrap(),
			background: to_color("#fff").unwrap(),
		}
	}
}

impl Style {
	pub fn load(&mut self, table: &toml::value::Table) {
		if let Some(value) = table.get("font").and_then(|v| v.as_str()) {
			self.font = value.into();
		}

		if let Some(value) = table.get("ligatures").and_then(|v| v.as_bool()) {
			self.ligatures = value;
		}

		if let Some(value) = table.get("bold").and_then(|v| v.as_str()) {
			match &*value.to_lowercase() {
				"bright" =>
					self.bold = Bold::Bright,

				"normal" =>
					self.bold = Bold::Normal,

				_ => ()
			}
		}

		if let Some(value) = table.get("blink") {
			match *value {
				Value::Boolean(false) =>
					self.blink = 0,

				Value::Integer(value) =>
					self.blink = value as u32,

				_ => ()
			}
		}

		if let Some(value) = table.get("margin").and_then(|v| v.as_integer()) {
			self.margin = value as u8;
		}

		if let Some(value) = table.get("spacing").and_then(|v| v.as_integer()) {
			self.spacing = value as u8;
		}

		if let Some(table) = table.get("color").and_then(|v| v.as_table()) {
			if let Some(value) = table.get("foreground").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
				self.color.foreground = value;
			}

			if let Some(value) = table.get("background").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
				self.color.background = value;
			}

			if let Some(value) = table.get("underline").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
				self.color.underline = Some(value);
			}

			if let Some(value) = table.get("strikethrough").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
				self.color.strikethrough = Some(value);
			}
		}

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
	}

	pub fn font(&self) -> &str {
		&self.font
	}

	pub fn ligatures(&self) -> bool {
		self.ligatures
	}

	pub fn blink(&self) -> u32 {
		self.blink
	}

	pub fn bold(&self) -> Bold {
		self.bold
	}

	pub fn margin(&self) -> u32 {
		self.margin as u32
	}

	pub fn spacing(&self) -> u32 {
		self.spacing as u32
	}

	pub fn color(&self) -> &Color {
		&self.color
	}

	pub fn cursor(&self) -> &Cursor {
		&self.cursor
	}
}

impl Color {
	pub fn foreground(&self) -> &Rgba<f64> {
		&self.foreground
	}

	pub fn background(&self) -> &Rgba<f64> {
		&self.background
	}

	pub fn underline(&self) -> Option<&Rgba<f64>> {
		self.underline.as_ref()
	}

	pub fn strikethrough(&self) -> Option<&Rgba<f64>> {
		self.strikethrough.as_ref()
	}
}

impl Cursor {
	pub fn shape(&self) -> Shape {
		self.shape
	}

	pub fn blink(&self) -> bool {
		self.blink
	}

	pub fn foreground(&self) -> &Rgba<f64> {
		&self.foreground
	}

	pub fn background(&self) -> &Rgba<f64> {
		&self.background
	}
}
