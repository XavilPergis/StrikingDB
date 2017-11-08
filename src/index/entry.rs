/*
 * index/entry.rs
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

use std::mem;
use std::ops::{Deref, DerefMut};

#[must_use]
pub struct IndexEntry<'i, 'k> {
    index: &'i Index,
    key: &'k [u8],
    value: Option<FilePointer>,
}

impl<'i, 'k> IndexEntry<'i, 'k> {
    pub fn new(
        index: &'i Index,
        key: &'k [u8],
        value: Option<FilePointer>,
    ) -> Self {
        IndexEntry {
            index: index,
            key: key,
            value: value,
        }
    }

    #[inline]
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

impl<'i, 'k> Drop for IndexEntry<'i, 'k> {
    fn drop(&mut self) {
        let map = self.index.0.read();
    }
}

#[must_use]
pub struct IndexEntryMut<'i, 'k> {
    entry: IndexEntry<'i, 'k>,
    existed: bool,
}

impl<'i, 'k> IndexEntryMut<'i, 'k> {
    pub fn new(
        index: &'i Index,
        key: &'k [u8],
        value: Option<FilePointer>,
    ) -> Self {
        IndexEntryMut {
            entry: IndexEntry::new(index, key, value),
            existed: value.is_some(),
        }
    }

    #[inline]
    pub fn exists(&self) -> bool {
        self.0.exists()
    }
}

impl<'i, 'k> DerefMut for IndexEntryMut<'i, 'k> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'i, 'k> Drop for IndexEntryMut<'i, 'k> {
    fn drop(&mut self) {
        let mut map = self.index.0.write();
        match self.entry.value {
            Some(
        }

        mem::forget(self.entry);
    }
}

        if self.exists() != self.existed {
            let mut map = self.index.0.write();
            match self.value {
                Some(ptr) => {
                    *map.get_mut(self.key) = ptr;
                },
                None => {
                    map.remove(self.key).unwrap();
                }
            }
        }
    }
}
