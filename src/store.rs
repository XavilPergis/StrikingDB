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

use cache::ReadCache;
use deleted::Deleted;
use index::Index;
use options::OpenOptions;
use serial::{read_item, write_item};
use std::fs::File;
use strand::Strand;
use super::device::Device;
use super::error::Error;
use super::volume::Volume;
use super::{MAX_KEY_LEN, MAX_VAL_LEN, FilePointer, Result};

#[derive(Debug)]
pub struct Store {
    volume: Volume,
    index: Index,
    deleted: Deleted,
    cache: ReadCache,
}

impl Store {
    // Create
    pub fn open(file: File, options: &OpenOptions) -> Result<Self> {
        let device = Device::open(file)?;
        let (volume, state) = Volume::open(device, options)?;
        let (index, deleted) = state.extract();

        Ok(Store {
            volume: volume,
            index: index,
            deleted: deleted,
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
        if !entry.exists() {
            return Err(Error::ItemNotFound);
        }

        let ptr = entry.value.unwrap();
        self.volume.read(
            ptr,
            |strand| self.lookup_item(strand, ptr, val),
        )
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
        if !entry.exists() {
            return Err(Error::ItemNotFound);
        }

        let old_ptr = entry.value.unwrap();
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

        let old_ptr = entry.value.unwrap();
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

        if entry.exists() {
            self.remove_item(key, old_ptr);
        }

        entry.value = Some(ptr);
        Ok(())
    }

    // Delete
    pub fn delete(&self, key: &[u8], val: &mut [u8]) -> Result<usize> {
        Self::verify_key(key)?;

        let entry = self.index.lock(key);
        if !entry.exists() {
            return Err(Error::ItemNotFound);
        }

        let ptr = entry.value.unwrap();
        self.remove_item(key, ptr);

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

    // Helpers
    fn lookup_item(&self, strand: &Strand, ptr: FilePointer, val: &mut [u8]) -> Result<usize> {
        read_item(strand, ptr, |ctx| ctx.copy_val(val))
    }

    fn remove_item(&self, key: &[u8], ptr: FilePointer) {
        self.cache.remove(key);
        self.deleted.add(ptr);
    }

    // Stats
    pub fn items(&self) -> usize {
        self.index.count()
    }

    pub fn deleted(&self) -> usize {
        self.deleted.count()
    }
}

unsafe impl Send for Store {}
unsafe impl Sync for Store {}
