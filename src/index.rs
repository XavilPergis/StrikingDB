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

use parking_lot::RwLock;
use std::collections::BTreeMap;
use super::FilePointer;

#[derive(Debug)]
pub struct Index(RwLock<BTreeMap<Box<[u8]>, FilePointer>>);

impl Index {
    pub fn new() -> Self {
        Index(RwLock::new(BTreeMap::new()))
    }

    pub fn key_exists(&self, key: &[u8]) -> bool {
        self.0.read().contains_key(key)
    }

    pub fn get(&self, key: &[u8]) -> Option<FilePointer> {
        self.0.read().get(key).map(|x| *x)
    }

    pub fn put(&self, key: &[u8], value: FilePointer) -> Option<FilePointer> {
        let key = Vec::from(key).into_boxed_slice();
        self.0.write().insert(key, value)
    }

    pub fn count(&self) -> usize {
        self.0.read().len()
    }

    pub fn remove(&self, key: &[u8]) -> Option<FilePointer> {
        self.0.write().remove(key)
    }
}
