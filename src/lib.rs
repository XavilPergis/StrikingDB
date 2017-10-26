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

// FIXME: remove in final version, this is just here so
// `chargo check`ing doesn't flood the terminal with warnings
// about unused code
#![allow(dead_code)]
#![allow(unused)]

extern crate capnp;

#[macro_use]
extern crate cfg_if;
extern crate lru_time_cache;
extern crate num_cpus;
extern crate parking_lot;

#[cfg(unix)]
#[macro_use]
extern crate nix;

mod serial_capnp {
    include!(concat!(env!("OUT_DIR"), "/serial_capnp.rs"));
}

mod cache;
mod deleted;
mod device;
mod error;
mod index;
mod options;
mod store;
mod strand;
mod utils;
mod volume;

pub use error::{Error, Result};
pub use store::Store;
pub use options::{OpenMode, OpenOptions};

pub const MAX_KEY_LEN: usize = 128 * 1024 * 1024; /* 128 KiB */
pub const MAX_VAL_LEN: usize = 512 * 1024 * 1024 * 1024; /* 512 MiB */
const PAGE_SIZE: u64 = 4096;

type FilePointer = u64;
