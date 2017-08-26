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
use std::fs::File;
use std::io::{self, Write};
use super::strand_pool::StrandPool;
use super::strand::Strand;
use super::device::Device;
use super::error::{SError, SResult};

#[derive(Debug)]
pub struct Store {
    pool: StrandPool,
    index: Index,
}

impl Store {
    // Create
    pub fn load(file: File, strands: Option<usize>) -> SResult<Self> {
        // TODO
        let pool = StrandPool::new(Device::open(file)?, strands);
        Ok(Store {
            index: Index::new(),
            pool,
        })
    }

    // Read
    pub fn lookup<K, W>(&self, key: K, buffer: W) -> SResult<()>
    where
        K: AsRef<[u8]>,
        W: Write,
    {
        Ok(())
    }

    // Update
    pub fn insert<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self, key: &[u8], value: &[u8]) -> SResult<()> {
        if self.index.key_exists(key.as_ref()) {
            return Err(SError::KeyExists);
        }

        self.put(key.as_ref(), value.as_ref())
    }

    pub fn update<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self, key: &[u8], value: &[u8]) -> SResult<()> {
        if !self.index.key_exists(key) {
            return Err(SError::KeyNotFound);
        }

        unimplemented!()
    }

    pub fn put<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self, key: &[u8], value: &[u8]) -> SResult<()> {
        let mut index_map = self.index.index_map();
        let strand = self.pool.write();
        index_map.insert(Vec::from(key).into_boxed_slice(), ptr);

        unimplemented!()
    }

    // Delete
    pub fn delete(&mut self, key: &[u8]) -> SResult<()> {
        unimplemented!();
    }

    pub fn remove(&mut self, key: &[u8]) -> SResult<()> {
        unimplemented!()
    }
}
