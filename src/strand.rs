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

use item::Item;
use lru_time_cache::LruCache;
use page::{Page, PageId};
use raw_strand::RawStrand;
use std::fmt;
use std::time::Duration;
use super::{FilePointer, Result};

pub struct Strand {
    raw: RawStrand,
    cache: LruCache<PageId, Page>,
}

impl Strand {
    pub fn new(raw_strand: RawStrand) -> Self {
        const CACHE_CAPACITY: usize = 512;
        let cache =
            LruCache::with_expiry_duration_and_capacity(Duration::from_millis(50), CACHE_CAPACITY);

        Strand {
            cache: cache,
            raw: raw_strand,
        }
    }

    #[inline]
    pub fn start(&self) -> u64 {
        self.raw.start()
    }

    #[inline]
    pub fn capacity(&self) -> u64 {
        self.raw.capacity()
    }

    // FIXME
    pub fn raw(&self) -> &RawStrand {
        &self.raw
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

    pub fn trim(&mut self, off: u64, len: u64) -> Result<()> {
        // TODO caching

        unimplemented!();
    }
}

impl fmt::Debug for Strand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Strand(<{} item page cache>, {:?})",
            self.cache.len(),
            self.raw
        )
    }
}
