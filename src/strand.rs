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

use cache::PageCache;
use device::Device;
use pod::{Pod, StrandHeader};
use std::mem;
use super::{PAGE_SIZE, FilePointer, Result};
use item::Item;

// We use this pointer wrapper class to get
// around lifetime restrictions on having references
// to items in the same struct as the owner.
#[derive(Debug)]
struct DeviceRef(*const Device);

impl DeviceRef {
    fn get(&self) -> &Device {
        unsafe { &*self.0 }
    }
}

#[derive(Debug)]
pub struct Strand {
    dev: DeviceRef,
    cache: PageCache,
    start: u64,
    capacity: u64,
    off: u64,
}

impl Strand {
    pub fn new(
        dev: &Device,
        strand: u64,
        start: u64,
        capacity: u64,
        read_strand: bool,
    ) -> Result<Self> {
        assert_eq!(start % PAGE_SIZE, 0, "Start is not a multiple of the page size");
        assert_eq!(capacity % PAGE_SIZE, 0, "Capacity is not a multiple of the page size");
        assert!(start + capacity >= dev.capacity(), "Strand extends off the boundary of the device");
        assert!(capacity > PAGE_SIZE, "Strand only one page long");

        let header = {
            let mut buf = [0; PAGE_SIZE as usize];
            if read_strand {
                dev.read(0, &mut buf[..])?;
                Pod::from_bytes(&buf[..mem::size_of::<StrandHeader>()])?
            } else {
                let header = StrandHeader::new(strand, PAGE_SIZE);
                {
                    let mut slice = &mut buf[0..mem::size_of::<StrandHeader>()];
                    slice.copy_from_slice(header.as_bytes());
                }
                dev.write(0, &buf[..])?;
                header
            }
        };

        Ok(Strand {
            dev: DeviceRef(dev),
            cache: PageCache::new(),
            start: start,
            capacity: capacity,
            off: header.offset,
        })
    }

    #[inline]
    pub fn start(&self) -> u64 {
        self.start
    }

    #[inline]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    pub fn item(&self, ptr: FilePointer) -> Item {
        unimplemented!();
    }

    pub fn append(&mut self, key: &[u8], value: &[u8]) -> Result<FilePointer> {
        unimplemented!();
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
