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

use std::fs::File;
use std::io::Read;
use std::path::Path;

use toml;
use xdg;

use error;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Environment {
	display: Option<String>,
	fps:     u8,
}

impl Default for Environment {
	fn default() -> Self {
		Environment {
			display: None,
			fps:     30,
		}
	}
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Style {
	font:    Option<String>,
	margin:  u8,
	spacing: u8,
}

impl Default for Style {
	fn default() -> Self {
		Style {
			font:    None,
			margin:  2,
			spacing: 1,
		}
	}
}

#[derive(Eq, PartialEq, Clone, Default, Debug)]
pub struct Config {
	environment: Environment,
	style:       Style,
}

impl Config {
	pub fn load<P: AsRef<Path>>(path: Option<P>) -> error::Result<Self> {
		let path = if let Some(path) = path {
			path.as_ref().into()
		}
		else {
			xdg::BaseDirectories::with_prefix("cancer").unwrap()
				.place_config_file("config.toml").unwrap()
		};

		let table = if let Ok(mut file) = File::open(path) {
			let mut content = String::new();
			file.read_to_string(&mut content)?;

			toml::Parser::new(&content).parse().ok_or(error::Error::Parse)?
		}
		else {
			toml::Table::new()
		};

		Ok(Config::from(table))
	}

	pub fn style(&self) -> &Style {
		&self.style
	}

	pub fn environment(&self) -> &Environment {
		&self.environment
	}
}

impl Style {
	pub fn font(&self) -> &str {
		self.font.as_ref().map(AsRef::as_ref).unwrap_or("monospace")
	}

	pub fn margin(&self) -> u32 {
		self.margin as u32
	}

	pub fn spacing(&self) -> u32 {
		self.spacing as u32
	}
}

impl Environment {
	pub fn display(&self) -> Option<&str> {
		self.display.as_ref().map(AsRef::as_ref)
	}

	pub fn fps(&self) -> u64 {
		self.fps as u64
	}
}

impl From<toml::Table> for Config {
	fn from(table: toml::Table) -> Config {
		let mut config = Config::default();

		if let Some(table) = table.get("environment") {
			let mut environment = Environment::default();

			if let Some(value) = table.lookup("display").and_then(|v| v.as_str()) {
				environment.display = Some(value.into());
			}

			if let Some(value) = table.lookup("fps").and_then(|v| v.as_integer()) {
				environment.fps = value as u8;
			}

			config.environment = environment;
		}

		if let Some(table) = table.get("style") {
			let mut style = Style::default();

			if let Some(value) = table.lookup("font").and_then(|v| v.as_str()) {
				style.font = Some(value.into());
			}

			if let Some(value) = table.lookup("margin").and_then(|v| v.as_integer()) {
				style.margin = value as u8;
			}

			if let Some(value) = table.lookup("spacing").and_then(|v| v.as_integer()) {
				style.spacing = value as u8;
			}

			config.style = style;
		}

		config
	}
}
