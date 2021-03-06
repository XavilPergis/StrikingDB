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
use parking_lot::RwLock;
use std::cmp::min;
use std::fmt;

pub struct ReadCache(RwLock<LruCache<Box<[u8]>, Box<[u8]>>>);

impl ReadCache {
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(512)
    }

    pub fn with_capacity(items: usize) -> Self {
        ReadCache(RwLock::new(LruCache::with_capacity(items)))
    }

    pub fn insert(&self, key: &[u8], val: &[u8]) -> Option<Box<[u8]>> {
        self.0.write().insert(
            Vec::from(key).into_boxed_slice(),
            Vec::from(val).into_boxed_slice(),
        )
    }

    pub fn get(&self, key: &[u8], val: &mut [u8]) -> Option<usize> {
        self.0.write().get(key).map(move |slice| {
            let slice = &**slice;
            let len = min(val.len(), slice.len());

            let dest = &mut val[..len];
            let src = &slice[..len];
            dest.copy_from_slice(src);

            len
        })
    }

    pub fn remove(&self, key: &[u8]) -> Option<Box<[u8]>> {
        self.0.write().remove(key)
    }

    #[allow(unused)]
    pub fn clear(&self) {
        self.0.write().clear();
    }
}

impl fmt::Debug for ReadCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut cache = self.0.write();
        let mut first = true;

        write!(f, "ReadCache({{")?;
        for (key, val) in cache.iter() {
            let comma = if first { "" } else { ", " };
            write!(f, "{}{:?}: {:?}", comma, &*key, &*val)?;
            first = false;
        }
        write!(f, "}})")?;

        Ok(())
    }
}
