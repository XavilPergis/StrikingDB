/*
 * pod/strand.rs
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

use super::Pod;

const SIGNATURE: u64 = 0x582f047b5ed83a7f;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(C, packed)]
pub struct StrandHeader {
    pub signature: u64,
    pub strand: u64,
    pub offset: u64,
}

impl StrandHeader {
    pub fn new(strand: u64, offset: u64) -> Self {
        StrandHeader {
            signature: SIGNATURE,
            strand: strand,
            offset: offset,
        }
    }
}

unsafe impl Pod for StrandHeader {
    fn validate(&self) -> bool {
        self.signature == SIGNATURE
    }
}
