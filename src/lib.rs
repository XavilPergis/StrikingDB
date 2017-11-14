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

//! StrikingDB, named after the illustrious [Rob Stolarz](https://github.com/robstolarz),
//! is a persistent key/value store that is specifically optimized for solid state devices
//! and flash technology, thought it can be used as in-memory only as well. Additionally,
//! it is intended to be used in concurrent and multi-threaded applications.
//!
//! It is mostly ACID-compliant, supporting basic CRUD methods for interfacing with the
//! datastore.

#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

extern crate capnp;

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate lazy_static;
extern crate lru_time_cache;
extern crate num_cpus;
extern crate parking_lot;

#[macro_use]
extern crate rental;

#[macro_use]
extern crate scopeguard;

/* Platform-specific dependencies */
cfg_if! {
    if #[cfg(unix)] {
        #[macro_use]
        extern crate nix;
    } else if #[cfg(windows)] {
        extern crate winapi;
    }
}

/* Generated sources */
mod build {
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

mod serial_capnp {
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/serial_capnp.rs"));
}

/* Private fields */
mod buffer;
mod cache;
mod deleted;
mod device;
mod error;
mod index;
mod options;
mod serial;
mod stats;
mod store;
mod strand;
mod utils;
mod volume;

type FilePointer = u64;

/* Reexports */

pub use error::{Error, Result};
pub use options::{OpenMode, OpenOptions};
pub use stats::Stats;
pub use store::Store;

/// The version of this crate, as a string.
pub const VERSION_STR: &'static str = build::PKG_VERSION;

/// The maximum size for a valid key (128 KiB). Also note that valid
/// keys may not have a length of zero.
pub const MAX_KEY_LEN: usize = 128 * 1024 * 1024; /* 128 KiB */

/// The maximum size of a valid value (512 MiB).
pub const MAX_VAL_LEN: usize = 512 * 1024 * 1024 * 1024; /* 512 MiB */

/// The minimum number of strands that a datastore can be created with.
pub const MIN_STRANDS: u16 = 2;

const PAGE_SIZE: usize = 4 * 1024;
const TRIM_SIZE: usize = 256 * 1024;

const PAGE_SIZE64: u64 = PAGE_SIZE as u64;
const TRIM_SIZE64: u64 = TRIM_SIZE as u64;

lazy_static! {
    /// A lazily-initialized struct that contains a 3-tuple, in the
    /// form `(major, minor, patch)`.
    #[derive(Debug)]
    pub static ref VERSION: (u8, u8, u8) = {
        let major = build::PKG_VERSION_MAJOR.parse::<u8>().unwrap();
        let minor = build::PKG_VERSION_MINOR.parse::<u8>().unwrap();
        let patch = build::PKG_VERSION_PATCH.parse::<u8>().unwrap();

        (major, minor, patch)
    };
}
