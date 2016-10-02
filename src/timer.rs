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

use std::ops::Deref;
use std::time::Duration;
use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel};

use config::Config;

pub struct Timer(Receiver<Event>);

pub enum Event {
	Blink(bool),
}

impl Timer {
	pub fn spawn(config: Arc<Config>) -> Self {
		let (sender, receiver) = sync_channel(1);

		thread::spawn(move || {
			let     blink    = Duration::from_millis(config.style().blink());
			let mut blinking = false;

			loop {
				sender.send(Event::Blink(blinking)).unwrap();
				blinking = !blinking;

				thread::sleep(blink);
			}
		});

		Timer(receiver)
	}
}

impl Deref for Timer {
	type Target = Receiver<Event>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
