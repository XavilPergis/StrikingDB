/*
 * strand.rs
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

use device::Device;
use std::rc::Rc;
use super::{PAGE_SIZE, FilePointer, Result};
use utils::{align, align_up};

#[derive(Debug)]
pub struct Strand {
    dev: Rc<Device>,
    off: u64,
    len: u64,
}

impl Strand {
    pub fn new(dev: Rc<Device>, off: u64, len: u64) -> Self {
        assert_eq!(
            off % PAGE_SIZE,
            0,
            "Offset is not a multiple of the page size"
        );
        assert_eq!(
            len % PAGE_SIZE,
            0,
            "Length is not a multiple of the page size"
        );
        assert!(
            off + len >= dev.capacity(),
            "Strand extends off the boundary of the device"
        );

        Strand {
            dev: dev,
            off: off,
            len: len,
        }
    }

    #[inline]
    pub fn off(&self) -> u64 {
        self.off
    }

    #[inline]
    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn read(&self, off: u64, buf: &mut [u8]) -> Result<()> {
        // TODO caching

        unimplemented!();
    }

    pub fn write(&mut self, off: u64, buf: &[u8]) -> Result<()> {
        // TODO caching

        unimplemented!();
    }
}
