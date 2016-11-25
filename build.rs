use std::env;

extern crate pkg_config;
use pkg_config::{Config, Error};

fn cairo() -> Result<(), Error> {
	if let Ok(path) = env::var("GTK_LIB_DIR") {
		for lib in &["cairo"] {
			println!("cargo:rustc-link-lib=dylib={}", lib);
		}

		println!("cargo:rustc-link-search=native={}", path);

		return Ok(());
	}

	Config::new().probe("cairo")?;

	Ok(())
}

fn pango() -> Result<(), Error> {
	if let Ok(path) = env::var("GTK_LIB_DIR") {
		for lib in &["pangocairo-1.0", "pango-1.0", "gobject-2.0", "glib-2.0"] {
			println!("cargo:rustc-link-lib=dylib={}", lib);
		}

		println!("cargo:rustc-link-search=native={}", path);

		return Ok(());
	}

	Config::new().probe("pangocairo")?;

	Ok(())
}

fn main() {
	cairo().unwrap();
	pango().unwrap();
}
