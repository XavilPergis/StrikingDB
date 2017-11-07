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
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::collections::BTreeMap;

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
            let key = Vec::from(key).into_boxed_slice();
            let val = RwLock::new(0);
            map.insert(key, val);
        }

        Some(map[key].write())
    }

    pub fn delete(&self, key: &[u8]) -> Option<FilePointer> {
        let map = self.0.write();
        map.remove(key).map(|lock| lock.into_inner())
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}
