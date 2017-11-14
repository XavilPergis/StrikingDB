/*
 * index/wrap.rs
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
use super::entry::{IndexEntry, IndexEntryMut};
use super::object::Index;
use super::sync::CopyRwLock;
use std::ops::{Deref, DerefMut};

pub type MutableEntry<'i, 'k> = IndexEntryMut<'i, 'k>;

#[must_use]
#[derive(Debug)]
pub struct LookupEntry<'i, 'k>(IndexEntry<'i, 'k>);

impl<'i, 'k> LookupEntry<'i, 'k> {
    pub fn new(
        index: &'i Index,
        key: &'k [u8],
        entry: &CopyRwLock<FilePointer>,
    ) -> Self {
        LookupEntry(IndexEntry::new(index, key, entry, false))
    }
}

impl<'i, 'k> Deref for LookupEntry<'i, 'k> {
    type Target = FilePointer;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[must_use]
#[derive(Debug)]
pub struct UpdateEntry<'i, 'k>(IndexEntry<'i, 'k>);

impl<'i, 'k> UpdateEntry<'i, 'k> {
    pub fn new(
        index: &'i Index,
        key: &'k [u8],
        entry: &CopyRwLock<FilePointer>,
    ) -> Self {
        UpdateEntry(IndexEntry::new(index, key, entry, true))
    }
}

impl<'i, 'k> Deref for UpdateEntry<'i, 'k> {
    type Target = FilePointer;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'i, 'k> DerefMut for UpdateEntry<'i, 'k> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

#[must_use]
#[derive(Debug)]
pub struct RemoveEntry<'i, 'k>(IndexEntryMut<'i, 'k>, FilePointer);

impl<'i, 'k> RemoveEntry<'i, 'k> {
    pub fn new(
        index: &'i Index,
        key: &'k [u8],
        entry: &CopyRwLock<FilePointer>,
    ) -> Self {
        let entry = IndexEntryMut::new(index, key, Some(entry));
        let value = entry.unwrap();
        RemoveEntry(entry, value)
    }
}

impl<'i, 'k> Deref for RemoveEntry<'i, 'k> {
    type Target = FilePointer;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<'i, 'k> Drop for RemoveEntry<'i, 'k> {
    fn drop(&mut self) {
        *self.0 = None;
    }
}

#[must_use]
#[derive(Debug)]
pub struct InsertEntry<'i, 'k>(IndexEntry<'i, 'k>);

impl<'i, 'k> InsertEntry<'i, 'k> {
    pub fn new(
        index: &'i Index,
        key: &'k [u8],
        entry: &CopyRwLock<FilePointer>,
    ) -> Self {
        InsertEntry(IndexEntry::new(index, key, entry, true))
    }
}

impl<'i, 'k> Deref for InsertEntry<'i, 'k> {
    type Target = FilePointer;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'i, 'k> DerefMut for InsertEntry<'i, 'k> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl<'i, 'k> Drop for InsertEntry<'i, 'k> {
    fn drop(&mut self) {
        debug_assert_ne!(*self.0, 0, "Value not set before insert");
    }
}
