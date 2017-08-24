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

const GiB: u64 = 1024 * 1024 * 1024;

pub struct StrandPool {
    dev: Device,
    strands: Box<[Strand]>,
}

impl StrandPool {
    pub fn new(dev: Device, count: Option<usize>) -> Self {
        let count = match count {
            Some(x) => x,
            None => {
                let gb = (dev.capacity() / GiB) as usize;
                8 * num_cpus::get() * gb
            },
        };
        assert!(count != 0, "Strand count must be nonzero");
        let strands = Vec::with_capacity(count);

        StrandPool {
            dev: dev,
            strands: strands.into_boxed_slice(),
        }
    }
}
