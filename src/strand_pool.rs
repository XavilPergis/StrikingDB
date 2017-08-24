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
use strand::Strand;
use std::cmp;
use std::rc::Rc;
use super::PAGE_SIZE;

#[derive(Debug, Hash)]
pub struct StrandPool {
    dev: Rc<Device>,
    strands: Box<[Strand]>,
}

impl StrandPool {
    pub fn new(dev: Device, count: Option<usize>) -> Self {
        #[warn(non_upper_case_globals)]
        const GiB: u64 = 1024 * 1024 * 1024;

        let dev = Rc::new(dev);
        let count = match count {
            Some(x) => x as u64,
            None => {
                let cores = num_cpus::get() as u64;
                8 * cores * dev.capacity() / GiB
            },
        };
        assert_ne!(count, 0, "Strand count must be nonzero");
        let mut strands = Vec::with_capacity(count as usize);

        let size = dev.capacity() / count;
        let size = (size / PAGE_SIZE) * PAGE_SIZE;

        // Allocate strands
        // The first page is reserved for metadata
        let mut left = dev.capacity();
        for i in 0..count {
            let off = i * size + PAGE_SIZE;
            let len = cmp::min(size, left);
            debug_assert_ne!(len, 0, "Length of strand must be nonzero");

            left -= len;
            strands.push(Strand::new(dev.clone(), off, len));
        }
        debug_assert_eq!(left, 0, "Not all space is allocated in a strand");

        StrandPool {
            dev: dev,
            strands: strands.into_boxed_slice(),
        }
    }
}
