/*
 * cache.rs
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

use lru_time_cache::LruCache;
use std::fmt;
use super::{FilePointer, PageId, PAGE_SIZE};

pub struct Cache(LruCache<PageId, [u8; PAGE_SIZE as usize]>);

impl Cache {
    pub fn new() -> Self {
        const CACHE_CAPACITY: usize = 512;
        Cache(LruCache::with_capacity(CACHE_CAPACITY))
    }
}

impl fmt::Debug for Cache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cache({} items)", self.0.len())
    }
}
