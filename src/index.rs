/*
 * index.rs
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

use std::collections::BTreeMap;
use super::FilePointer;

type IndexMap = BTreeMap<Box<[u8]>, FilePointer>;

#[derive(Debug)]
pub struct Index(IndexMap);

impl Index {
    pub fn new() -> Self {
        Index(BTreeMap::new())
    }

    pub fn index_map(&mut self) -> &mut IndexMap {
        &mut self.0
    }

    pub fn key_exists<'a>(&self, key: &'a [u8]) -> bool {
        self.0.contains_key(key)
    }
}
