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
use std::mem;
use std::fs::File;
use std::os::unix::io::{RawFd, FromRawFd, AsRawFd};
use std::io::{Read, Write, BufRead, BufReader};
use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel};

use libc::{c_void, c_char, c_ushort, c_int, winsize};
use libc::{SIGCHLD, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGALRM, SIG_DFL, TIOCSCTTY};
use libc::{open, close, openpty, fork, setsid, dup2, signal, ioctl, getpwuid, getuid, execvp};

use config::Config;
use error::{self, Error};

pub struct Tty(File);

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
					Ok(Tty(File::from_raw_fd(master)))
				}
			}
		}
	}

	pub fn output(&mut self) -> Receiver<Vec<u8>> {
		let     (sender, receiver) = sync_channel(1);
		let mut stream             = BufReader::new(unsafe { File::from_raw_fd(self.0.as_raw_fd()) });

		thread::spawn(move || {
			loop {
				let consumed = {
					let buffer = stream.fill_buf().unwrap();
					sender.send(buffer.to_vec()).unwrap();

					buffer.len()
				};

				stream.consume(consumed);
			}

			while let Ok(buffer) = stream.fill_buf() {
			}
		});

		receiver
	}

	pub fn write(&mut self, value: &[u8]) -> error::Result<()> {
		Ok(try!(self.0.write_all(value)))
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
