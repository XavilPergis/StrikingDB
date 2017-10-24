/*
 * cache/read.rs
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

pub struct ReadCache(LruCache<Box<[u8]>, Box<[u8]>>);

impl ReadCache {
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(512)
    }

    pub fn with_capacity(items: usize) -> Self {
        ReadCache(LruCache::with_capacity(items))
    }

    pub fn insert(&mut self, key: &[u8], val: &[u8]) -> Option<Box<[u8]>> {
        self.0.insert(
            Vec::from(key).into_boxed_slice(),
            Vec::from(val).into_boxed_slice(),
        )
    }

    pub fn lookup(&mut self, key: &[u8]) -> Option<&[u8]> {
        self.0.get(key).map(|x| &**x)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl fmt::Debug for ReadCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ReadCache({} items)", self.0.len())
    }
}
