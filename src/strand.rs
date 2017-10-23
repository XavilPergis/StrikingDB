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

use cache::LruCache;
use item::Item;
use page::{Page, PageId};
use raw_strand::RawStrand;
use super::{FilePointer, Result};

type CleanupFn = FnMut(PageId, &mut Page) -> Result<()>;

#[derive(Debug)]
pub struct Strand {
    cache: LruCache<PageId, Page, CleanupFn>,
    strand: RawStrand,
}

impl Strand {
    pub fn new(raw_strand: RawStrand) -> Self {
        const CACHE_CAPACITY: usize = 512;

        let cache = LruCache::with_capacity(
            Box::new(|id, page| page.flush(&mut raw_strand, id)),
            CACHE_CAPACITY,
        );

        Strand {
            cache: cache,
            strand: raw_strand,
        }
    }

    #[inline]
    pub fn start(&self) -> u64 {
        self.strand.start()
    }

    #[inline]
    pub fn capacity(&self) -> u64 {
        self.strand.capacity()
    }

    // FIXME
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
