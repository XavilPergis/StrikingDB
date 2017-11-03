/*
 * device/memory.rs
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

use super::{Device, Result};

#[derive(Debug)]
pub struct Memory(Box<[u8]>);

impl Memory {
    pub fn new(bytes: usize) -> Self {
        let buffer = Vec::with_capacity(bytes).into_boxed_slice();
        Memory(buffer)
    }
}

impl Device for Memory {
    fn capacity(&self) -> u64 {
        unimplemented!();
    }

    fn block(&self) -> bool {
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
