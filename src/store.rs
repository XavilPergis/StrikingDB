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

/// Represents an open handle to a datastore.
///
/// This handle is thread-safe, and thus, is both [`Send`] and [`Sync`],
/// and only requires a `&self` in order to operate on it.
///
/// ## Panics
/// When dropped, it writes the current state of the indexer and deleted
/// items. If this fails, the destructor will panic.
///
/// [`Send`]: https://doc.rust-lang.org/stable/std/marker/trait.Send.html
/// [`Sync`]: https://doc.rust-lang.org/stable/std/marker/trait.Sync.html
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

    /// Gets the given item in the datastore.
    ///
    /// This method searches for the item associated with the passed
    /// `key`. If found, then as much of the value as will fit is
    /// copied into `val`, and the number of bytes written is returned.
    ///
    /// If there is no such item, then [`Error::ItemNotFound`] is returned.
    ///
    /// [`Error::ItemNotFound`]: enum.Error.html
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

    /// Checks if the given item exists in the datastore.
    /// This is a fast check, since it only inspects the
    /// in-memory index, and does not need to go to disk
    /// to fetch any data.
    pub fn exists(&self, key: &[u8]) -> bool {
        self.index.exists(key)
    }

    /// Inserts an item into the datastore.
    ///
    /// This method will insert the item associated with the passed
    /// `key`, failing with [`Error::ItemExists`] if there is already
    /// such an item in the datastore.
    ///
    /// [`Error::ItemExists`]: enum.Error.html
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

    /// Updates an item in the datastore.
    ///
    /// This method will update the item associated with the passed
    /// `key` with the provided `val`. If there is no such item,
    /// then [`Error::ItemNotFound`] is returned.
    ///
    /// [`Error::ItemNotFound`]: enum.Error.html
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

    /// Puts an item in the datastore.
    ///
    /// This method will place the provided key/value pair into the
    /// datastore, regardless of whether or not such an item existed
    /// before.
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

    /// Removes an item from the datastore.
    ///
    /// This will remove the data associated with the
    /// given key from the datastore. It is not an error
    /// if the item doesn't exist.
    ///
    /// Note that the data may not be deleted from disk,
    /// but rather it is flagged for removal when
    /// vacuuming occurs.
    pub fn remove(&self, key: &[u8]) -> Result<()> {
        Self::verify_key(key)?;

        let entry = self.index.lock(key);
        if let Some(ptr) = entry.value {
            self.volume.read(ptr, |strand| {
                let stats = &mut strand.stats.lock();
                stats.deleted_items += 1;
            });

            self.remove_item(key, ptr);
        }

        Ok(())
    }

    /// Deletes an item from the datastore, retrieving it's value data.
    ///
    /// This will remove the item from the datastore, and retrieve its
    /// value before deletion, copying as much as can fit into the buffer.
    /// The number of bytes is returned. (See [`lookup`]).
    ///
    /// If the item does not exist, then [`Error::ItemNotFound`] is returned.
    ///
    /// Note that the data may not be deleted from disk,
    /// but rather it is flagged for removal when
    /// vacuuming occurs.
    ///
    /// [`Error::ItemNotFound`]: enum.Error.html
    /// [`lookup`]: #method.lookup
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

    /// Performs an atomic read-modify-write on the given item.
    ///
    /// The closure takes an option containing the value buffer,
    /// and returns an option containing a potentially modified
    /// buffer. The input is `Some` if there is an associated
    /// on disk, and `None` if there isn't. The table below shows
    /// the behavior of this function for different combinations
    /// of inputs and outputs:
    ///
    /// |    Input   |    Output   | Behavior                                |
    /// |:----------:|:-----------:|-----------------------------------------|
    /// |   `None`   |    `None`   | Nothing                                 |
    /// |   `None`   | `Some(buf)` | Entry will be inserted with value `buf` |
    /// |  `Some(_)` |    `None`   | Entry will be deleted                   |
    /// |  `Some(_)` | `Some(buf)` | Entry will be updated with value `buf`  |
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

    /// Retrieves statistics about the current state of the datastore.
    /// See [`Stats`] for more information about each field.
    ///
    /// [`Stats`]: struct.Stats.html
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
