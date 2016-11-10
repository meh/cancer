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

use std::sync::Arc;
use std::io::{self, Write};
use std::mem;
use std::vec;
use std::str;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use picto::Region;
use picto::color::Rgba;
use control::{self, Control, C0, C1, DEC, CSI, SGR};
use util;
use error;
use config::{self, Config};
use config::style::Shape;
use style::{self, Style};
use platform::key::{self, Key};
use platform::mouse::{self, Mouse};
use terminal::{Access, Iter, Touched, Cell, Tabs, Grid, Action, cell};
use terminal::mode::{self, Mode};
use terminal::cursor::{self, Cursor};
use terminal::touched;
use terminal::input::{self, Input};

#[derive(Debug)]
pub struct Terminal {
	config:  Arc<Config>,
	region:  Region,
	cache:   Option<Vec<u8>>,
	touched: Touched,
	mode:    Mode,
	click:   Option<mouse::Click>,

	scroll: Option<u32>,
	grid:   Grid,
	tabs:   Tabs,

	cursor: Cursor,
	saved:  Option<Cursor>,
}

macro_rules! term {
	($term:ident; charset) => (
		$term.cursor.charsets[$term.cursor.charset as usize]
	);

	($term:ident; scroll! up $n:tt) => (
		if $term.cursor.scroll == (0, $term.region.height - 1) {
			$term.touched.all();
			$term.grid.up($n, None);
		}
		else {
			term!($term; scroll up $n)
		}
	);

	($term:ident; scroll up $n:tt) => (
		term!($term; scroll up $n from $term.cursor.scroll.0)
	);

	($term:ident; scroll up $n:tt from $y:expr) => ({
		$term.grid.up($n as u32, Some(($y, $term.cursor.scroll.1)));

		for y in $y ... $term.cursor.scroll.1 {
			$term.touched.line(y);
		}
	});

	($term:ident; scroll down $n:tt) => (
		term!($term; scroll down $n from $term.cursor.scroll.0)
	);

	($term:ident; scroll down $n:tt from $y:expr) => ({
		$term.grid.down($n as u32, Some(($y, $term.cursor.scroll.1)));

		for y in $y ... $term.cursor.scroll.1 {
			$term.touched.line(y);
		}
	});

	($term:ident; cursor) => ({
		let x = $term.cursor.x();
		let y = $term.cursor.y();

		if let Cell::Reference(offset) = *$term.grid.get(x, y) {
			(x - offset as u32, y)
		}
		else {
			(x, y)
		}
	});

	($term:ident; cursor $($travel:tt)*) => ({
		$term.touched.push($term.cursor.position());
		let r = $term.cursor.travel(cursor::$($travel)*);
		$term.touched.push($term.cursor.position());
		r
	});

	($term:ident; tab $n:expr) => ({
		let (x, _) = term!($term; cursor);
		term!($term; cursor Position(Some($term.tabs.next($n, x)), None));
	});

	($term:ident; clean references ($x:expr, $y:expr)) => ({
		if $x < $term.region.width {
			$term.grid.clean_references($x, $y);
		}
	});
}

impl Terminal {
	pub fn open(config: Arc<Config>, width: u32, height: u32) -> error::Result<Self> {
		let region = Region::from(0, 0, width, height);
		let grid   = Grid::new(width, height, config.environment().scroll());
		let tabs   = Tabs::new(width, height);

		Ok(Terminal {
			config:  config.clone(),
			region:  region,
			cache:   Default::default(),
			touched: Touched::default(),
			mode:    Mode::default(),
			click:   None,

			scroll: None,
			grid:   grid,
			tabs:   tabs,

			cursor: Cursor::new(config.clone(), width, height),
			saved:  None,
		})
	}

	pub fn config(&self) -> &Config {
		&self.config
	}

	pub fn columns(&self) -> u32 {
		self.region.width
	}

	pub fn rows(&self) -> u32 {
		self.region.height
	}

	pub fn mode(&self) -> Mode {
		self.mode
	}

	pub fn grid(&self) -> &Grid {
		&self.grid
	}

	/// Get the cursor.
	pub fn cursor(&self) -> cursor::Cell {
		let (x, y) = term!(self; cursor);
		cursor::Cell::new(&self.cursor, cell::Position::new(x, y, self.grid.get(x, y)))
	}

	/// Get the region of the terminal.
	pub fn region(&self) -> Region {
		self.region
	}

	/// Get an iterator over positioned cells.
	pub fn iter<T: Iterator<Item = (u32, u32)>>(&self, iter: T) -> Iter<Self, T> {
		Iter::new(self, iter)
	}

	/// Resize the terminal.
	pub fn resize(&mut self, width: u32, height: u32) {
		self.region.width  = width;
		self.region.height = height;

		self.tabs.resize(width, height);

		match self.grid.resize(width, height) {
			n if n > 0 => {
				self.cursor.travel(cursor::Down(n as u32));
			}

			n if n < 0 => {
				self.cursor.travel(cursor::Up((-n) as u32));
			}

			_ => ()
		}

		self.cursor.resize(width, height);
		self.saved = None;
	}

	/// Enable or disable blinking and return the affected cells.
	pub fn blinking(&mut self, value: bool) -> touched::Iter {
		if value {
			self.mode.insert(mode::BLINK);
		}
		else {
			self.mode.remove(mode::BLINK);
		}

		for (x, y) in self.region.absolute() {
			match *self.grid.get(x, y) {
				Cell::Empty { ref style, .. } |
				Cell::Occupied { ref style, .. } if style.attributes().contains(style::BLINK) => {
					self.touched.mark(x, y);
				}

				_ => ()
			}
		}

		self.touched.iter(self.region)
	}

