/*
 * lib.rs
 *
 * striking-db - Persistent key/value store for SSDs.
 * Copyright (c) 2017 Maxwell Duzen, Ammon Smith
 *
 * striking-db is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by the Free Software Foundation, either version 2 of
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
extern crate stable_deref_trait;

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
pub use store::Store;
pub use options::{OpenMode, OpenOptions};

/* Constants */
pub const VERSION_STR: &'static str = build::PKG_VERSION;

pub const MAX_KEY_LEN: usize = 128 * 1024 * 1024; /* 128 KiB */
pub const MAX_VAL_LEN: usize = 512 * 1024 * 1024 * 1024; /* 512 MiB */

pub const MIN_STRANDS: u16 = 2;

const PAGE_SIZE: usize = 4 * 1024;
const TRIM_SIZE: usize = 256 * 1024;

const PAGE_SIZE64: u64 = PAGE_SIZE as u64;
const TRIM_SIZE64: u64 = TRIM_SIZE as u64;

lazy_static! {
    pub static ref VERSION: (u8, u8, u8) = {
        let major = build::PKG_VERSION_MAJOR.parse::<u8>().unwrap();
        let minor = build::PKG_VERSION_MINOR.parse::<u8>().unwrap();
        let patch = build::PKG_VERSION_PATCH.parse::<u8>().unwrap();

        (major, minor, patch)
    };
}
