/*
 * index/object.rs
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
use super::sync::CopyRwLock;
use super::wrap::{MutableEntry, LookupEntry, UpdateEntry, RemoveEntry, InsertEntry};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::collections::BTreeMap;

pub type IndexTree = BTreeMap<Box<[u8]>, CopyRwLock<FilePointer>>;

#[derive(Debug)]
pub struct Index(RwLock<IndexTree>);

impl Index {
    pub fn new() -> Self {
        Index(RwLock::new(BTreeMap::new()))
    }

    pub fn from(map: IndexTree) -> Self {
        Index(RwLock::new(map))
    }

    pub fn raw_read(&self) -> RwLockReadGuard<IndexTree> {
        self.0.read()
    }

    pub fn raw_write(&self) -> RwLockWriteGuard<IndexTree> {
        self.0.write()
    }

    pub fn get_mut(&mut self) -> &mut IndexTree {
        self.0.get_mut()
    }

    pub fn exists(&self, key: &[u8]) -> bool {
        self.0.read().contains_key(key)
    }

    pub fn lookup<'i, 'k>(&'i self, key: &'k [u8]) -> Option<LookupEntry<'i, 'k>> {
        let map = self.0.read();
        map.get(key).map(|lock| LookupEntry::new(self, key, lock))
    }

    pub fn update<'i, 'k>(&'i self, key: &'k [u8]) -> Option<UpdateEntry<'i, 'k>> {
        let map = self.0.read();
        map.get(key).map(|lock| UpdateEntry::new(self, key, lock))
    }

    pub fn remove<'i, 'k>(&'i self, key: &'k [u8]) -> Option<RemoveEntry<'i, 'k>> {
        let map = self.0.read();
        map.get(key).map(|lock| RemoveEntry::new(self, key, lock))
    }

    pub fn insert<'i, 'k>(&'i self, key: &'k [u8]) -> Option<InsertEntry<'i, 'k>> {
        use std::collections::btree_map::Entry;

        let mut map = self.0.write();
        let key_copy = key.to_vec().into_boxed_slice();
        let lock = match map.entry(key_copy) {
            Entry::Vacant(entry) => entry.insert(CopyRwLock::new(0)),
            Entry::Occupied(_) => return None,
        };

        Some(InsertEntry::new(self, key, lock))
    }

    pub fn entry<'i, 'k>(&'i self, key: &'k [u8]) -> MutableEntry<'i, 'k> {
        let mut map = self.0.write();
        if let Some(lock) = map.get_mut(key) {
            return MutableEntry::new(self, key, Some(lock));
        }

        {
            let key = key.to_vec().into_boxed_slice();
            map.insert(key, CopyRwLock::new(0));
        }

        MutableEntry::new(self, key, Some(&map[key]))
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}
