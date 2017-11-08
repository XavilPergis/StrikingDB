/*
 * index.rs
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

use super::FilePointer;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

pub type IndexTree = BTreeMap<Box<[u8]>, RwLock<FilePointer>>;

#[derive(Debug)]
pub struct Index(RwLock<IndexTree>);

impl Index {
    pub fn new() -> Self {
        Index(RwLock::new(BTreeMap::new()))
    }

    pub fn from(map: IndexTree) -> Self {
        Index(RwLock::new(map))
    }

    pub fn get_mut(&mut self) -> &mut IndexTree {
        self.0.get_mut()
    }

    pub fn exists(&self, key: &[u8]) -> bool {
        self.0.read().contains_key(key)
    }

    pub fn entry<'i, 'k>(&'i self, key: &'k [u8]) -> IndexEntry<'i, 'k> {
        let mut map = self.0.write();
        if let Some(lock) = map.get_mut(key) {
            return IndexEntry::new(self, key, lock.write(), true);
        }

        {
            let key = key.to_vec().into_boxed_slice();
            map.insert(key, RwLock::new(0));
        }

        IndexEntry::new(self, key, map[key].write(), false)
    }

    pub fn get(&self, key: &[u8]) -> Option<RwLockReadGuard<FilePointer>> {
        let map = self.0.read();
        map.get(key).map(|lock| lock.read())
    }

    pub fn update(&self, key: &[u8]) -> Option<RwLockWriteGuard<FilePointer>> {
        let map = self.0.read();
        map.get(key).map(|lock| lock.write())
    }

    pub fn insert(&self, key: &[u8]) -> Option<RwLockWriteGuard<FilePointer>> {
        let map = self.0.write();
        if map.contains_key(key) {
            return None;
        }

        {
            let key = key.to_vec().into_boxed_slice();
            let val = RwLock::new(0);
            map.insert(key, val);
        }

        Some(map[key].write())
    }

    pub fn remove<'i, 'k>(&'i self, key: &'k [u8]) -> Option<IndexRemoved<'i, 'k>> {
        let map = self.0.read();
        map.get(key).map(|lock| IndexRemoved::new(self, lock.write(), key))
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub struct IndexEntry<'i, 'k> {
    index: &'i Index,
    guard: RwLockWriteGuard<'i, FilePointer>,
    key: &'k [u8],
    value: Option<FilePointer>,
}

impl<'i, 'k> IndexEntry<'i, 'k> {
    fn new(
        index: &'i Index,
        key: &'k [u8],
        guard: RwLockWriteGuard<'i, FilePointer>,
        exists: bool,
    ) -> Self {
        let value = match exists {
            true => Some(*guard),
            false => None,
        };

        IndexEntry {
            index: index,
            guard: guard,
            key: key,
            value: value,
        }
    }

    pub fn exists(&self) -> bool {
        self.value.is_some()
    }
}

impl<'i, 'k> Deref for IndexEntry<'i, 'k> {
    type Target = Option<FilePointer>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'i, 'k> DerefMut for IndexEntry<'i, 'k> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'i, 'k> Drop for IndexEntry<'i, 'k> {
    fn drop(&mut self) {
        let mut map = self.index.0.write();
        match self.value {
            Some(ptr) => {
                *self.guard = ptr;
            },
            None => {
                map.remove(self.key).unwrap();
            }
        }
    }
}

#[must_use]
pub struct IndexRemoved<'i, 'k> {
    index: &'i Index,
    guard: RwLockWriteGuard<'i, FilePointer>,
    key: &'k [u8],
}

impl<'i, 'k> IndexRemoved<'i, 'k> {
    fn new(
        index: &'i Index,
        guard: RwLockWriteGuard<'i, FilePointer>,
        key: &'k [u8],
    ) -> Self {
        IndexRemoved {
            index: index,
            guard: guard,
            key: key,
        }
    }
}

impl<'i, 'k> Deref for IndexRemoved<'i, 'k> {
    type Target = FilePointer;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<'i, 'k> Drop for IndexRemoved<'i, 'k> {
    fn drop(&mut self) {
        let mut map = self.index.0.write();
        map.remove(self.key).unwrap();
    }
}
