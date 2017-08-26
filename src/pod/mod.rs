/*
 * pod/mod.rs
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

mod header;

pub use self::header::Header;

pub unsafe trait Pod {}

use std::{mem, ptr, slice};
use super::*;

fn as_bytes<T: Pod>(src: &T) -> &[u8] {
    let ptr: *const T = src;
    unsafe {
        slice::from_raw_parts(ptr as *const u8, mem::size_of::<T>())
    }
}

fn from_bytes<T: Pod>(src: &[u8]) -> T {
    assert_eq!(src.len(), mem::size_of::<T>());
    let src = src.as_ptr();

    unsafe {
        let mut dest = mem::uninitialized::<T>();
        let dest_ptr: *mut T = &mut dest;
        ptr::copy_nonoverlapping(
            src as *const u8,
            dest_ptr as *mut u8,
            1);
        dest
    }
}