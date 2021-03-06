/*
 * buffer/mod.rs
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

mod block;
mod buffer;
mod page;
mod traits;


pub use self::block::Block;
pub use self::buffer::Buffer;
pub use self::page::Page;
pub use self::traits::ByteArray;
use super::{PAGE_SIZE, TRIM_SIZE};

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum BufferStatus {
    Clean,
    Dirty,
    Empty,
}
