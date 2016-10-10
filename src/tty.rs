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
use std::fs::File;
use std::os::unix::io::{RawFd, FromRawFd};
use std::io::{self, Write, BufRead, BufReader};
use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, channel};

use libc::{c_void, c_char, c_ushort, c_int, winsize};
use libc::{SIGCHLD, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGALRM, SIG_DFL, TIOCSCTTY, TIOCSWINSZ};
use libc::{close, openpty, fork, setsid, dup2, signal, ioctl, getpwuid, getuid, execvp};

use config::Config;
use error::{self, Error};

#[derive(Debug)]
pub struct Tty {
	id: c_int,
	fd: RawFd,

	input:  Sender<Vec<u8>>,
	output: Option<Receiver<Vec<u8>>>,
	buffer: Option<Vec<u8>>,
}

impl Tty {
	pub fn spawn(config: Arc<Config>, program: Option<String>, width: u32, height: u32) -> error::Result<Self> {
		unsafe {
			let size = winsize {
				ws_row:    height as c_ushort,
				ws_col:    width as c_ushort,
				ws_xpixel: 0,
				ws_ypixel: 0,
			};

			let mut master = 0;
			let mut slave  = 0;

			if openpty(&mut master, &mut slave, ptr::null_mut(), ptr::null(), &size) < 0 {
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

					if ioctl(slave, TIOCSCTTY, ptr::null::<c_void>()) < 0 {
						panic!("ioctl TIOCSCTTY failed");
					}

					// Clean fds.
					close(master);
					close(slave);

					// Execute program.
					execute(&config, program.as_ref().map(AsRef::as_ref));
				}

				// From our process.
				id => {
					close(slave);

					let (i_sender, i_receiver) = channel::<Vec<u8>>();
					let (o_sender, o_receiver) = channel::<Vec<u8>>();

					// Spawn the reader.
					thread::spawn(move || {
						let mut stream = BufReader::new(File::from_raw_fd(master));

						loop {
							let consumed = {
								let buffer = stream.fill_buf().unwrap();
								o_sender.send(buffer.to_vec()).unwrap();
								buffer.len()
							};

							stream.consume(consumed);
						}
					});

					// Spawn writer.
					thread::spawn(move || {
						let mut stream = File::from_raw_fd(master);

						while let Ok(buffer) = i_receiver.recv() {
							stream.write_all(&buffer).unwrap();
						}
					});

					Ok(Tty {
						id: id,
						fd: master,

						input:  i_sender,
						output: Some(o_receiver),
						buffer: None,
					})
				}
			}
		}
	}

	pub fn output(&mut self) -> Receiver<Vec<u8>> {
		self.output.take().unwrap()
	}

	pub fn resize(&mut self, width: u32, height: u32) -> error::Result<()> {
		unsafe {
			let size = winsize {
				ws_row:    height as c_ushort,
				ws_col:    width as c_ushort,
				ws_xpixel: 0,
				ws_ypixel: 0,
			};

			if ioctl(self.fd, TIOCSWINSZ, &size) < 0 {
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

unsafe fn execute(config: &Config, program: Option<&str>) -> ! {
	use std::env;
	use std::ffi::{CString, CStr};
	use shlex;

	let passwd  = getpwuid(getuid()).as_mut().expect("no user?");
	let program = if let Some(program) = program {
		program.into()
	}
	else if let Some(program) = config.environment().program() {
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

	// Fill environment.
	env::set_var("LOGNAME", CStr::from_ptr((*passwd).pw_name).to_str().unwrap());
	env::set_var("USER", CStr::from_ptr((*passwd).pw_name).to_str().unwrap());
	env::set_var("SHELL", &program);
	env::set_var("HOME", CStr::from_ptr((*passwd).pw_dir).to_str().unwrap());
	env::set_var("TERM", "cancer-256color");

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