	/// Send focus events.
	pub fn focus<O: Write>(&mut self, value: bool, mut output: O) -> io::Result<()> {
		if self.mode.contains(mode::FOCUS) {
			try!(output.write_all(if value {
				b"\x1B[I"
			}
			else {
				b"\x1B[O"
			}));
		}

		Ok(())
	}

	/// Handle a key.
	pub fn key<O: Write>(&mut self, key: Key, mut output: O) -> io::Result<()> {
		use platform::key::{Value, Button};

		macro_rules! write {
			() => ();

			(_ # $($modes:ident)|+ => $string:expr, $($rest:tt)*) => ({
				if self.mode.contains($(mode::$modes)|*) {
					return output.write_all($string);
				}

				write!($($rest)*)
			});

			(_ => $string:expr,) => ({
				output.write_all($string)
			});

			($($modifier:ident)|+ # $($modes:ident)|+ => $string:expr, $($rest:tt)*) => ({
				if key.modifier().contains($(key::$modifier)|*) && self.mode.contains($(mode::$modes)|*) {
					return output.write_all($string);
				}

				write!($($rest)*)
			});

			($($modifier:ident)|+ => $string:expr, $($rest:tt)*) => ({
				if key.modifier().contains($(key::$modifier)|*) {
					return output.write_all($string);
				}

				write!($($rest)*)
			});
		}

		if self.mode.contains(mode::KEYBOARD_LOCK) {
			return Ok(());
		}

		debug!(target: "cancer::terminal::key", "key {:?}", key);

		match *key.value() {
			Value::Char(ref string) => {
				if key.modifier().contains(key::ALT) {
					try!(output.write_all(b"\x1B"));
				}

				output.write_all(string.as_bytes())
			}

			Value::Button(Button::Tab) => write! {
				SHIFT => b"\x1B[Z",
				_     => b"\t",
			},

			Value::Button(Button::Escape) => write! {
				_ => b"\x1B",
			},

			Value::Button(Button::Backspace) => write! {
				ALT => b"\x1B\x7F",
				_   => b"\x7F",
			},

			Value::Button(Button::Enter) => write! {
				ALT # CRLF => b"\x1B\r\n",
				ALT        => b"\x1B\r",

				_ # CRLF => b"\r\n",
				_        => b"\r",
			},

			Value::Button(Button::Delete) => write! {
				CTRL # APPLICATION_KEYPAD => b"\x1B[3;5~",
				CTRL                      => b"\x1B[M",

				SHIFT # APPLICATION_KEYPAD => b"\x1B[3;2~",
				SHIFT                      => b"\x1B[2K",

				_ # APPLICATION_KEYPAD => b"\x1B[3~",
				_                      => b"\x1B[P",
			},

			Value::Button(Button::Insert) => write! {
				CTRL # APPLICATION_KEYPAD => b"\x1B[2;5~",
				CTRL                      => b"\x1B[L",

				SHIFT # APPLICATION_KEYPAD => b"\x1B[2;2~",
				SHIFT                      => b"\x1B[4l",

				_ # APPLICATION_KEYPAD => b"\x1B[2~",
				_                      => b"\x1B[M",
			},

			Value::Button(Button::Home) => write! {
				SHIFT # APPLICATION_CURSOR => b"\x1B[1;2H",
				SHIFT                      => b"\x1B[2J",

				_ # APPLICATION_CURSOR => b"\x1B[H",
				_                      => b"\x1B[7~",
			},

			Value::Button(Button::End) => write! {
				CTRL # APPLICATION_KEYPAD => b"\x1B[1;5F",
				CTRL                      => b"\x1B[J",

				SHIFT # APPLICATION_KEYPAD => b"\x1B[1;2F",
				SHIFT                      => b"\x1B[K",

				_ => b"\x1B[8~",
			},

			Value::Button(Button::PageUp) => write! {
				CTRL  => b"\x1B[5;5~",
				SHIFT => b"\x1B[5;2~",
				_     => b"\x1B[5~",
			},

			Value::Button(Button::PageDown) => write! {
				CTRL  => b"\x1B[6;5~",
				SHIFT => b"\x1B[6;2~",
				_     => b"\x1B[6~",
			},

			Value::Button(Button::Up) => write! {
				CTRL  => b"\x1B[1;5A",
				ALT   => b"\x1B[1;3A",
				SHIFT => b"\x1B[1;2A",

				_ # APPLICATION_CURSOR => b"\x1BOA",
				_                      => b"\x1B[A",
			},

			Value::Button(Button::Down) => write! {
				CTRL  => b"\x1B[1;5B",
				ALT   => b"\x1B[1;3B",
				SHIFT => b"\x1B[1;2B",

				_ # APPLICATION_CURSOR => b"\x1BOB",
				_                      => b"\x1B[B",
			},

			Value::Button(Button::Right) => write! {
				CTRL  => b"\x1B[1;5C",
				ALT   => b"\x1B[1;3C",
				SHIFT => b"\x1B[1;2C",

				_ # APPLICATION_CURSOR => b"\x1BOC",
				_                      => b"\x1B[C",
			},

			Value::Button(Button::Left) => write! {
				CTRL  => b"\x1B[1;5D",
				ALT   => b"\x1B[1;3D",
				SHIFT => b"\x1B[1;2D",

				_ # APPLICATION_CURSOR => b"\x1BOD",
				_                      => b"\x1B[D",
			},

			Value::Button(Button::F(1)) => write! {
				CTRL  => b"\x1B[1;5P",
				ALT   => b"\x1B[1;3P",
				LOGO  => b"\x1B[1;6P",
				SHIFT => b"\x1B[1;2P",
				_     => b"\x1BOP",
			},

			Value::Button(Button::F(2)) => write! {
				CTRL  => b"\x1B[1;5Q",
				ALT   => b"\x1B[1;3Q",
				LOGO  => b"\x1B[1;6Q",
				SHIFT => b"\x1B[1;2Q",
				_     => b"\x1BOQ",
			},

			Value::Button(Button::F(3)) => write! {
				CTRL  => b"\x1B[1;5R",
				ALT   => b"\x1B[1;3R",
				LOGO  => b"\x1B[1;6R",
				SHIFT => b"\x1B[1;2R",
				_     => b"\x1BOR",
			},

			Value::Button(Button::F(4)) => write! {
				CTRL  => b"\x1B[1;5S",
				ALT   => b"\x1B[1;3S",
				LOGO  => b"\x1B[1;6S",
				SHIFT => b"\x1B[1;2S",
				_     => b"\x1BOS",
			},

			Value::Button(Button::F(5)) => write! {
				CTRL  => b"\x1B[15;5~",
				ALT   => b"\x1B[15;3~",
				LOGO  => b"\x1B[15;6~",
				SHIFT => b"\x1B[15;2~",
				_     => b"\x1B[15~",
			},

			Value::Button(Button::F(6)) => write! {
				CTRL  => b"\x1B[17;5~",
				ALT   => b"\x1B[17;3~",
				LOGO  => b"\x1B[17;6~",
				SHIFT => b"\x1B[17;2~",
				_     => b"\x1B[17~",
			},

			Value::Button(Button::F(7)) => write! {
				CTRL  => b"\x1B[18;5~",
				ALT   => b"\x1B[18;3~",
				LOGO  => b"\x1B[18;6~",
				SHIFT => b"\x1B[18;2~",
				_     => b"\x1B[18~",
			},

			Value::Button(Button::F(8)) => write! {
				CTRL  => b"\x1B[19;5~",
				ALT   => b"\x1B[19;3~",
				LOGO  => b"\x1B[19;6~",
				SHIFT => b"\x1B[19;2~",
				_     => b"\x1B[19~",
			},

			Value::Button(Button::F(9)) => write! {
				CTRL  => b"\x1B[20;5~",
				ALT   => b"\x1B[20;3~",
				LOGO  => b"\x1B[20;6~",
				SHIFT => b"\x1B[20;2~",
				_     => b"\x1B[20~",
			},

			Value::Button(Button::F(10)) => write! {
				CTRL  => b"\x1B[21;5~",
				ALT   => b"\x1B[21;3~",
				LOGO  => b"\x1B[21;6~",
				SHIFT => b"\x1B[21;2~",
				_     => b"\x1B[21~",
			},

			Value::Button(Button::F(11)) => write! {
				CTRL  => b"\x1B[23;5~",
				ALT   => b"\x1B[23;3~",
				LOGO  => b"\x1B[23;6~",
				SHIFT => b"\x1B[23;2~",
				_     => b"\x1B[23~",
			},

			Value::Button(Button::F(12)) => write! {
				CTRL  => b"\x1B[24;5~",
				ALT   => b"\x1B[24;3~",
				LOGO  => b"\x1B[24;6~",
				SHIFT => b"\x1B[24;2~",
				_     => b"\x1B[24~",
			},

			Value::Button(Button::F(13)) => write! {
				_ => b"\x1B[1;2P",
			},

			Value::Button(Button::F(14)) => write! {
				_ => b"\x1B[1;2Q",
			},

			Value::Button(Button::F(15)) => write! {
				_ => b"\x1B[1;2R",
			},

			Value::Button(Button::F(16)) => write! {
				_ => b"\x1B[1;2S",
			},

			Value::Button(Button::F(17)) => write! {
				_ => b"\x1B[15;2~",
			},

			Value::Button(Button::F(18)) => write! {
				_ => b"\x1B[17;2~",
			},

			Value::Button(Button::F(19)) => write! {
				_ => b"\x1B[18;2~",
			},

			Value::Button(Button::F(20)) => write! {
				_ => b"\x1B[19;2~",
			},

			Value::Button(Button::F(21)) => write! {
				_ => b"\x1B[20;2~",
			},

			Value::Button(Button::F(22)) => write! {
				_ => b"\x1B[21;2~",
			},

			Value::Button(Button::F(23)) => write! {
				_ => b"\x1B[23;2~",
			},

			Value::Button(Button::F(24)) => write! {
				_ => b"\x1B[24;2~",
			},

			Value::Button(Button::F(25)) => write! {
				_ => b"\x1B[1;5P",
			},

			Value::Button(Button::F(26)) => write! {
				_ => b"\x1B[1;5Q",
			},

			Value::Button(Button::F(27)) => write! {
				_ => b"\x1B[1;5R",
			},

			Value::Button(Button::F(28)) => write! {
				_ => b"\x1B[1;5S",
			},

			Value::Button(Button::F(29)) => write! {
				_ => b"\x1B[15;5~",
			},

			Value::Button(Button::F(30)) => write! {
				_ => b"\x1B[17;5~",
			},

			Value::Button(Button::F(31)) => write! {
				_ => b"\x1B[18;5~",
			},

			Value::Button(Button::F(32)) => write! {
				_ => b"\x1B[19;5~",
			},

			Value::Button(Button::F(33)) => write! {
				_ => b"\x1B[20;5~",
			},

			Value::Button(Button::F(34)) => write! {
				_ => b"\x1B[21;5~",
			},

			Value::Button(Button::F(35)) => write! {
				_ => b"\x1B[23;5~",
			},

			Value::Button(Button::F(_)) =>
				unreachable!(),

			Value::Keypad(..) =>
				unimplemented!(),
		}
	}

	/// Handle mouse inputs.
	pub fn mouse<O: Write>(&mut self, mouse: Mouse, mut output: O) -> io::Result<()> {
		debug!(target: "cancer::terminal::mouse", "mouse {:?}", mouse);

		// If none of the mouse reporting modes are set, bail out.
		if !self.mode.intersects(mode::MOUSE) {
			return Ok(());
		}

		// Build the proper click event.
		let click = match mouse {
			Mouse::Click(click) =>
				click,

			Mouse::Motion(motion) => {
				// If no button is being clicked, motions aren't reported.
				if let Some(mut click) = self.click {
					// Don't report the same position twice.
					if click.position == motion.position {
						return Ok(());
					}

					click.position = motion.position;
					click
				}
				else if self.mode.contains(mode::MOUSE_MANY) {
					mouse::Click {
						press:    false,
						modifier: motion.modifier,
						button:   mouse::Button::Middle,
						position: motion.position,
					}
				}
				else {
					return Ok(());
				}
			}
		};

		// Reset the click on button release.
		if !click.press {
			self.click = None;
		}
		else if click.button != mouse::Button::Up && click.button != mouse::Button::Down {
			self.click = Some(click);
		}

		let mut button = if !self.mode.contains(mode::MOUSE_SGR) && !click.press {
			3
		}
		else {
			match click.button {
				mouse::Button::Left   => 0,
				mouse::Button::Middle => 1,
				mouse::Button::Right  => 2,
				mouse::Button::Up     => 64,
				mouse::Button::Down   => 65,
			}
		};

		if !self.mode.contains(mode::MOUSE_X10) {
			if click.modifier.contains(key::SHIFT) {
				button += 4;
			}

			if click.modifier.contains(key::ALT) {
				button += 8;
			}

			if click.modifier.contains(key::CTRL) {
				button += 16;
			}
		}

		if self.mode.contains(mode::MOUSE_SGR) {
			try!(write!(output, "\x1B[<{button};{x};{y}{mode}",
				mode = if click.press { 'M' } else { 'm' },
				button = button, x = click.position.x, y = click.position.y));
		}
		else if click.position.x < 223 && click.position.y < 223 {
			try!(output.write_all(b"\x1B[M"));
			try!(output.write_all(&[
				32 + button,
				32 + click.position.x as u8 + 1,
				32 + click.position.y as u8 + 1]));
		}

		Ok(())
	}

	/// Handle output from the tty.
	pub fn input<I: AsRef<[u8]>, O: Write>(&mut self, input: I, mut output: O) -> error::Result<(vec::IntoIter<Action>, touched::Iter)> {
		// Juggle the incomplete buffer cache and the real input.
		let     input  = input.as_ref();
		let mut buffer = self.cache.take();

		if let Some(buffer) = buffer.as_mut() {
			buffer.extend_from_slice(input);
		}

		let     buffer  = buffer.as_ref();
		let mut input   = buffer.as_ref().map(AsRef::as_ref).unwrap_or(input);
		let mut actions = Vec::new();

		debug!(target: "cancer::terminal::input::raw", "input: {:?}", input);

		loop {
			if input.is_empty() {
				break;
			}

			// Try to parse the rest of the input.
			let item = match control::parse(input) {
				// No control code.
				control::Result::Error(_) => {
					let kind = match input::parse(input) {
						Input::Error(0) => {
							input = &input[1..];
							input::Kind::Unicode("�")
						}

						Input::Error(length) => {
							input = &input[length..];
							input::Kind::Unicode("�")
						}

						Input::Incomplete(_) => {
							debug!(target: "cancer::terminal::input", "incomplete input: {:?}", input);

							self.cache = Some(input.to_vec());
							break;
						}

						Input::Done(rest, value) => {
							input = rest;
							value
						}
					};

					match kind {
						input::Kind::Unicode(string) => {
							for ch in string.graphemes(true) {
								self.insert(ch);
							}
						}

						input::Kind::Ascii(string) => {
							for i in 0 .. string.len() {
								self.insert(unsafe { str::from_utf8_unchecked(&string[i .. i + 1]) });
							}
						}
					}

					continue;
				}

				// The given input isn't a complete sequence, cache the current input.
				control::Result::Incomplete(_) => {
					debug!(target: "cancer::terminal::input", "incomplete input: {:?}", input);

					self.cache = Some(input.to_vec());
					break;
				}

				// We got a sequence or a raw input.
				control::Result::Done(rest, item) => {
					input = rest;
					item
				}
			};

			debug!(target: "cancer::terminal::input::parsed", "item: {:?}", item);

			actions.extend(self.control(item, output.by_ref())?);
		}

		Ok((actions.into_iter(), self.touched.iter(self.region)))
	}

	fn control<O: Write>(&mut self, control: Control, mut output: O) -> error::Result<Vec<Action>> {
		let mut actions = Vec::new();

		match control {
			// Attributes.
			Control::C1(C1::ControlSequence(CSI::DeviceAttributes(0))) => {
				try!(output.write_all(b"\033[?64;6;21c"));
			}

			Control::C1(C1::ControlSequence(CSI::DeviceStatusReport(CSI::Report::CursorPosition))) => {
				try!(control::format_to(output.by_ref(),
					&CSI::CursorPositionReport { x: self.cursor.x(), y: self.cursor.y() }, true));
			}

			Control::DEC(DEC::ScrollRegion { top, bottom }) => {
				let mut top    = top;
				let mut bottom = util::clamp(bottom.unwrap_or(self.region.height),
					0, self.region.height - 1);

				if top > bottom {
					mem::swap(&mut top, &mut bottom);
				}

				self.cursor.scroll = (top, bottom);
				term!(self; cursor Position(Some(0), Some(0)));
			}

			Control::C1(C1::ControlSequence(CSI::Set(modes))) => {
				debug!(target: "cancer::terminal::mode::set", "set ECMA modes: {:?}", modes);

				for mode in modes {
					match mode {
						CSI::Mode::KeyboardLock =>
							self.mode.insert(mode::KEYBOARD_LOCK),

						CSI::Mode::InsertionReplacement =>
							self.mode.insert(mode::INSERT),

						CSI::Mode::SendReceive =>
							self.mode.insert(mode::ECHO),

						CSI::Mode::LineFeed =>
							self.mode.insert(mode::CRLF),

						mode =>
							debug!(target: "cancer::terminal::unhandled", "unhandled set: {:?}", mode)
					}
				}
			}

			Control::DEC(DEC::Set(modes)) => {
				debug!(target: "cancer::terminal::mode::set", "set DEC modes: {:?}", modes);

				for mode in modes {
					match mode {
						DEC::Mode::ApplicationCursor =>
							self.mode.insert(mode::APPLICATION_CURSOR),

						DEC::Mode::ReverseVideo => {
							self.mode.insert(mode::REVERSE);
							self.touched.all();
						}

						DEC::Mode::Origin => {
							self.cursor.state.insert(cursor::ORIGIN);
							term!(self; cursor Position(Some(0), Some(0)));
						}

						DEC::Mode::AutoWrap =>
							self.mode.insert(mode::WRAP),

						DEC::Mode::CursorVisible =>
							self.cursor.state.insert(cursor::VISIBLE),

						DEC::Mode::SmallFont =>
							actions.push(Action::Resize(132, 24)),

						mode =>
							debug!(target: "cancer::terminal::unhandled", "unhandled set: {:?}", mode)
					}
				}
			}

			Control::C1(C1::ControlSequence(CSI::Private(b'h', None, args))) => {
				debug!(target: "cancer::terminal::mode::set", "set private modes: {:?}", args);

				for arg in args.into_iter().flat_map(Option::into_iter) {
					match arg {
						1004 =>
							self.mode.insert(mode::FOCUS),

						2004 =>
							self.mode.insert(mode::BRACKETED_PASTE),

						9 | 1000 | 1002 | 1003 => {
							self.mode.remove(mode::MOUSE);
							self.mode.insert(match arg {
								9    => mode::MOUSE_X10,
								1000 => mode::MOUSE_BUTTON,
								1002 => mode::MOUSE_MOTION,
								1003 => mode::MOUSE_MANY,
								_    => unreachable!()
							});
						}

						1006 =>
							self.mode.insert(mode::MOUSE_SGR),

						n =>
							debug!(target: "cancer::terminal::unhandled", "unhandled set: {}", n)
					}
				}
			}

			Control::C1(C1::ControlSequence(CSI::Reset(modes))) => {
				debug!(target: "cancer::terminal::mode::reset", "reset ECMA modes: {:?}", modes);

				for mode in modes {
					match mode {
						CSI::Mode::KeyboardLock =>
							self.mode.remove(mode::KEYBOARD_LOCK),

						CSI::Mode::InsertionReplacement =>
							self.mode.remove(mode::INSERT),

						CSI::Mode::SendReceive =>
							self.mode.remove(mode::ECHO),

						CSI::Mode::LineFeed =>
							self.mode.remove(mode::CRLF),

						mode =>
							debug!(target: "cancer::terminal::unhandled", "unhandled reset: {:?}", mode)
					}
				}
			}

			Control::DEC(DEC::Reset(modes)) => {
				debug!(target: "cancer::terminal::mode::reset", "reset DEC modes: {:?}", modes);

				for mode in modes {
					match mode {
						DEC::Mode::ApplicationCursor =>
							self.mode.remove(mode::APPLICATION_CURSOR),

						DEC::Mode::ReverseVideo => {
							self.mode.remove(mode::REVERSE);
							self.touched.all();
						}

						DEC::Mode::Origin =>
							self.cursor.state.remove(cursor::ORIGIN),

						DEC::Mode::AutoWrap =>
							self.mode.remove(mode::WRAP),

						DEC::Mode::CursorVisible =>
							self.cursor.state.remove(cursor::VISIBLE),

						DEC::Mode::SmallFont =>
							actions.push(Action::Resize(80, 24)),

						mode =>
							debug!(target: "cancer::terminal::unhandled", "unhandled reset: {:?}", mode)
					}
				}
			}

			Control::C1(C1::ControlSequence(CSI::Private(b'l', None, args))) => {
				debug!(target: "cancer::terminal::mode::reset", "reset private modes: {:?}", args);

				for arg in args.into_iter().flat_map(Option::into_iter) {
					match arg {
						1004 =>
							self.mode.remove(mode::FOCUS),

						2004 =>
							self.mode.remove(mode::BRACKETED_PASTE),

						9 | 1000 | 1002 | 1003 =>
							self.mode.remove(mode::MOUSE),

						1006 =>
							self.mode.remove(mode::MOUSE_SGR),

						n =>
							debug!(target: "cancer::terminal::unhandled", "unhandled reset: {:?}", n)
					}
				}
			}

			Control::DEC(DEC::ApplicationKeypad(true)) => {
				self.mode.insert(mode::APPLICATION_KEYPAD);
			}

			Control::DEC(DEC::ApplicationKeypad(false)) => {
				self.mode.remove(mode::APPLICATION_KEYPAD);
			}

			Control::C1(C1::ControlSequence(CSI::SaveCursor)) |
			Control::DEC(DEC::SaveCursor) => {
				self.saved = Some(self.cursor.clone());
			}

			Control::C1(C1::ControlSequence(CSI::RestoreCursor)) |
			Control::DEC(DEC::RestoreCursor) => {
				if let Some(saved) = self.saved.clone() {
					self.cursor = saved;
				}
			}

			// Charset.
			Control::DEC(DEC::SelectCharset(i, charset)) => {
				if self.cursor.charsets.len() >= i as usize {
					self.cursor.charsets[i as usize] = charset;
				}
			}

			Control::C0(C0::ShiftIn) => {
				self.cursor.charset = 0;
			}

			Control::C0(C0::ShiftOut) => {
				self.cursor.charset = 1;
			}

			// Movement functions.
			Control::C0(C0::CarriageReturn) => {
				term!(self; cursor Position(Some(0), None));
			}

			Control::C0(C0::LineFeed) => {
				if term!(self; cursor Down(1)).is_some() {
					term!(self; scroll! up 1);
				}
			}

			Control::C0(C0::Backspace) => {
				term!(self; cursor Left(1));
			}

			Control::C1(C1::ControlSequence(CSI::CursorPosition { x, y })) => {
				term!(self; cursor Position(Some(x), Some(y)));
			}

			Control::C1(C1::ControlSequence(CSI::CursorVerticalPosition(n))) => {
				term!(self; cursor Position(None, Some(n)));
			}

			Control::C1(C1::ControlSequence(CSI::CursorHorizontalPosition(n))) => {
				term!(self; cursor Position(Some(n), None));
			}

			Control::C1(C1::ControlSequence(CSI::CursorUp(n))) => {
				term!(self; cursor Up(n));
			}

			Control::C1(C1::ControlSequence(CSI::CursorDown(n))) => {
				term!(self; cursor Down(n));
			}

			Control::C1(C1::ControlSequence(CSI::CursorBack(n))) => {
				term!(self; cursor Left(n));
			}

			Control::C1(C1::ControlSequence(CSI::CursorForward(n))) => {
				term!(self; cursor Right(n));
			}

			Control::C1(C1::Index) => {
				if term!(self; cursor Down(1)).is_some() {
					term!(self; scroll up 1);
				}
			}

			Control::C1(C1::ReverseIndex) => {
				if term!(self; cursor Up(1)).is_some() {
					term!(self; scroll down 1);
				}
			}

			Control::C1(C1::ControlSequence(CSI::ScrollUp(n))) => {
				term!(self; scroll up n);
			}

			Control::C1(C1::ControlSequence(CSI::ScrollDown(n))) => {
				term!(self; scroll down n);
			}

			Control::DEC(DEC::BackIndex) => {
				if self.cursor.x() == 0 {
					self.grid.left(1);
					self.touched.all();
				}
				else {
					term!(self; cursor Left(1));
				}
			}

			Control::DEC(DEC::ForwardIndex) => {
				if self.cursor.x() == self.region.width - 1 {
					self.grid.right(1);
					self.touched.all();
				}
				else {
					term!(self; cursor Right(1));
				}
			}

			Control::C1(C1::NextLine) => {
				if term!(self; cursor Down(1)).is_some() {
					term!(self; scroll up 1);
				}

				term!(self; cursor Position(Some(0), None));
			}

			// Erase functions.
			Control::C1(C1::ControlSequence(CSI::EraseDisplay(CSI::Erase::ToEnd))) => {
				let (x, y) = term!(self; cursor);

				for x in x .. self.region.width {
					self.touched.mark(x, y);
					self.grid.get_mut(x, y).make_empty(self.cursor.style().clone());
				}

				for y in y + 1 .. self.region.height {
					self.touched.line(y);

					for x in 0 .. self.region.width {
						self.grid.get_mut(x, y).make_empty(self.cursor.style().clone());
					}
				}
			}

			Control::C1(C1::ControlSequence(CSI::EraseDisplay(CSI::Erase::ToStart))) => {
				let (x, y) = term!(self; cursor);

				for x in 0 ... x {
					self.touched.mark(x, y);
					self.grid.get_mut(x, y).make_empty(self.cursor.style().clone());
				}

				for y in 0 .. y {
					self.touched.line(y);

					for x in 0 .. self.region.width {
						self.grid.get_mut(x, y).make_empty(self.cursor.style().clone());
					}
				}
			}

			Control::C1(C1::ControlSequence(CSI::EraseDisplay(CSI::Erase::All))) => {
				self.touched.all();

				for y in 0 .. self.region.height {
					for x in 0 .. self.region.width {
						self.grid.get_mut(x, y).make_empty(self.cursor.style().clone());
					}
				}
			}

			Control::C1(C1::ControlSequence(CSI::EraseLine(CSI::Erase::ToEnd))) => {
				let (x, y) = term!(self; cursor);

				for x in x .. self.region.width {
					self.touched.mark(x, y);
					self.grid.get_mut(x, y).make_empty(self.cursor.style().clone());
				}
			}

			Control::C1(C1::ControlSequence(CSI::EraseLine(CSI::Erase::ToStart))) => {
				let (x, y) = term!(self; cursor);

				for x in 0 ... x {
					self.touched.mark(x, y);
					self.grid.get_mut(x, y).make_empty(self.cursor.style().clone());
				}
			}

			Control::C1(C1::ControlSequence(CSI::EraseLine(CSI::Erase::All))) => {
				let y = self.cursor.y();
				self.touched.line(y);

				for x in 0 .. self.region.width {
					self.grid.get_mut(x, y).make_empty(self.cursor.style().clone());
				}
			}

			Control::C1(C1::ControlSequence(CSI::EraseCharacter(n))) => {
				let (x, y) = term!(self; cursor);

				for x in x .. x + n {
					self.grid.get_mut(x, y).make_empty(self.cursor.style().clone());
					self.touched.mark(x, y);
				}

				term!(self; clean references (x + n, y));
			}

			Control::C1(C1::ControlSequence(CSI::DeleteLine(n))) => {
				term!(self; scroll up n from self.cursor.y());
			}

			Control::C1(C1::ControlSequence(CSI::DeleteCharacter(n))) => {
				let (x, y) = term!(self; cursor);
				self.grid.delete(x, y, n);

				for x in x .. self.region.width {
					self.touched.mark(x, y);
				}
			}

			// Insertion functions.
			Control::DEC(DEC::AlignmentTest) => {
				for (x, y) in self.region.absolute() {
					self.grid.get_mut(x, y).make_occupied("E", self.cursor.style().clone());
				}

				self.touched.all();
			}

			Control::C1(C1::ControlSequence(CSI::InsertLine(n))) => {
				term!(self; scroll down n from self.cursor.y());
			}

			Control::C1(C1::ControlSequence(CSI::InsertCharacter(n))) => {
				let (x, y) = term!(self; cursor);
				self.grid.insert(x, y, n);

				for x in x .. self.region.width {
					self.touched.mark(x, y);
				}
			}

			Control::C0(C0::HorizontalTabulation) => {
				term!(self; tab 1);
			}

			Control::C1(C1::ControlSequence(CSI::CursorForwardTabulation(n))) => {
				term!(self; tab n as i32);
			}

			Control::C1(C1::ControlSequence(CSI::CursorBackTabulation(n))) => {
				term!(self; tab -(n as i32));
			}

			Control::C1(C1::HorizontalTabulationSet) => {
				let (x, _) = term!(self; cursor);
				self.tabs.set(x, true);
			}

			Control::C1(C1::ControlSequence(CSI::TabulationClear(CSI::Tabulation::AllCharacters))) => {
				self.tabs.clear();
			}

			Control::C1(C1::ControlSequence(CSI::TabulationClear(CSI::Tabulation::Character))) => {
				let (x, _) = term!(self; cursor);
				self.tabs.set(x, false);
			}

			// Style functions.
			Control::C1(C1::ControlSequence(CSI::SelectGraphicalRendition(attrs))) => {
				fn to_rgba(color: &SGR::Color) -> Rgba<f64> {
					match *color {
						SGR::Color::Transparent =>
							Rgba::new(0.0, 0.0, 0.0, 0.0),

						SGR::Color::Rgb(r, g, b) =>
							Rgba::new_u8(r, g, b, 255),

						SGR::Color::Cmy(c, m, y) => {
							let c = c as f64 / 255.0;
							let m = m as f64 / 255.0;
							let y = y as f64 / 255.0;

							Rgba::new(
								1.0 - c,
								1.0 - m,
								1.0 - y,
								1.0)
						}

						SGR::Color::Cmyk(c, m, y, k) => {
							let c = c as f64 / 255.0;
							let m = m as f64 / 255.0;
							let y = y as f64 / 255.0;
							let k = k as f64 / 255.0;

							Rgba::new(
								1.0 - (c * (1.0 - k) + k),
								1.0 - (m * (1.0 - k) + k),
								1.0 - (y * (1.0 - k) + k),
								1.0)
						}

						_ => unreachable!()
					}
				}

				let mut style = **self.cursor.style();

				for mut attr in attrs {
					if self.config.style().bold().is_bright() {
						match attr {
							SGR::Foreground(SGR::Color::Index(ref mut n)) if *n < 8 => {
								self.cursor.bright = Some(*n);

								if style.attributes.contains(style::BOLD) {
									*n += 8;
								}
							}

							SGR::Reset | SGR::Foreground(_) => {
								self.cursor.bright = None
							}

							SGR::Font(SGR::Weight::Normal) | SGR::Font(SGR::Weight::Faint) => {
								if let Some(n) = self.cursor.bright {
									style.foreground = Some(*self.config.color().get(n));
								}
							}

							SGR::Font(SGR::Weight::Bold) => {
								if let Some(n) = self.cursor.bright {
									style.foreground = Some(*self.config.color().get(n + 8));
								}
							}

							_ => ()
						}
					}

					match attr {
						SGR::Reset =>
							style = Style::default(),

						SGR::Italic(true) =>
							style.attributes.insert(style::ITALIC),
						SGR::Italic(false) =>
							style.attributes.remove(style::ITALIC),

						SGR::Underline(true) =>
							style.attributes.insert(style::UNDERLINE),
						SGR::Underline(false) =>
							style.attributes.remove(style::UNDERLINE),

						SGR::Blink(true) =>
							style.attributes.insert(style::BLINK),
						SGR::Blink(false) =>
							style.attributes.remove(style::BLINK),

						SGR::Reverse(true) =>
							style.attributes.insert(style::REVERSE),
						SGR::Reverse(false) =>
							style.attributes.remove(style::REVERSE),

						SGR::Invisible(true) =>
							style.attributes.insert(style::INVISIBLE),
						SGR::Invisible(false) =>
							style.attributes.remove(style::INVISIBLE),

						SGR::Struck(true) =>
							style.attributes.insert(style::STRUCK),
						SGR::Struck(false) =>
							style.attributes.remove(style::STRUCK),

						SGR::Font(SGR::Weight::Normal) =>
							style.attributes.remove(style::BOLD | style::FAINT),

						SGR::Font(SGR::Weight::Bold) => {
							style.attributes.remove(style::FAINT);
							style.attributes.insert(style::BOLD);
						}

						SGR::Font(SGR::Weight::Faint) => {
							style.attributes.remove(style::BOLD);
							style.attributes.insert(style::FAINT);
						}

						SGR::Foreground(SGR::Color::Default) =>
							style.foreground = Some(*self.config.style().color().foreground()),

						SGR::Foreground(SGR::Color::Index(n)) =>
							style.foreground = Some(*self.config.color().get(n)),

						SGR::Foreground(ref color) =>
							style.foreground = Some(to_rgba(color)),

						SGR::Background(SGR::Color::Default) =>
							style.background = Some(*self.config.style().color().background()),

						SGR::Background(SGR::Color::Index(n)) =>
							style.background = Some(*self.config.color().get(n)),

						SGR::Background(ref color) =>
							style.background = Some(to_rgba(color)),
					}
				}

				self.cursor.update(style);
			}

			Control::DEC(DEC::CursorStyle(n)) => {
				match n {
					0 => {
						if self.config.style().cursor().blink() {
							self.cursor.state.insert(cursor::BLINK);
						}

						self.cursor.shape = self.config.style().cursor().shape();
					}

					1 => {
						self.cursor.state.insert(cursor::BLINK);
						self.cursor.shape = Shape::Block;
					}

					2 => {
						self.cursor.state.remove(cursor::BLINK);
						self.cursor.shape = Shape::Block;
					}

					3 => {
						self.cursor.state.insert(cursor::BLINK);
						self.cursor.shape = Shape::Line;
					}

					4 => {
						self.cursor.state.remove(cursor::BLINK);
						self.cursor.shape = Shape::Line;
					}

					5 => {
						self.cursor.state.insert(cursor::BLINK);
						self.cursor.shape = Shape::Beam;
					}

					6 => {
						self.cursor.state.remove(cursor::BLINK);
						self.cursor.shape = Shape::Beam;
					}

					_ => ()
				}
			}

			// Secret control codes.
			Control::C1(C1::OperatingSystemCommand(cmd)) if cmd.starts_with("0;") ||
				                                              cmd.starts_with("1;") ||
				                                              cmd.starts_with("2;") ||
				                                              cmd.starts_with("k;") => {
				actions.push(Action::Title(String::from(&cmd[2..])));
			}

			Control::C1(C1::OperatingSystemCommand(cmd)) if cmd.starts_with("cursor:") => {
				let mut parts = cmd.split(':').skip(1);

				match parts.next() {
					Some("fg") => {
						let     desc  = parts.next().unwrap_or("-");
						let mut color = *self.config.style().cursor().foreground();

						if let Some(c) = config::util::to_color(desc) {
							color = c;
						}

						self.cursor.foreground = color;
					}

					Some("bg") => {
						let     desc  = parts.next().unwrap_or("-");
						let mut color = *self.config.style().cursor().background();

						if let Some(c) = config::util::to_color(desc) {
							color = c;
						}

						self.cursor.background = color;
					}

					_ => ()
				}
			}

			Control::C1(C1::OperatingSystemCommand(cmd)) if cmd.starts_with("clipboard:") => {
				let mut parts = cmd.split(':').skip(1);

				match parts.next() {
					Some("set") => {
						if let (Some(name), Some(string)) = (parts.next(), parts.next()) {
							actions.push(Action::Clipboard(name.into(), string.into()));
						}
					}

					_ => ()
				}
			}

			code => {
				debug!(target: "cancer::terminal::unhandled", "unhandled control code: {:?}", code);
			}
		}

		Ok(actions)
	}

	fn insert<T: AsRef<str>>(&mut self, ch: T) {
		let mut ch = ch.as_ref();

		if term!(self; charset) == DEC::Charset::DEC(DEC::charset::DEC::Graphic) {
			ch = match ch {
				"A" => "↑",
				"B" => "↓",
				"C" => "→",
				"D" => "←",
				"E" => "█",
				"F" => "▚",
				"G" => "☃",
				"_" => " ",
				"`" => "◆",
				"a" => "▒",
				"b" => "␉",
				"c" => "␌",
				"d" => "␍",
				"e" => "␊",
				"f" => "°",
				"g" => "±",
				"h" => "␤",
				"i" => "␋",
				"j" => "┘",
				"k" => "┐",
				"l" => "┌",
				"m" => "└",
				"n" => "┼",
				"o" => "⎺",
				"p" => "⎻",
				"q" => "─",
				"r" => "⎼",
				"s" => "⎽",
				"t" => "├",
				"u" => "┤",
				"v" => "┴",
				"w" => "┬",
				"x" => "│",
				"y" => "≤",
				"z" => "≥",
				"{" => "π",
				"|" => "≠",
				"}" => "£",
				"~" => "·",
				_   => ch,
			};
		}

		let width = ch.width() as u32;

		// Bail out if it cannot be displayed.
		if width == 0 {
			return;
		}

		if self.mode.contains(mode::WRAP) && self.cursor.wrap() {
			if term!(self; cursor Down(1)).is_some() {
				term!(self; scroll! up 1);
			}

			term!(self; cursor Position(Some(0), None));
			let (_, y) = term!(self; cursor);
			self.grid.wrap(y);
		}

		let (x, y) = term!(self; cursor);

		// If the inserted character is all whitespace, make the cell empty,
		// otherwise make it occupied.
		if ch.chars().all(char::is_whitespace) {
			for x in x .. x + width {
				self.grid.get_mut(x, y).make_empty(self.cursor.style().clone());
				self.touched.mark(x, y);
			}

			term!(self; clean references (x + width, y));
		}
		else {
			self.grid.get_mut(x, y).make_occupied(ch, self.cursor.style().clone());
			self.touched.mark(x, y);

			for (i, x) in (x + 1 .. x + width).enumerate() {
				self.grid.get_mut(x, y).make_reference(i as u8 + 1);
			}

			term!(self; clean references (x + width, y));
		}

		// If the character overflows the region, mark it for wrapping.
		if self.cursor.x() + width >= self.region.width {
			self.cursor.state.insert(cursor::WRAP);
		}
		else {
			term!(self; cursor Right(width));
		}
	}
}

impl Access for Terminal {
	fn access(&self, x: u32, y: u32) -> &Cell {
		self.grid.get(x, y)
	}
}
