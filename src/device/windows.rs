/*
 * device/windows.rs
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

// TODO

use std::fs::File;
use std::os::windows::prelude::*;
use super::{Device, Result};

#[derive(Debug)]
pub struct Ssd {
    file: File,
    capacity: u64,
    block: bool,
}

impl Ssd {
    fn get_metadata(file: &mut File) -> Result<(u64, bool)> {
        let metadata = file.metadata()?;
        unimplemented!();
    }

    pub fn open(mut file: File) -> Result<Self> {
        let (capacity, block) = Self::get_metadata()?;

        Ok(Ssd {
            file: file,
            capacity: capacity,
            block: block,
        })
    }
}

impl Device for Ssd {
    #[inline]
    fn capacity(&self) -> u64 {
        self.capacity
    }

    #[inline]
    fn block_device(&self) -> bool {
        unimplemented!();
    }

    fn read(&self, off: u64, buf: &mut [u8]) -> Result<()> {
        unimplemented!();
    }

    fn write(&self, off: u64, buf: &[u8]) -> Result<()> {
        unimplemented!();
    }

    fn trim(&self, off: u64, len: u64) -> Result<()> {
        unimplemented!();
    }
}
