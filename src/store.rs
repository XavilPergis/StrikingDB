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
use std::io::{self, Write};

pub struct Store {
    index: Index,
}

impl Store {
    pub fn new() -> Self {
        // TODO
        Store { index: Index::new() }
    }

    pub fn lookup<K, W>(&self, key: K, buffer: W) -> io::Result<()>
    where
        K: AsRef<[u8]>,
        W: Write,
    {
        Ok(())
    }
}
