/*
 * store.rs
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

use super::{MAX_KEY_LEN, MAX_VAL_LEN, FilePointer, Result};
use super::device::{Ssd, Memory};
use super::error::Error;
use super::volume::Volume;
use cache::ReadCache;
use deleted::Deleted;
use index::Index;
use options::OpenOptions;
use serial::{DatastoreState, read_item, write_item};
use stats::Stats;
use std::path::Path;
use strand::Strand;

#[derive(Debug)]
pub struct Store<'a> {
    volume: Volume<'a>,
    index: Index,
    deleted: Deleted,
    cache: ReadCache,
}

impl<'a> Store<'a> {
    pub fn open<P: AsRef<Path>>(path: P, options: &OpenOptions) -> Result<Self> {
        let ssd = Ssd::open(path.as_ref())?;
        let (volume, state) = Volume::open(Box::new(ssd), options)?;
        let (index, deleted) = state.extract();

        Ok(Store {
            volume: volume,
            index: index,
            deleted: deleted,
            cache: ReadCache::new(),
        })
    }

    pub fn memory(bytes: usize, options: &OpenOptions) -> Result<Self> {
        let memory = Memory::new(bytes);
        let (volume, _) = Volume::open(Box::new(memory), options)?;

        Ok(Store {
            volume: volume,
            index: Index::new(),
            deleted: Deleted::new(),
            cache: ReadCache::new(),
        })
    }

    // Helper methods
    #[inline]
    fn verify_key(key: &[u8]) -> Result<()> {
        if key.is_empty() || key.len() > MAX_KEY_LEN {
            Err(Error::InvalidKey)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn verify_val(val: &[u8]) -> Result<()> {
        if val.len() > MAX_VAL_LEN {
            Err(Error::InvalidValue)
        } else {
            Ok(())
        }
    }

    // Read
    pub fn lookup(&self, key: &[u8], val: &mut [u8]) -> Result<usize> {
        Self::verify_key(key)?;

        if let Some(len) = self.cache.get(key, val) {
            return Ok(len);
        }

        let entry = self.index.lock(key);
        let ptr = match entry.value {
            Some(ptr) => ptr,
            None => return Err(Error::ItemNotFound),
        };

        self.volume.read(
            ptr,
            |strand| self.lookup_item(strand, ptr, val),
        )
    }

    pub fn exists(&self, key: &[u8]) -> bool {
        self.index.exists(key)
    }

    // Update
    pub fn insert(&self, key: &[u8], val: &[u8]) -> Result<()> {
        Self::verify_key(key)?;
        Self::verify_val(val)?;

        let mut entry = self.index.lock(key);
        if entry.exists() {
            return Err(Error::ItemExists);
        }

        let ptr = self.volume.write(|strand| {
            {
                let stats = &mut strand.stats.lock();
                stats.valid_items += 1;
            }

            write_item(strand, key, val)
        })?;

        entry.value = Some(ptr);
        Ok(())
    }

    pub fn update(&self, key: &[u8], val: &[u8]) -> Result<()> {
        Self::verify_key(key)?;
        Self::verify_val(val)?;

        let mut entry = self.index.lock(key);
        let old_ptr = match entry.value {
            Some(ptr) => ptr,
            None => return Err(Error::ItemNotFound),
        };

        let ptr = self.volume.write(|strand| {
            {
                let stats = &mut strand.stats.lock();
                stats.valid_items += 1;
                stats.deleted_items += 1;
            }

            write_item(strand, key, val)
        })?;

        self.remove_item(key, old_ptr);
        entry.value = Some(ptr);
        Ok(())
    }

    pub fn put(&self, key: &[u8], val: &[u8]) -> Result<()> {
        Self::verify_key(key)?;
        Self::verify_val(val)?;

        let mut entry = self.index.lock(key);
        let ptr = self.volume.write(|strand| {
            {
                let stats = &mut strand.stats.lock();
                stats.valid_items += 1;
                if entry.exists() {
                    stats.deleted_items += 1
                }
            }

            write_item(strand, key, val)
        })?;

        if let Some(old_ptr) = entry.value {
            self.remove_item(key, old_ptr);
        }

        entry.value = Some(ptr);
        Ok(())
    }

    /// Performs an atomic read-modify-write on the given item.
    /// This method takes the key to act upon, and a closure.
    /// The closure is passed three arguments: the key, the value
    /// buffer, and a boolean reference.
    ///
    /// If the item already exists in the datastore, then the
    /// value vector is filled with its current value, and
    /// "exists" is set to `true`. Otherwise the vector is
    /// empty and "exists" is set to `false`.
    ///
    /// After the execution of the closure, the value of the boolean
    /// is inspected. If it is true, then the item is inserted / updated
    /// with the given value in the vector. If it is false, the item is
    /// deleted.
    ///
    /// As this method has forms of handling for all cases of item
    /// existence / non-existence, it will never return
    /// [`Error::ItemNotFound`] or [`Error::Exists`].
    ///
    /// [`Error::Exists`]: enum.Error.html
    /// [`Error::ItemNotFound`]: enum.Error.html
    pub fn merge<F>(&self, key: &[u8], func: F) -> Result<()>
    where
        F: FnOnce(Option<Vec<u8>>) -> Option<Vec<u8>>,
    {
        Self::verify_key(key)?;

        let mut entry = self.index.lock(key);

        // Read a value from the store if it's there, and return it in a vec
        let val = match entry.value {
            Some(ptr) => {
                let val_buffer = self.volume.read(ptr, |strand| {
                    read_item(strand, ptr, |ctx| Ok(Vec::from(ctx.val()?)))
                })?;

                // NOTE: "updates" are really just a removal and an insert
                self.remove_item(key, ptr);

                Some(val_buffer)
            }
            None => None
        };

        // Call the user's function...
        let result = func(val);

        // Write it back!
        self.volume.write(|strand| {
            // Update stats
            {
                let stats = strand.stats.get_mut();
                if entry.exists() {
                    stats.deleted_items += 1;
                }
                if result.is_some() {
                    stats.valid_items += 1;
                }
            }

            entry.value = match result {
                Some(ref val) => Some(write_item(strand, key, val.as_slice())?),
                None => None,
            };

            Ok(())
        })
    }

    // Delete
    pub fn delete(&self, key: &[u8], val: &mut [u8]) -> Result<usize> {
        Self::verify_key(key)?;

        let mut entry = self.index.lock(key);
        let ptr = match entry.value {
            Some(ptr) => ptr,
            None => return Err(Error::ItemNotFound),
        };

        self.remove_item(key, ptr);
        entry.value = None;

        if let Some(len) = self.cache.get(key, val) {
            return Ok(len);
        }

        self.volume.read(ptr, |strand| {
            {
                let stats = &mut strand.stats.lock();
                stats.deleted_items += 1;
            }

            self.lookup_item(strand, ptr, val)
        })
    }

    pub fn remove(&self, key: &[u8]) -> Result<()> {
        Self::verify_key(key)?;

        let mut entry = self.index.lock(key);
        if let Some(ptr) = entry.value {
            self.volume.read(ptr, |strand| {
                let stats = &mut strand.stats.lock();
                stats.deleted_items += 1;
            });

            self.remove_item(key, ptr);
            entry.value = None;
        }

        Ok(())
    }

    // Stats
    #[inline]
    pub fn stats(&self) -> Stats {
        self.volume.stats()
    }

    // Helpers
    fn lookup_item(&self, strand: &Strand, ptr: FilePointer, buf: &mut [u8]) -> Result<usize> {
        read_item(strand, ptr, |ctx| {
            let key = ctx.key()?;
            let val = ctx.val()?;
            self.cache.insert(key, val);
            ctx.copy_val(buf)
        })
    }

    fn remove_item(&self, key: &[u8], ptr: FilePointer) {
        self.cache.remove(key);
        self.deleted.add(ptr);
    }

    fn write_state(&mut self) -> Result<()> {
        let index = self.index.get_mut();
        let deleted = self.deleted.get_mut();

        self.volume.write(|strand| {
            let state = DatastoreState::new(index, deleted)?;
            state.write(strand)
        })
    }
}

impl<'a> Drop for Store<'a> {
    fn drop(&mut self) {
        self.write_state().expect("Writing datastore state failed");
    }
}

unsafe impl<'a> Send for Store<'a> {}
unsafe impl<'a> Sync for Store<'a> {}
