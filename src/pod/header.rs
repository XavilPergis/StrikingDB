/*
 * pod/header.rs
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

const SIGNATURE: u64 = 0x864d26e37a418b16;
const PAD: u8 = 0x12;

lazy_static! {
    static ref MAJOR: u8 = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap();
    static ref MINOR: u8 = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap();
    static ref PATCH: u8 = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap();
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(C, packed)]
pub struct Header {
    pub signature: u64,
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    _pad: u8,
    pub strands: u32,
}

impl Header {
    pub fn new(strands: u32) -> Self {
        assert_ne!(strands, 0);

        Header {
            signature: SIGNATURE,
            major: *MAJOR,
            minor: *MINOR,
            patch: *PATCH,
            _pad: PAD,
            strands: strands,
        }
    }
}

unsafe impl Pod for Header {
    fn validate(&self) -> bool {
        self.signature == SIGNATURE && self.major == *MAJOR && self.minor == *MINOR &&
            self.patch == *PATCH && self._pad == PAD && self.strands > 0
    }
}
