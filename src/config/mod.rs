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

use toml::{self, Value};
use xdg;
use picto::color::Rgba;

use error;

mod environment;
pub use self::environment::Environment;

mod style;
pub use self::style::Style;

mod color;
pub use self::color::Color;

#[derive(PartialEq, Clone, Default, Debug)]
pub struct Config {
	environment: Environment,
	style:       Style,
	color:       Color,
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

impl From<toml::Table> for Config {
	fn from(table: toml::Table) -> Config {
		let mut config = Config::default();

		if let Some(table) = table.get("environment").and_then(|v| v.as_table()) {
			config.environment.load(table);
		}

		if let Some(table) = table.get("style").and_then(|v| v.as_table()) {
			config.style.load(table);
		}

		if let Some(table) = table.get("color").and_then(|v| v.as_table()) {
			config.color.load(table);
		}

		config
	}
}

fn is_color(arg: &str) -> bool {
	if arg.starts_with('#') {
		if arg.len() == 4 || arg.len() == 5 || arg.len() == 7 || arg.len() == 9 {
			if arg.chars().skip(1).all(|c| c.is_digit(16)) {
				return true;
			}
		}
	}

	false
}

fn to_color(arg: &str) -> Option<Rgba<f64>> {
	if !is_color(arg) {
		return None;
	}

	let (r, g, b, a) = if arg.len() == 4 {
		(u8::from_str_radix(&arg[1..2], 16).unwrap() * 0x11,
		 u8::from_str_radix(&arg[2..3], 16).unwrap() * 0x11,
		 u8::from_str_radix(&arg[3..4], 16).unwrap() * 0x11,
		 255)
	}
	else if arg.len() == 5 {
		(u8::from_str_radix(&arg[1..2], 16).unwrap() * 0x11,
		 u8::from_str_radix(&arg[2..3], 16).unwrap() * 0x11,
		 u8::from_str_radix(&arg[3..4], 16).unwrap() * 0x11,
		 u8::from_str_radix(&arg[4..5], 16).unwrap() * 0x11)
	}
	else if arg.len() == 7 {
		(u8::from_str_radix(&arg[1..3], 16).unwrap(),
		 u8::from_str_radix(&arg[3..5], 16).unwrap(),
		 u8::from_str_radix(&arg[5..7], 16).unwrap(),
		 255)
	}
	else if arg.len() == 9 {
		(u8::from_str_radix(&arg[1..3], 16).unwrap(),
		 u8::from_str_radix(&arg[3..5], 16).unwrap(),
		 u8::from_str_radix(&arg[5..7], 16).unwrap(),
		 u8::from_str_radix(&arg[7..9], 16).unwrap())
	}
	else {
		unreachable!()
	};

	Some(Rgba::new_u8(r, g, b, a))
}
