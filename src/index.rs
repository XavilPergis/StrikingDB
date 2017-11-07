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

use super::{MAX_KEY_LEN, FilePointer};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::thread;

pub type IndexTree = BTreeMap<Box<[u8]>, (FilePointer, bool)>;

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
    #[allow(unused)]
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
        let mut map = index.write();

        {
            let tuple = map.get_mut(self.key).expect("Locked entry is now empty");
            let (ref mut ptr, ref mut locked) = *tuple;
            debug_assert!(*locked, "Entry is unlocked");

            // If entry is Some(_), update entry
            if let Some(new_ptr) = self.value {
                *ptr = new_ptr;
                *locked = false;
                return;
            }
        }

        // If entry is None, delete entry
        map.remove(self.key);
    }
}

#[derive(Debug)]
pub struct Index(RwLock<IndexTree>);

impl Index {
    fn tree_valid(map: &IndexTree) -> bool {
        for (key, &(_, locked)) in map.iter() {
            if key.is_empty() || key.len() > MAX_KEY_LEN {
                return false;
            }

            if locked {
                return false;
            }
        }

        true
    }

    pub fn new() -> Self {
        Index(RwLock::new(BTreeMap::new()))
    }

    pub fn from(map: IndexTree) -> Self {
        debug_assert!(Self::tree_valid(&map));
        Index(RwLock::new(map))
    }

    pub fn exists(&self, key: &[u8]) -> bool {
        self.0.read().contains_key(key)
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
        let mut map = self.0.write();
        let mut value = None;

        // We use this stupid pattern instead of a
        // match because we need to .insert() in the
        // None case, but the borrow checker thinks
        // we already have a mutable reference to
        // "map" because of get_mut().
        if let Some(tuple) = map.get_mut(key) {
            let (ptr, ref mut locked) = *tuple;
            if *locked {
                return None;
            }

            *locked = true;
            value = Some(ptr);
        }

        if value.is_none() {
            let key_box = Vec::from(key).into_boxed_slice();
            map.insert(key_box, (0, true));
        }

        Some(IndexEntryGuard::new(&self.0, key.clone(), value))
    }

    pub fn get_mut(&mut self) -> &mut IndexTree {
        self.0.get_mut()
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}
