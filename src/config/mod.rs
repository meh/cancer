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

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use toml;
use app_dirs::{AppInfo, AppDataType, app_root};

use error;

pub mod util;

pub mod environment;
pub use self::environment::Environment;

pub mod input;
pub use self::input::Input;

pub mod style;
pub use self::style::Style;

pub mod overlay;
pub use self::overlay::Overlay;

pub mod color;
pub use self::color::Color;

#[derive(Clone, Default, Debug)]
pub struct Config {
	environment: Environment,
	input:       Input,
	style:       Style,
	overlay:     Overlay,
	color:       Color,
}

impl Config {
	pub fn load<P: AsRef<Path>>(path: Option<P>) -> error::Result<Self> {
		let path = if let Some(path) = path {
			path.as_ref().into()
		}
		else {
			let path = app_root(AppDataType::UserConfig,
				&AppInfo { name: "cancer", author: "meh." })?.join("config.toml");

			if fs::metadata(&path).is_err() {
				if let Ok(mut file) = File::create(&path) {
					let _ = file.write_all(include_bytes!("../../assets/default.toml"));
				}
			}

			path
		};

		if let Ok(mut file) = File::open(path) {
			let mut content = String::new();
			let     _       = file.read_to_string(&mut content);

			match content.parse::<toml::Value>() {
				Ok(table) =>
					return Ok(Config::from(table.as_table().unwrap())),

				Err(error) => {
					error!(target: "cancer::config", "could not load configuration file");
					error!(target: "cancer::config", "{:?}", error);
				}
			}

		}
		else {
			error!(target: "cancer::config", "could not read configuration file");
		}

		Ok(Config::from(&toml::value::Table::new()))
	}

	pub fn environment(&self) -> &Environment {
		&self.environment
	}

	pub fn overlay(&self) -> &Overlay {
		&self.overlay
	}

	pub fn input(&self) -> &Input {
		&self.input
	}

	pub fn style(&self) -> &Style {
		&self.style
	}

	pub fn color(&self) -> &Color {
		&self.color
	}
}

impl<'a> From<&'a toml::value::Table> for Config {
	fn from(table: &'a toml::value::Table) -> Config {
		let mut config = Config::default();

		if let Some(table) = table.get("environment").and_then(|v| v.as_table()) {
			config.environment.load(table);
		}

		if let Some(table) = table.get("overlay").and_then(|v| v.as_table()) {
			config.overlay.load(table);
		}

		if let Some(table) = table.get("style").and_then(|v| v.as_table()) {
			config.style.load(table);
		}

		if let Some(table) = table.get("color").and_then(|v| v.as_table()) {
			config.color.load(table);
		}

		if let Some(table) = table.get("input").and_then(|v| v.as_table()) {
			config.input.load(table);
		}

		config
	}
}
