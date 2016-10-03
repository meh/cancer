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

pub use ffi::pango::PangoWeight as Weight;
pub use ffi::pango::PangoStyle as Style;

mod map;
pub use self::map::Map;

mod set;
pub use self::set::Set;

mod attributes;
pub use self::attributes::Attributes;

mod metrics;
pub use self::metrics::Metrics;

mod context;
pub use self::context::Context;

mod layout;
pub use self::layout::Layout;

mod description;
pub use self::description::Description;
