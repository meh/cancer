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
use config::util::{to_color, to_attributes};
use style;

#[derive(PartialEq, Clone, Debug)]
pub struct Style {
	font:  Option<String>,
	blink: u32,
	bold:  Bold,

	margin:  u8,
	spacing: u8,

	color:  Color,
	cursor: Cursor,
	status: Option<Status>,
}

impl Default for Style {
	fn default() -> Self {
		Style {
			font:  None,
			blink: 500,
			bold:  Bold::default(),

			margin:  0,
			spacing: 0,

			color:  Default::default(),
			cursor: Default::default(),
			status: Some(Default::default()),
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
		Bold::Normal
	}
}

impl Bold {
	pub fn is_bright(&self) -> bool {
		*self == Bold::Bright
	}
}

#[derive(PartialEq, Clone, Debug)]
pub struct Color {
	foreground: Rgba<f64>,
	background: Rgba<f64>,

	underline:     Option<Rgba<f64>>,
	strikethrough: Option<Rgba<f64>>,
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
	shape: Shape,
	blink: bool,

	foreground: Rgba<f64>,
	background: Rgba<f64>,
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

#[derive(PartialEq, Clone, Debug)]
pub struct Status {
	foreground: Rgba<f64>,
	background: Rgba<f64>,
	attributes: style::Attributes,
}

impl Default for Status {
	fn default() -> Self {
		Status {
			foreground: to_color("#000").unwrap(),
			background: to_color("#c0c0c0").unwrap(),
			attributes: style::NONE,
		}
	}
}

impl Style {
	pub fn load(&mut self, table: &toml::Table) {
		if let Some(value) = table.get("font").and_then(|v| v.as_str()) {
			self.font = Some(value.into());
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

		if let Some(value) = table.get("status") {
			if let Some(table) = value.as_table() {
				let mut status = Status::default();

				if let Some(value) = table.get("foreground").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
					status.foreground = value;
				}

				if let Some(value) = table.get("background").and_then(|v| v.as_str()).and_then(|v| to_color(v)) {
					status.background = value;
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
	}

	pub fn font(&self) -> &str {
		self.font.as_ref().map(AsRef::as_ref).unwrap_or("monospace 16px")
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

	pub fn status(&self) -> Option<&Status> {
		self.status.as_ref()
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

impl Status {
	pub fn foreground(&self) -> &Rgba<f64> {
		&self.foreground
	}

	pub fn background(&self) -> &Rgba<f64> {
		&self.background
	}

	pub fn attributes(&self) -> style::Attributes {
		self.attributes
	}
}
