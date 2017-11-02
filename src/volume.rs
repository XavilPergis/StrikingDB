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
use parking_lot::RwLock;
use std::cmp::{self, Ordering};
use std::time::Duration;
use std::u16;
use strand::Strand;
use super::{PAGE_SIZE, PAGE_SIZE64, FilePointer, Result};
use utils::align;

#[derive(Debug)]
struct VolumeOpen {
    strand_count: u16,
    read_strand: bool,
}

impl VolumeOpen {
    fn new(dev: &Device, options: &OpenOptions) -> Self {
        const GB: u64 = 1024 * 1024 * 1024;

        let count = match options.strands {
            Some(x) => x as u64,
            None => {
                let cores = num_cpus::get() as u64;
                8 * cores * dev.capacity() / GB
            }
        };
        assert_ne!(count, 0, "Strand count must be nonzero");
        assert!(count <= u16::MAX as u64, "Integer not large enough for all these strands");

        VolumeOpen {
            strand_count: count as u16,
            read_strand: false,
        }
    }

    fn read(dev: &Device, options: &OpenOptions) -> Result<Self> {
        let mut buf = [0; PAGE_SIZE];
        dev.read(0, &mut buf[..])?;

        // TODO read meta block
        unimplemented!();
    }
}

rental! {
    mod rentals {
        use super::*;

        #[rental(debug_borrow)]
        pub struct VolumeRental {
            dev: Box<Device>,
            strands: Box<[RwLock<Strand<'dev>>]>,
        }
    }
}

use self::rentals::VolumeRental;

#[derive(Debug)]
pub struct Volume(VolumeRental);

impl Volume {
    pub fn open(dev: Device, options: &OpenOptions) -> Result<Self> {
        use rental::TryNewError;

        let rental = VolumeRental::try_new(Box::new(dev), |dev| {
            let open = match options.mode {
                OpenMode::Open => VolumeOpen::read(&dev, options)?,
                OpenMode::Create | OpenMode::Truncate => VolumeOpen::new(&dev, options),
            };

            if options.mode == OpenMode::Truncate {
                dev.trim(0, dev.capacity())?;
            }

            let mut left = dev.capacity();
            let size = align(dev.capacity() / open.strand_count as u64);

            let mut strands = Vec::with_capacity(open.strand_count as usize);
            for i in 0..open.strand_count {
                // The first page is reserved for metadata
                let off = (i as u64) * size + PAGE_SIZE64;
                let len = cmp::min(size, left);
                debug_assert_eq!(off % PAGE_SIZE64, 0, "Strand offset is not page-aligned");
                debug_assert_eq!(len % PAGE_SIZE64, 0, "Strand length is not page-aligned");
                debug_assert_ne!(len, 0, "Length of strand must be nonzero");

                left -= len;
                let strand = Strand::new(&dev, i, off, len, open.read_strand)?;
                let lock = RwLock::new(strand);
                strands.push(lock);
            }
            debug_assert_eq!(left, 0, "Not all space is allocated in a strand");

            Ok(strands.into_boxed_slice())
        });

        match rental {
            Ok(rental) => Ok(Volume(rental)),
            Err(TryNewError(err, _)) => Err(err),
        }
    }

    pub fn read<F, R>(&self, ptr: FilePointer, func: F) -> R
        where F: FnOnce(&Strand) -> R
    {
        self.0.rent(|strands| {
            // Search for the strand that has this file pointer
            let result = strands.binary_search_by(|strand| {
                let guard = strand.read();
                if ptr < guard.start() {
                    Ordering::Less
                } else if ptr < guard.end() {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            });

            let guard = match result {
                Ok(idx) => strands[idx].read(),
                Err(_) => panic!("File pointer 0x{:x} is not in any strand", ptr),
            };

            func(&*guard)
        })
    }

    pub fn write<F, R>(&self, func: F) -> R
        where F: FnOnce(&mut Strand) -> R
    {
        let delay = Duration::new(0, 100 * 1000);

        self.0.rent(|strands| {
            // Look for the first strand that is available for writing
            loop {
                for ref strand in strands.iter() {
                    if let Some(mut guard) = strand.try_write_for(delay) {
                        return func(&mut *guard);
                    }
                }
            }
        })
    }
}
