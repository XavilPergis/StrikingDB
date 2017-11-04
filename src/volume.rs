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

use buffer::Page;
use deleted::Deleted;
use device::Device;
use error::Error;
use index::Index;
use num_cpus;
use options::{OpenMode, OpenOptions};
use parking_lot::RwLock;
use serial::{DatastoreState, VolumeHeader};
use stats::Stats;
use std::cmp::{Ordering, min};
use std::time::Duration;
use std::u16;
use strand::Strand;
use super::{MIN_STRANDS, PAGE_SIZE, PAGE_SIZE64, FilePointer, Result};
use utils::align;

#[derive(Debug)]
struct VolumeOpen {
    strands: u16,
    state_ptr: Option<FilePointer>,
    read_disk: bool,
}

impl VolumeOpen {
    fn new(dev: &Device, options: &OpenOptions) -> Result<Self> {
        const GB: u64 = 1024 * 1024 * 1024;

        let count = match options.strands {
            Some(x) => {
                if x <= MIN_STRANDS {
                    return Err(Error::BadArgument("Too few strands were specified."));
                }

                x as u64
            }
            None => {
                let cores = num_cpus::get() as u64;
                8 * cores * dev.capacity() / GB
            }
        };
        assert_ne!(count, 0, "Strand count must be nonzero");
        assert!(
            count <= u16::MAX as u64,
            "Integer not large enough for all these strands"
        );

        Ok(VolumeOpen {
            strands: count as u16,
            state_ptr: None,
            read_disk: false,
        })
    }

    fn read(dev: &Device, options: &OpenOptions) -> Result<Self> {
        let mut page = Page::default();
        dev.read(0, &mut page[..])?;

        let header = VolumeHeader::read(&page)?;
        Ok(VolumeOpen {
            strands: header.get_strands(),
            state_ptr: header.get_state_ptr(),
            read_disk: true,
        })
    }
}

rental! {
    mod rentals {
        use super::*;

        #[rental(debug_borrow)]
        pub struct VolumeRental {
            device: Box<Device>,
            strands: Box<[RwLock<Strand<'device>>]>,
        }
    }
}

use self::rentals::VolumeRental;

#[derive(Debug, Default)]
pub struct VolumeState(Option<(Index, Deleted)>);

impl VolumeState {
    pub fn new(index: Index, deleted: Deleted) -> Self {
        VolumeState(Some((index, deleted)))
    }

    pub fn extract(self) -> (Index, Deleted) {
        match self.0 {
            Some((idx, del)) => (idx, del),
            None => (Index::new(), Deleted::new()),
        }
    }
}

#[derive(Debug)]
pub struct Volume(VolumeRental);

impl Volume {
    pub fn open(device: Device, options: &OpenOptions) -> Result<(Self, VolumeState)> {
        use rental::TryNewError;

        let mut state_ptr = None;
        let try_rental = VolumeRental::try_new(Box::new(device), |device| {
            use OpenMode::*;

            // Collect options
            let open = match options.mode {
                Read => VolumeOpen::read(&device, options)?,
                Create | Truncate => VolumeOpen::new(&device, options)?,
            };

            if !options.reindex {
                state_ptr = open.state_ptr;
            }

            if options.mode == Truncate {
                device.trim(0, device.capacity())?;
            }

            // Divide device into strands
            let mut left = device.capacity();
            let size = align(device.capacity() / open.strands as u64);

            let mut strands = Vec::with_capacity(open.strands as usize);
            for i in 0..open.strands {
                // The first page is reserved for metadata
                let off = (i as u64) * size + PAGE_SIZE64;
                let len = min(size, left);
                debug_assert_eq!(off % PAGE_SIZE64, 0, "Strand offset is not page-aligned");
                debug_assert_eq!(len % PAGE_SIZE64, 0, "Strand length is not page-aligned");
                debug_assert_ne!(len, 0, "Length of strand must be nonzero");

                left -= len;
                let strand = Strand::new(&device, i, off, len, open.read_disk)?;
                strands.push(RwLock::new(strand));
            }
            debug_assert_eq!(left, 0, "Not all space is allocated in a strand");

            Ok(strands.into_boxed_slice())
        });

        if options.reindex {
            return Err(Error::Unimplemented);
        }

        match try_rental {
            Ok(rental) => {
                let volume = Volume(rental);
                let state = match state_ptr {
                    Some(ptr) => volume.read(ptr, |strand| DatastoreState::read(strand, ptr))?,
                    None => VolumeState::default(),
                };

                Ok((volume, state))
            }
            Err(TryNewError(err, _)) => Err(err),
        }
    }

    pub fn read<F, R>(&self, ptr: FilePointer, func: F) -> R
    where
        F: FnOnce(&Strand) -> R,
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
    where
        F: FnOnce(&mut Strand) -> R,
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

    pub fn stats(&self) -> Stats {
        let mut total_stats = Stats::default();

        self.0.rent(|strands| for ref strand in strands.iter() {
            let guard = strand.read();
            let stats = guard.stats.lock();
            total_stats += stats.clone();
        });

        total_stats
    }
}
