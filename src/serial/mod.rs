/*
 * serial/mod.rs
 *
 * striking-db - Persistent key/value store for SSDs.
 * Copyright (c) 2017 Maxwell Duzen, Ammon Smith
 *
 * striking-db is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as
 * published by the Free Software Foundation, either version 2 of
 * the License, or (at your option) any later version.
 *
 * striking-db is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with striking-db.  If not, see <http://www.gnu.org/licenses/>.
 *
 */

mod alloc;
mod fake_box;
mod header;
mod io;
mod item;
mod state;

use super::*;

pub use self::header::{StrandHeader, VolumeHeader};
pub use self::io::{StrandReader, StrandWriter};
pub use self::item::{ReadContext, read_item, write_item};
pub use self::state::DatastoreState;
