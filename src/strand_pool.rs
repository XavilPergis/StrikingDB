/*
 * strand_pool.rs
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
use num_cpus;
use options::{OpenMode, OpenOptions};
use strand::Strand;
use std::cmp::{self, Ordering};
use std::time::Duration;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use super::{PAGE_SIZE, FilePointer, Result};
use utils::align;

#[derive(Debug)]
pub struct StrandPool {
    dev: Box<Device>,
    strands: Box<[RwLock<Strand>]>,
}

impl StrandPool {
    pub fn new(dev: Device, options: &OpenOptions) -> Result<Self> {
        let count = options.strands;

        // TODO handle options

        const GB: u64 = 1024 * 1024 * 1024;

        let count = match count {
            Some(x) => x as u64,
            None => {
                let cores = num_cpus::get() as u64;
                8 * cores * dev.capacity() / GB
            }
        };
        assert_ne!(count, 0, "Strand count must be nonzero");
        let mut strands = Vec::with_capacity(count as usize);

        let size = align(dev.capacity() / count);

        // Allocate strands
        // The first page is reserved for metadata
        let mut left = dev.capacity();
        for i in 0..count {
            let off = i * size + PAGE_SIZE;
            let len = cmp::min(size, left);
            debug_assert_ne!(len, 0, "Length of strand must be nonzero");

            left -= len;
            let strand = Strand::new(&dev, i, off, len, options.mode == OpenMode::Open)?;
            let lock = RwLock::new(strand);
            strands.push(lock);
        }
        debug_assert_eq!(left, 0, "Not all space is allocated in a strand");

        Ok(StrandPool {
            dev: dev,
            strands: strands.into_boxed_slice(),
        })
    }

    pub fn read(&self, ptr: FilePointer) -> RwLockReadGuard<Strand> {
        // Search for the strand that has this file pointer
        let result = self.strands.binary_search_by(|strand| {
            let guard = strand.read();
            if ptr < guard.start() {
                Ordering::Less
            } else if ptr < guard.start() + guard.capacity() {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        });

        match result {
            Ok(idx) => self.strands[idx].read(),
            Err(_) => panic!("File pointer {:x} is not in any strand", ptr),
        }
    }

    pub fn write(&self) -> RwLockWriteGuard<Strand> {
        // Look for the first strand that is available for writing
        let delay = Duration::new(0, 100 * 1000);
        loop {
            for ref strand in &*self.strands {
                if let Some(guard) = strand.try_write_for(delay) {
                    return guard;
                }
            }
        }
    }
}
