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

use sys::pango;
use config::Config;
use terminal::Cell;
use style;

#[derive(Debug)]
pub struct Style {
	list:  pango::AttributeList,
	style: style::Style,

	bold:   pango::Attribute,
	light:  pango::Attribute,
	normal: pango::Attribute,

	italic:  pango::Attribute,
	regular: pango::Attribute,

	underline:       pango::Attribute,
	underline_no:    pango::Attribute,
	underline_color: pango::Attribute,

	strikethrough:       pango::Attribute,
	strikethrough_no:    pango::Attribute,
	strikethrough_color: pango::Attribute,
}

impl Style {
	pub fn new(config: &Config) -> Self {
		Style {
			list:  pango::AttributeList::new(),
			style: Default::default(),

			bold:   pango::Attribute::weight(pango::Weight::Bold),
			light:  pango::Attribute::weight(pango::Weight::Light),
			normal: pango::Attribute::weight(pango::Weight::Normal),

			italic:  pango::Attribute::style(pango::Style::Italic),
			regular: pango::Attribute::style(pango::Style::Normal),

			underline:       pango::Attribute::underline(pango::Underline::Single),
			underline_no:    pango::Attribute::underline(pango::Underline::None),
			underline_color: pango::Attribute::underline_color(
				config.style().color().underline().unwrap_or_else(||
				config.style().color().foreground())),

			strikethrough:       pango::Attribute::strikethrough(true),
			strikethrough_no:    pango::Attribute::strikethrough(false),
			strikethrough_color: pango::Attribute::strikethrough_color(
				config.style().color().strikethrough().unwrap_or_else(||
				config.style().color().foreground())),
		}
	}

	pub fn update(&mut self, cell: &Cell) {
		if cell.style().attributes() == self.style.attributes() {
			return;
		}

		// Set font weight.
		if cell.style().attributes().contains(style::BOLD) {
			self.list.change(self.bold.clone());
		}
		else if cell.style().attributes().contains(style::FAINT) {
			self.list.change(self.light.clone());
		}
		else {
			self.list.change(self.normal.clone());
		}

		// Set font style.
		if cell.style().attributes().contains(style::ITALIC) {
			self.list.change(self.italic.clone());
		}
		else {
			self.list.change(self.regular.clone());
		}

		// Set underline.
		if cell.style().attributes().contains(style::UNDERLINE) {
			self.list.change(self.underline.clone());
			self.list.change(self.underline_color.clone());
		}
		else {
			self.list.change(self.underline_no.clone());
		}

		// Set strikethrough.
		if cell.style().attributes().contains(style::STRUCK) {
			self.list.change(self.strikethrough.clone());
			self.list.change(self.strikethrough_color.clone());
		}
		else {
			self.list.change(self.strikethrough_no.clone());
		}

		self.style = cell.style().clone();
	}
}

impl Deref for Style {
	type Target = pango::AttributeList;

	fn deref(&self) -> &Self::Target {
		&self.list
	}
}
