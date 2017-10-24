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
use std::fs::File;
use super::Result;
use super::device::Device;
use super::error::SError;
use super::volume::Volume;

#[derive(Debug)]
pub struct Store {
    volume: Volume,
    index: Index,
    deleted: Deleted,
    cache: ReadCache,
}

impl Store {
    // Create
    pub fn open(file: File, options: OpenOptions) -> Result<Self> {
        let device = Device::open(file)?;
        let volume = Volume::open(device, &options)?;

        Ok(Store {
            volume,
            index: Index::new(),
            deleted: Deleted::new(),
            cache: ReadCache::new(),
        })
    }

    // Read
    pub fn lookup(&self, key: &[u8], val: &mut [u8]) -> Result<usize> {
        if let Some(val_slice) = self.cache.get(key, val) {
            return Ok(val_slice.len());
        }

        let ptr = match self.index.get(key) {
            Some(ptr) => ptr,
            None => return Err(SError::ItemNotFound),
        };

        let item = self.volume.read(ptr).item(ptr);
        let bytes = item.value(val);

        Ok(bytes)
    }

    // Update
    pub fn insert(&self, key: &[u8], val: &[u8]) -> Result<()> {
        if self.index.key_exists(key) {
            return Err(SError::ItemExists);
        }

        let ptr = self.volume.write().append(key, val)?;
        self.index.put(key, ptr);
        Ok(())
    }

    pub fn update(&self, key: &[u8], val: &[u8]) -> Result<()> {
        if !self.index.key_exists(key) {
            return Err(SError::ItemNotFound);
        }

        self.remove_item(key)?;
        let ptr = self.volume.write().append(key, val)?;
        self.index.put(key, ptr);
        Ok(())
    }

    pub fn put(&self, key: &[u8], val: &[u8]) -> Result<()> {
        if self.index.key_exists(key) {
            self.remove_item(key)?;
        }

        let ptr = self.volume.write().append(key, val)?;
        self.index.put(key, ptr);
        Ok(())
    }

    // Delete
    pub fn delete(&self, key: &[u8], val: &mut [u8]) -> Result<usize> {
        if !self.index.key_exists(key) {
            return Err(SError::ItemNotFound);
        }

        let ptr = match self.index.get(key) {
            Some(ptr) => ptr,
            None => return Err(SError::ItemNotFound),
        };

        let item = self.volume.read(ptr).item(ptr);
        let bytes_witten = item.value(val);

        self.remove_item(key)?;

        Ok(bytes_witten)
    }

    pub fn remove(&self, key: &[u8]) -> Result<()> {
        match self.remove_item(key) {
            Ok(()) | Err(SError::ItemNotFound) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn remove_item(&self, key: &[u8]) -> Result<()> {
        let ptr = match self.index.get(key) {
            Some(ptr) => ptr,
            None => return Err(SError::ItemNotFound),
        };

        self.deleted.put(ptr);
        self.index.remove(key);
        Ok(())
    }

    // Stats
    pub fn items(&self) -> usize {
        self.index.count()
    }

    pub fn deleted(&self) -> usize {
        self.deleted.count()
    }
}
