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
mod strand;
mod item;

pub use self::header::Header;
pub use self::strand::StrandHeader;

use std::{mem, ptr, slice};
use super::*;

pub unsafe trait Pod: Sized {
    fn validate(&self) -> bool;

    fn as_bytes<'a>(&'a self) -> &'a [u8] {
        let ptr: *const Self = self;
        unsafe {
            slice::from_raw_parts(ptr as *const u8, mem::size_of::<Self>())
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        assert_eq!(bytes.len(), mem::size_of::<Self>());
        let src_ptr = bytes.as_ptr();
        let dest = unsafe {
            let mut dest = mem::uninitialized::<Self>();
            let dest_ptr: *mut Self = &mut dest;
            ptr::copy_nonoverlapping(src_ptr as *const u8, dest_ptr as *mut u8, 1);
            dest
        };
        match dest.validate() {
            true => Ok(dest),
            false => Err(error::SError::Corruption),
        }
    }
}
