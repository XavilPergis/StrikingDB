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

use index::Index;
use options::OpenOptions;
use std::fs::File;
use std::io::Write;
use super::device::Device;
use super::error::{SError, SResult};
use super::strand_pool::StrandPool;

#[derive(Debug)]
pub struct Store {
    pool: StrandPool,
    index: Index,
}

impl Store {
    // Create
    pub fn open(file: File, options: OpenOptions) -> SResult<Self> {
        // TODO
        let pool = StrandPool::new(Device::open(file)?, &options);
        Ok(Store {
            index: Index::new(),
            pool,
        })
    }

    // Read
    pub fn lookup<W: Write>(&self, key: &[u8], value: W) -> SResult<usize> {
        let ptr = match self.index.get(key) {
            Some(ptr) => ptr,
            None => return Err(SError::ItemNotFound),
        };
        let strand = self.pool.read(ptr);
        let item = strand.item(ptr);
        let bytes = item.value(value);
        Ok(bytes)
    }

    // Update
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> SResult<()> {
        if self.index.key_exists(key) {
            return Err(SError::ItemExists);
        }

        let ptr = self.pool.write().append(key, value)?;
        self.index.put(key, ptr);

        Ok(())
    }

    pub fn update(&mut self, key: &[u8], value: &[u8]) -> SResult<()> {
        if !self.index.key_exists(key) {
            return Err(SError::ItemNotFound);
        }

        self.remove(key)?;
        let ptr = self.pool.write().append(key, value)?;
        self.index.put(key, ptr);

        Ok(())
    }

    pub fn put(&mut self, key: &[u8], value: &[u8]) -> SResult<()> {
        if self.index.key_exists(key) {
            self.remove(key)?;
        }

        let ptr = self.pool.write().append(key, value)?;
        self.index.put(key, ptr);

        Ok(())
    }

    // Delete
    pub fn delete<W: Write>(&mut self, key: &[u8], value: W) -> SResult<()> {
        unimplemented!();
    }

    pub fn remove(&mut self, key: &[u8]) -> SResult<()> {
        unimplemented!()
    }
}
