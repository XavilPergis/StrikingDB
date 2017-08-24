/*
 * lib.rs
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

#[cfg(unix)]
#[macro_use]
extern crate nix;

mod error;
mod index;
mod store;
mod strand;

#[cfg(unix)]
mod unix_device;

pub use error::SError as Error;
pub use error::SResult as Result;
pub use store::Store;

const PAGE_SIZE: u64 = 4096;
const MAX_KEY_LEN: usize = 512;
const MAX_VAL_LEN: usize = 65535;

type FilePointer = u64;
