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
use std::sync::mpsc::Sender;
use std::ffi::CStr;
use std::str;
use std::cell::RefCell;

use objc::runtime::{Object, YES, NO};
use cocoa::base::nil;
use cocoa::foundation::{NSAutoreleasePool, NSDate, NSDefaultRunLoopMode, NSPoint, NSRect, NSSize};
use cocoa::foundation::{NSString, NSUInteger};
use cocoa::appkit::{self, NSApplication, NSApplicationActivationPolicy, NSEvent, NSWindow, NSView};

use error;
use config::Config;
use font::Font;
use platform::Event;
use platform::macos::{Proxy, IdRef, Delegate};
use platform::key::{self, Key};
use platform::mouse::{self, Mouse};

#[derive(Debug)]
pub struct Window {
	config: Arc<Config>,
	window: IdRef,
	view:   IdRef,
	proxy:  Option<Proxy>,
}

impl Window {
	pub fn new(_name: Option<&str>, config: Arc<Config>, font: Arc<Font>) -> error::Result<Self> {
		let margin  = config.style().margin();
		let spacing = config.style().spacing();

		let width  = (80 * font.width()) + (margin * 2);
		let height = (24 * (font.height() + spacing)) + (margin * 2);

		unsafe {
			let app = appkit::NSApp();
			app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);
			app.finishLaunching();

			let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width as f64, height as f64));

			let mask = appkit::NSClosableWindowMask as NSUInteger
				| appkit::NSMiniaturizableWindowMask as NSUInteger
				| appkit::NSResizableWindowMask as NSUInteger
				| appkit::NSTitledWindowMask as NSUInteger;

			let window = IdRef::new(NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
				frame, mask, appkit::NSBackingStoreBuffered, NO)).non_nil().unwrap();

			window.setReleasedWhenClosed_(NO);
			window.setTitle_(*IdRef::new(NSString::alloc(nil).init_str("cancer")));
			window.setAcceptsMouseMovedEvents_(YES);
			window.center();

			let view = IdRef::new(NSView::alloc(nil).init()).non_nil().unwrap();
			window.setContentView_(*view);

			app.activateIgnoringOtherApps_(YES);
			window.makeKeyAndOrderFront_(nil);

			let proxy = Proxy {
				config:  config.clone(),
				manager: None,

				window:  window.clone(),
				view:    view.clone(),
				context: RefCell::new(IdRef::new(nil)),
			};

			Ok(Window {
				config: config,
				window: window,
				view:   view,
				proxy:  Some(proxy),
			})
		}
	}

	pub fn proxy(&mut self) -> Proxy {
		self.proxy.take().unwrap()
	}

	pub fn run(&mut self, manager: Sender<Event>) -> error::Result<()> {
		unsafe fn position(window: &IdRef, view: &IdRef, event: *mut Object) -> mouse::Position {
			let point  = event.locationInWindow();
			let point  = view.convertPoint_fromView_(point, nil);
			let rect   = NSView::frame(**view);
			let factor = window.backingScaleFactor() as f32;

			mouse::Position {
				x: (factor * point.x as f32) as u32,
				y: (factor * (rect.size.height - point.y) as f32) as u32,
			}
		}

		unsafe fn key(event: *mut Object, modifier: key::Modifier, lock: key::Lock) -> Option<Key> {
			use platform::key::{Button, Keypad};

			Some(Key::new(match event.keyCode() {
				0x24 => Button::Enter.into(),
				0x30 => Button::Tab.into(),
				0x33 => Button::Backspace.into(),
				0x35 => Button::Escape.into(),

				0x7b => Button::Left.into(),
				0x7c => Button::Right.into(),
				0x7d => Button::Down.into(),
				0x7e => Button::Up.into(),

				0x60 => Button::F(5).into(),
				0x61 => Button::F(6).into(),
				0x62 => Button::F(7).into(),
				0x63 => Button::F(3).into(),
				0x64 => Button::F(8).into(),
				0x65 => Button::F(9).into(),
				0x67 => Button::F(11).into(),
				0x69 => Button::F(13).into(),
				0x6b => Button::F(14).into(),
				0x6d => Button::F(10).into(),
				0x6f => Button::F(12).into(),
				0x71 => Button::F(15).into(),
				0x76 => Button::F(4).into(),
				0x78 => Button::F(2).into(),
				0x7a => Button::F(1).into(),

				0x72 => Button::Insert.into(),
				0x73 => Button::Home.into(),
				0x74 => Button::PageUp.into(),
				0x75 => Button::Delete.into(),
				0x77 => Button::End.into(),
				0x79 => Button::PageDown.into(),

				0x52 => Keypad::Number(0).into(),
				0x53 => Keypad::Number(1).into(),
				0x54 => Keypad::Number(2).into(),
				0x55 => Keypad::Number(3).into(),
				0x56 => Keypad::Number(4).into(),
				0x57 => Keypad::Number(5).into(),
				0x58 => Keypad::Number(6).into(),
				0x59 => Keypad::Number(7).into(),
				0x5b => Keypad::Number(8).into(),
				0x5c => Keypad::Number(9).into(),

				0x41 => Keypad::Decimal.into(),
				0x43 => Keypad::Multiply.into(),
				0x45 => Keypad::Add.into(),
				0x4b => Keypad::Divide.into(),
				0x4c => Keypad::Enter.into(),
				0x4e => Keypad::Subtract.into(),

				_ => {
					let string = event.characters().UTF8String();
					let string = CStr::from_ptr(string);
					let string = str::from_utf8_unchecked(string.to_bytes());

					if string.is_empty() {
						return None;
					}

					string.to_owned().into()
				}
			}, modifier, lock))
		}

		unsafe {
			#[allow(unused_variables)]
			let delegate = Delegate::new(manager.clone(), self.window.clone(), self.view.clone());

			let mut modifier = key::Modifier::empty();
			let mut lock     = key::Lock::empty();
			let mut first    = true;

			loop {
				let pool  = NSAutoreleasePool::new(nil);
				let event = appkit::NSApp().nextEventMatchingMask_untilDate_inMode_dequeue_(
					appkit::NSAnyEventMask.bits() | appkit::NSEventMaskPressure.bits(),
					NSDate::distantFuture(nil), NSDefaultRunLoopMode, YES);

				if event == nil {
					continue;
				}

				// Only forward the event if it's not a KeyDown one, this is because
				// Cocoa is shit and it will beep if it thinks you didn't handle key
				// input.
				if event.eventType() != appkit::NSKeyDown {
					appkit::NSApp().sendEvent_(event);
				}

				match event.eventType() {
					// It doesn't render properly if it isn't redrawn after the first
					// event is received, so hack around it.
					appkit::NSAppKitDefined => {
						if first {
							first = false;
							try!(manager.send(Event::Redraw));
						}
					}

					// Key modifiers.
					appkit::NSFlagsChanged => {
						let flags = event.modifierFlags();

						if flags.contains(appkit::NSShiftKeyMask) {
							modifier.insert(key::SHIFT);
						}
						else {
							modifier.remove(key::SHIFT);
						}

						if flags.contains(appkit::NSControlKeyMask) {
							modifier.insert(key::CTRL);
						}
						else {
							modifier.remove(key::CTRL);
						}

						if flags.contains(appkit::NSCommandKeyMask) {
							modifier.insert(key::LOGO);
						}
						else {
							modifier.remove(key::LOGO);
						}

						if flags.contains(appkit::NSAlternateKeyMask) {
							modifier.insert(key::ALT);
						}
						else {
							modifier.remove(key::ALT);
						}

						if flags.contains(appkit::NSAlphaShiftKeyMask) {
							lock.insert(key::CAPS);
						}
						else {
							lock.remove(key::CAPS);
						}
					}

					// Handle key input.
					appkit::NSKeyDown => {
						if let Some(key) = key(event, modifier, lock) {
							try!(manager.send(Event::Key(key)));
						}
					}

					// Handle mouse clicks.
					appkit::NSLeftMouseDown |
					appkit::NSRightMouseDown |
					appkit::NSOtherMouseDown |
					appkit::NSLeftMouseUp |
					appkit::NSRightMouseUp |
					appkit::NSOtherMouseUp => {
						let press = event.eventType() == appkit::NSLeftMouseDown ||
						            event.eventType() == appkit::NSRightMouseDown ||
						            event.eventType() == appkit::NSOtherMouseDown;

						let button = match event.buttonNumber() {
							0 => Some(mouse::Button::Left),
							1 => Some(mouse::Button::Right),
							2 => Some(mouse::Button::Middle),
							_ => None
						};

						if let Some(button) = button {
							try!(manager.send(Event::Mouse(Mouse::Click(mouse::Click {
								press:    press,
								button:   button,
								modifier: modifier,
								position: position(&self.window, &self.view, event),
							}))));
						}
					}

					// Handle mouse motion.
					appkit::NSMouseMoved |
					appkit::NSLeftMouseDragged |
					appkit::NSRightMouseDragged |
					appkit::NSOtherMouseDragged => {
						try!(manager.send(Event::Mouse(Mouse::Motion(mouse::Motion {
							modifier: modifier,
							position: position(&self.window, &self.view, event),
						}))));
					}

					appkit::NSScrollWheel => {
						let button = if event.scrollingDeltaY().is_sign_positive() {
							mouse::Button::Up
						}
						else {
							mouse::Button::Down
						};

						try!(manager.send(Event::Mouse(Mouse::Click(mouse::Click {
							press:    true,
							button:   button,
							modifier: modifier,
							position: position(&self.window, &self.view, event),
						}))));

						try!(manager.send(Event::Mouse(Mouse::Click(mouse::Click {
							press:    false,
							button:   button,
							modifier: modifier,
							position: position(&self.window, &self.view, event),
						}))));
					}

					event => {
						debug!(target: "cancer::platform", "unhandled event: {:?}", event);
					}
				}

				msg_send![pool, release];
			}
		}
	}
}
