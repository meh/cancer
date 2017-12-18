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

use std::ptr;
use std::io::{self, Write};
use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel, sync_channel};

use libc::{c_void, c_char, c_ushort, c_int, winsize};
use libc::{SIGCHLD, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGALRM, SIG_DFL, TIOCSCTTY, TIOCSWINSZ};
use libc::{close, read, write, openpty, fork, setsid, dup2, signal, ioctl, getpwuid, getuid, execvp};
use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

use error::{self, Error};

#[derive(Debug)]
pub struct Tty {
	id:   c_int,
	fd:   c_int,
	font: (u32, u32),

	input:  Sender<Vec<u8>>,
	output: Option<Receiver<Vec<u8>>>,
	buffer: Option<Vec<u8>>,
}

impl Tty {
	pub fn spawn(term: Option<&str>, program: Option<&str>, font: (u32, u32), size: (u32, u32)) -> error::Result<Self> {
		unsafe {
			let mut size = winsize {
				ws_row:    size.1 as c_ushort,
				ws_col:    size.0 as c_ushort,

				ws_xpixel: (size.0 * font.0) as c_ushort,
				ws_ypixel: (size.1 * font.1) as c_ushort,
			};

			let mut master = 0;
			let mut slave  = 0;

			if openpty(&mut master, &mut slave, ptr::null_mut(), ptr::null_mut(), &mut size) < 0 {
				return Err(Error::Message("failed to open pty".into()));
			}

			match fork() {
				// Fork failed.
				-1 => {
					Err(Error::Message("failed to fork".into()))
				}

				// Into the new process.
				0 => {
					// Create a new process group.
					setsid();

					// Set up fds.
					dup2(slave, 0);
					dup2(slave, 1);
					dup2(slave, 2);

					// Set up the tty.
					if ioctl(slave, TIOCSCTTY as _, ptr::null::<c_void>()) < 0 {
						panic!("ioctl TIOCSCTTY failed");
					}

					// Clean fds.
					close(master);
					close(slave);

					// Execute program.
					execute(term, program);
				}

				// From our process.
				id => {
					// Free the slaves.
					close(slave);

					let (input, output) = Tty::open(master, master);

					Ok(Tty {
						id:   id,
						fd:   master,
						font: font,

						input:  input,
						output: Some(output),
						buffer: None,
					})
				}
			}
		}
	}

	unsafe fn open(input: c_int, output: c_int) -> (Sender<Vec<u8>>, Receiver<Vec<u8>>) {
		let (i_sender, i_receiver) = channel::<Vec<u8>>();
		let (o_sender, o_receiver) = sync_channel::<Vec<u8>>(16);

		// Spawn the reader.
		thread::Builder::new().name("cancer::tty::reader".into()).spawn(move || {
			let mut buffer = [0u8; 64 * 1024];
			let     flags  = fcntl(input, F_GETFL, 0);

			loop {
				let mut consumed = 0usize;

				// First do a blocking read.
				match read(input, buffer.as_mut_ptr() as _, buffer.len() as _) {
					// Stop the thread on failure or EOF.
					-1 | 0 =>
						return,

					n =>
						consumed += n as usize
				}

				// Set as non-blocking and try to read until the buffer is full.
				{
					fcntl(input, F_SETFL, flags | O_NONBLOCK);

					loop {
						let mut offset = &mut buffer[consumed ..];

						match read(input, offset.as_mut_ptr() as _, offset.len() as _) {
							// Break out of the non-blocking loop, any errors or EOF
							// will be handled by the next loop.
							-1 | 0 =>
								break,

							n =>
								consumed += n as usize
						}
					}

					fcntl(input, F_SETFL, flags);
				}

				try!(return o_sender.send((&buffer[.. consumed]).to_vec()));
			}
		}).unwrap();

		// Spawn writer.
		thread::Builder::new().name("cancer::tty::writer".into()).spawn(move || {
			while let Ok(buffer) = i_receiver.recv() {
				let mut buffer = &buffer[..];

				while !buffer.is_empty() {
					match write(output, buffer.as_ptr() as _, buffer.len() as _) {
						-1 | 0 => return,
						n      => buffer = &buffer[n as usize ..],
					}
				}
			}
		}).unwrap();

		(i_sender, o_receiver)
	}

	pub fn output(&mut self) -> Receiver<Vec<u8>> {
		self.output.take().unwrap()
	}

	pub fn resize(&mut self, width: u32, height: u32) -> error::Result<()> {
		unsafe {
			let size = winsize {
				ws_row:    height as c_ushort,
				ws_col:    width as c_ushort,
				ws_xpixel: (self.font.0 * width) as c_ushort,
				ws_ypixel: (self.font.1 * height) as c_ushort,
			};

			if ioctl(self.fd, TIOCSWINSZ as _, &size) < 0 {
				return Err(Error::Message("failed to resize tty".into()));
			}
		}

		Ok(())
	}
}

impl Write for Tty {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		if self.buffer.is_none() {
			self.buffer = Some(Vec::with_capacity(buf.len()));
		}

		self.buffer.as_mut().unwrap().extend_from_slice(buf);

		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		if let Some(buffer) = self.buffer.take() {
			match self.input.send(buffer) {
				Ok(_) =>
					Ok(()),

				Err(e) =>
					Err(io::Error::new(io::ErrorKind::BrokenPipe, e))
			}
		}
		else {
			Ok(())
		}
	}
}

/// Execute the program, this calls `exec` so it will never return, unless
/// something went very wrong, then we're doomed.
unsafe fn execute(term: Option<&str>, program: Option<&str>) -> ! {
	use std::env;
	use std::ffi::{CString, CStr};
	use shlex;

	let passwd  = getpwuid(getuid()).as_mut().expect("no user?");
	let program = if let Some(program) = program {
		program.into()
	}
	else if let Ok(program) = env::var("SHELL") {
		program
	}
	else if *(*passwd).pw_shell != 0 {
		CString::from_raw((*passwd).pw_shell).into_string().unwrap()
	}
	else {
		"/bin/sh".into()
	};

	// Cleanup signals.
	signal(SIGCHLD, SIG_DFL);
	signal(SIGHUP,  SIG_DFL);
	signal(SIGINT,  SIG_DFL);
	signal(SIGQUIT, SIG_DFL);
	signal(SIGTERM, SIG_DFL);
	signal(SIGALRM, SIG_DFL);

	// Cleanup environment.
	env::remove_var("COLUMNS");
	env::remove_var("LINES");
	env::remove_var("TERMCAP");
	env::remove_var("TERMINFO");

	// Fill environment.
	env::set_var("LOGNAME", CStr::from_ptr((*passwd).pw_name).to_str().unwrap());
	env::set_var("USER", CStr::from_ptr((*passwd).pw_name).to_str().unwrap());
	env::set_var("SHELL", &program);
	env::set_var("HOME", CStr::from_ptr((*passwd).pw_dir).to_str().unwrap());
	env::set_var("TERM", term.unwrap_or("cancer-256color"));

	// Parse program line.
	let mut name = shlex::split(&program).unwrap();
	let     args = name.split_off(1);

	// Create arguments for execvpe.
	let     name = CString::new(name.into_iter().next().unwrap()).unwrap();
	let     args = args.into_iter().map(|arg| CString::new(arg).unwrap()).collect::<Vec<CString>>();
	let mut args = args.iter().map(|arg| arg.as_ptr()).collect::<Vec<*const c_char>>();
	args.push(name.as_ptr());
	args.push(ptr::null());

	execvp(name.as_ptr(), args.as_ptr());
	unreachable!();
}
