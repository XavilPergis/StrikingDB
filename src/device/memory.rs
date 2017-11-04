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

use parking_lot::RwLock;
use super::{Device, Result};
use super::{check_read, check_write, check_trim};

#[derive(Debug)]
pub struct Memory(RwLock<Box<[u8]>>, u64);

impl Memory {
    pub fn new(bytes: usize) -> Self {
        let buffer = vec![0; bytes].into_boxed_slice();
        Memory(RwLock::new(buffer), bytes as u64)
    }
}

impl Device for Memory {
    #[inline]
    fn capacity(&self) -> u64 {
        self.1
    }

    #[inline]
    fn block_device(&self) -> bool {
        false
    }

    fn read(&self, off: u64, buf: &mut [u8]) -> Result<()> {
        check_read(self, off, buf);

        let off = off as usize;
        let end = off + buf.len();

        let guard = self.0.read();
        let buffer = &*guard;
        let src = &buffer[off..end];
        buf.copy_from_slice(src);

        Ok(())
    }

    fn write(&self, off: u64, buf: &[u8]) -> Result<()> {
        check_write(self, off, buf);

        let off = off as usize;
        let end = off + buf.len();

        let mut guard = self.0.write();
        let buffer = &mut *guard;
        let dest = &mut buffer[off..end];
        dest.copy_from_slice(buf);

        Ok(())
    }

    fn trim(&self, off: u64, len: u64) -> Result<()> {
        check_trim(self, off, len);

        // A trim "erases" the given device's region,
        // and any reads from this region will yield
        // undefined values.
        //
        // As such, it doesn't matter what we do to this
        // region, since the user shouldn't be reading
        // from this section anyways. Thus, we do nothing.

        Ok(())
    }
}
