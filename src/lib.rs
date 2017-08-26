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

#[macro_use]
extern crate cfg_if;
extern crate num_cpus;

#[macro_use]
extern crate lazy_static;
extern crate parking_lot;

#[cfg(unix)]
#[macro_use]
extern crate nix;

mod device;
mod error;
mod index;
mod pod;
mod store;
mod strand;
mod strand_pool;
mod utils;

pub use error::SError as Error;
pub use error::SResult as Result;
pub use store::Store;

const PAGE_SIZE: u64 = 4096;
pub const MAX_KEY_LEN: usize = 512;
pub const MAX_VAL_LEN: usize = 65535;

type FilePointer = u64;
