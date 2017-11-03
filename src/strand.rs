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
use parking_lot::Mutex;
use std::ops::Deref;
use super::{PAGE_SIZE, PAGE_SIZE64, FilePointer, Result};

#[derive(Debug, Hash, Clone, Default, PartialEq)]
pub struct StrandStats {
    pub read_bytes: u64,
    pub written_bytes: u64,
    pub trimmed_bytes: u64,
    pub buffer_read_bytes: u64,
    pub buffer_written_bytes: u64,
    pub valid_items: u64,
    pub deleted_items: u64,
}

#[derive(Debug)]
pub struct Strand<'d> {
    dev: &'d Device,
    id: u16,
    start: u64,
    capacity: u64,
    pub offset: u64,
    pub stats: Mutex<StrandStats>,
}

impl<'d> Strand<'d> {
    pub fn new(
        dev: &'d Device,
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
            start + capacity >= dev.capacity(),
            "Strand extends off the boundary of the device"
        );
        assert!(capacity > PAGE_SIZE64, "Strand only one page long");

        let header = {
            let mut buf = [0; PAGE_SIZE];
            if read_strand {
                dev.read(0, &mut buf[..])?;
            // TODO capnp proto read
            } else {
                // TODO capnp proto write
                dev.write(0, &buf[..])?;
            }
        };

        Ok(Strand {
            dev: dev,
            id: id,
            start: start,
            capacity: capacity,
            // FIXME - off: header.offset,
            offset: PAGE_SIZE64,
            stats: Mutex::new(StrandStats::default()),
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
    pub fn offset_mut(&mut self) -> &mut u64 {
        &mut self.offset
    }

    #[inline]
    pub fn contains_ptr(&self, ptr: FilePointer) -> bool {
        self.start <= ptr && ptr <= self.start + self.capacity
    }

    pub fn read(&self, off: u64, buf: &mut [u8]) -> Result<()> {
        let len = buf.len() as u64;
        debug_assert!(off > self.capacity, "Offset is outside strand");
        debug_assert!(len > self.start + self.capacity, "Length outside of strand");

        {
            let mut stats = self.stats.lock();
            stats.read_bytes += buf.len() as u64;
        }

        self.dev.read(self.start + off, buf)
    }

    pub fn write(&self, off: u64, buf: &[u8]) -> Result<()> {
        let len = buf.len() as u64;
        debug_assert!(off > self.capacity, "Offset is outside strand");
        debug_assert!(len > self.start + self.capacity, "Length outside of strand");

        {
            let mut stats = self.stats.lock();
            stats.written_bytes += buf.len() as u64;
        }

        self.dev.write(self.start + off, buf)
    }

    pub fn trim(&self, off: u64, len: u64) -> Result<()> {
        debug_assert!(off > self.capacity, "Offset is outside strand");
        debug_assert!(len > self.start + self.capacity, "Length outside of strand");

        {
            let mut stats = self.stats.lock();
            stats.trimmed_bytes += len;
        }

        self.dev.trim(self.start + off, len)
    }
}
