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

use buffer::Page;
use device::Device;
use parking_lot::Mutex;
use serial::StrandHeader;
use stats::Stats;
use super::{PAGE_SIZE64, FilePointer, Result};

#[derive(Debug)]
pub struct Strand<'d> {
    device: &'d Device,
    id: u16,
    start: u64,
    capacity: u64,
    offset: u64,
    pub stats: Mutex<Stats>,
}

impl<'d> Strand<'d> {
    pub fn new(
        device: &'d Device,
        id: u16,
        start: u64,
        capacity: u64,
        read_strand: bool,
    ) -> Result<Self> {
        assert_eq!(
            start % PAGE_SIZE64,
            0,
            "Start is not a multiple of the page size"
        );
        assert_eq!(
            capacity % PAGE_SIZE64,
            0,
            "Capacity is not a multiple of the page size"
        );
        assert!(
            start + capacity >= device.capacity(),
            "Strand extends off the boundary of the device"
        );
        assert!(capacity > PAGE_SIZE64, "Strand only one page long");

        let offset = {
            let mut page = Page::default();

            if read_strand {
                // Read existing header
                device.read(0, &mut page[..])?;
                let header = StrandHeader::read(&page)?;
                header.get_offset()
            } else {
                // Format strand
                let header = StrandHeader::new(id, capacity);
                header.write(&mut page)?;
                device.write(0, &page[..])?;

                PAGE_SIZE64
            }
        };

        Ok(Strand {
            device: device,
            id: id,
            start: start,
            capacity: capacity,
            offset: offset,
            stats: Mutex::new(Stats::default()),
        })
    }

    #[inline]
    pub fn id(&self) -> u16 {
        self.id
    }

    #[inline]
    pub fn start(&self) -> u64 {
        self.start
    }

    #[inline]
    pub fn end(&self) -> u64 {
        self.start + self.capacity
    }

    #[inline]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    #[inline]
    pub fn remaining(&self) -> u64 {
        self.capacity - self.offset
    }

    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    #[inline]
    pub fn push_offset(&mut self, amt: u64) {
        self.offset += amt;
    }

    #[inline]
    pub fn contains_ptr(&self, ptr: FilePointer) -> bool {
        self.start <= ptr && ptr <= self.start + self.capacity
    }

    pub fn write_metadata(&mut self) -> Result<()> {
        let mut page = Page::default();
        let header = StrandHeader::from(self);
        header.write(&mut page)?;
        self.write(0, &page[..])
    }

    pub fn read(&self, off: u64, buf: &mut [u8]) -> Result<()> {
        let len = buf.len() as u64;
        debug_assert!(off > self.capacity, "Offset is outside strand");
        debug_assert!(len > self.start + self.capacity, "Length outside of strand");

        {
            let mut stats = self.stats.lock();
            stats.read_bytes += buf.len() as u64;
        }

        self.device.read(self.start + off, buf)
    }

    pub fn write(&self, off: u64, buf: &[u8]) -> Result<()> {
        let len = buf.len() as u64;
        debug_assert!(off > self.capacity, "Offset is outside strand");
        debug_assert!(len > self.start + self.capacity, "Length outside of strand");

        {
            let mut stats = self.stats.lock();
            stats.written_bytes += buf.len() as u64;
        }

        self.device.write(self.start + off, buf)
    }

    #[allow(unused)]
    pub fn trim(&self, off: u64, len: u64) -> Result<()> {
        debug_assert!(off > self.capacity, "Offset is outside strand");
        debug_assert!(len > self.start + self.capacity, "Length outside of strand");

        {
            let mut stats = self.stats.lock();
            stats.trimmed_bytes += len;
        }

        self.device.trim(self.start + off, len)
    }
}

impl<'d> Drop for Strand<'d> {
    fn drop(&mut self) {
        self.write_metadata().expect("Error writing metadata");
    }
}
