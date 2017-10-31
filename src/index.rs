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
use std::marker::PhantomData;
use std::sync::Arc;
use std::thread;
use super::FilePointer;

type IndexTree = BTreeMap<Box<[u8]>, (FilePointer, bool)>;

#[must_use]
#[derive(Debug)]
pub struct IndexEntryGuard<'i, 'k> {
    phantom: PhantomData<&'i RwLock<IndexTree>>,
    index: *const RwLock<IndexTree>,
    key: &'k [u8],
    pub value: Option<FilePointer>,
}

impl<'i, 'k> IndexEntryGuard<'i, 'k> {
    fn new(index: &'i RwLock<IndexTree>, key: &'k [u8], value: Option<FilePointer>) -> Self {
        IndexEntryGuard {
            phantom: PhantomData,
            index: index,
            key: key,
            value: value,
        }
    }

    #[inline]
    pub fn key(&self) -> &[u8] {
        self.key
    }

    #[inline]
    pub fn exists(&self) -> bool {
        self.value.is_some()
    }
}

impl<'i, 'k> Drop for IndexEntryGuard<'i, 'k> {
    fn drop(&mut self) {
        let index = unsafe { &*self.index };
        let map = index.write();

        match guard.entry(self.key) {
            Vacant(_) => panic!("Locked entry was vacant"),
            Occupied(entry) => {
                match self.value {
                    Some(ptr) => entry.insert(ptr, false),
                    None => entry.remove(),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Index(RwLock<IndexTree>);

impl Index {
    pub fn new() -> Self {
        Index(RwLock::new(BTreeMap::new()))
    }

    pub fn lock<'i, 'k>(&'i self, key: &'k [u8]) -> IndexEntryGuard<'i, 'k> {
        loop {
            if let Some(guard) = self.try_lock(key) {
                return guard;
            }

            thread::yield_now();
        }
    }

    pub fn try_lock<'i, 'k>(&'i self, key: &'k [u8]) -> Option<IndexEntryGuard<'i, 'k>> {
        use std::collections::btree_map::Entry::{Occupied, Vacant};

        let map = self.0.write();
        let value;

        match map.entry(key) {
            Vacant(ref entry) => {
                entry.insert((0, true));
                value = None;
            },
            Occupied(ref entry) => {
                let (ptr, locked) = entry.get();
                if locked {
                    return None;
                }

                entry.get_mut().1 = true;
                value = Some(ptr);
            },
        }

        Some(IndexEntryGuard::new(&self.0, key.clone(), value))
    }
}
