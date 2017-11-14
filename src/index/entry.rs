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

use super::FilePointer;
use super::object::Index;
use super::sync::CopyRwLock;
use std::ops::{Deref, DerefMut};

#[must_use]
#[derive(Debug)]
pub struct IndexEntry<'i, 'k> {
    index: &'i Index,
    key: &'k [u8],
    value: FilePointer,
    exclusive: bool,
}

impl<'i, 'k> IndexEntry<'i, 'k> {
    pub fn new(
        index: &'i Index,
        key: &'k [u8],
        entry: &CopyRwLock<FilePointer>,
        exclusive: bool,
    ) -> Self {
        let value = if exclusive {
            entry.write_lock()
        } else {
            entry.read_lock()
        };

        IndexEntry {
            index: index,
            key: key,
            value: value,
            exclusive: exclusive,
        }
    }
}

impl<'i, 'k> Deref for IndexEntry<'i, 'k> {
    type Target = FilePointer;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'i, 'k> DerefMut for IndexEntry<'i, 'k> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if !self.exclusive {
            panic!("Cannot modify without exclusive lock");
        }

        &mut self.value
    }
}

impl<'i, 'k> Drop for IndexEntry<'i, 'k> {
    fn drop(&mut self) {
        let map = self.index.raw_read();
        let lock = map.get(self.key).unwrap();

        if self.exclusive {
            lock.write_unlock(self.value);
        } else {
            lock.read_unlock();
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct IndexEntryMut<'i, 'k> {
    index: &'i Index,
    key: &'k [u8],
    value: Option<FilePointer>,
}

impl<'i, 'k> IndexEntryMut<'i, 'k> {
    pub fn new(
        index: &'i Index,
        key: &'k [u8],
        entry: Option<&CopyRwLock<FilePointer>>,
    ) -> Self {
        let value = entry.map(|lock| lock.write_lock());

        IndexEntryMut {
            index: index,
            key: key,
            value: value,
        }
    }

    #[inline]
    pub fn exists(&self) -> bool {
        self.index.exists(self.key)
    }
}

impl<'i, 'k> Deref for IndexEntryMut<'i, 'k> {
    type Target = Option<FilePointer>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'i, 'k> DerefMut for IndexEntryMut<'i, 'k> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'i, 'k> Drop for IndexEntryMut<'i, 'k> {
    fn drop(&mut self) {
        let mut map = self.index.raw_write();
        match self.value {
            Some(value) => {
                let entry = map.get_mut(self.key).unwrap();
                entry.write_unlock(value);
            },
            None => {
                map.remove(self.key);
            },
        }
    }
}
