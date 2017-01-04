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

use std::sync::mpsc::Sender;
use std::os::raw::c_void;

use objc::runtime::{Class, Object, Sel, BOOL, YES};
use objc::declare::ClassDecl;
use cocoa::base::{id, nil};
use cocoa::appkit::{NSWindow, NSView};

use platform::quartz::IdRef;
use platform::Event;

#[derive(Debug)]
pub struct Delegate {
	state: Box<State>,
	this:  IdRef,
}

#[derive(Debug)]
pub struct State {
	manager: Sender<Event>,
	window:  IdRef,
	view:    IdRef,
}

impl Delegate {
	unsafe fn class() -> *const Class {
		extern fn window_should_close(this: &Object, _: Sel, _: id) -> BOOL {
			unsafe {
				let state: *mut c_void = *this.get_ivar("state");
				let state = state as *mut State;

				let _ = (*state).manager.send(Event::Closed);

				YES
			}
		}

		extern fn window_did_resize(this: &Object, _: Sel, _: id) {
			unsafe {
				let state: *mut c_void = *this.get_ivar("state");
				let state  = state as *mut State;
				let rect   = NSView::frame(*(*state).view);
				let factor = NSWindow::backingScaleFactor(*(*state).window) as f32;
				let width  = factor * rect.size.width as f32;
				let height = factor * rect.size.height as f32;

				let _ = (*state).manager.send(Event::Resize(width as u32, height as u32));
				let _ = (*state).manager.send(Event::Redraw);
			}
		}

		extern fn window_did_become_key(this: &Object, _: Sel, _: id) {
			unsafe {
				let state: *mut c_void = *this.get_ivar("state");
				let state = state as *mut State;

				let _ = (*state).manager.send(Event::Focus(true));
				let _ = (*state).manager.send(Event::Redraw);
			}
		}

		extern fn window_did_resign_key(this: &Object, _: Sel, _: id) {
			unsafe {
				let state: *mut c_void = *this.get_ivar("state");
				let state = state as *mut State;

				let _ = (*state).manager.send(Event::Focus(false));
			}
		}

		let mut decl = ClassDecl::new("CancerDelegate", Class::get("NSObject").unwrap()).unwrap();

		decl.add_ivar::<*mut c_void>("state");

		decl.add_method(sel!(windowShouldClose:),
			window_should_close as extern fn(&Object, Sel, id) -> BOOL);

		decl.add_method(sel!(windowDidEndLiveResize:),
			window_did_resize as extern fn(&Object, Sel, id));

		decl.add_method(sel!(windowDidBecomeKey:),
			window_did_become_key as extern fn(&Object, Sel, id));

		decl.add_method(sel!(windowDidResignKey:),
			window_did_resign_key as extern fn(&Object, Sel, id));

		decl.register()
	}

	pub unsafe fn new(manager: Sender<Event>, window: IdRef, view: IdRef) -> Delegate {
		let mut state = Box::new(State {
			manager: manager,
			window:  window.clone(),
			view:    view,
		});

		let delegate = IdRef::new(msg_send![Delegate::class(), new]);
		(&mut **delegate).set_ivar("state", &mut *state as *mut _ as *mut c_void);
		msg_send![*window, setDelegate:*delegate];

		Delegate {
			state: state,
			this:  delegate
		}
	}
}

impl Drop for Delegate {
	fn drop(&mut self) {
		unsafe {
			msg_send![*self.state.window, setDelegate:nil];
		}
	}
}
