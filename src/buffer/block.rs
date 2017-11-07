/*
 * buffer/block.rs
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

use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use super::{TRIM_SIZE, ByteArray};

pub struct Block([u8; TRIM_SIZE]);

// FIXME: We can derive clone once const generics land
impl Clone for Block {
   fn clone(&self) -> Self {
        let mut copy: [u8; TRIM_SIZE] = unsafe { ::std::mem::uninitialized() };
        {
            let mut dest = &mut copy[..];
            dest.copy_from_slice(&self[..]);
        }
        Block(copy)
    }
}

impl Default for Block {
    fn default() -> Self {
        Block([0; TRIM_SIZE])
    }
}

impl Deref for Block {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl DerefMut for Block {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl AsRef<[u8]> for Block {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.0[..]);
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Block: {:?}..", &self.0[..16])
    }
}

impl ByteArray for Block {}
