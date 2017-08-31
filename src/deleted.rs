/*
 * deleted.rs
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

use std::collections::BTreeSet;
use std::sync::RwLock;
use super::FilePointer;

#[derive(Debug)]
pub struct Deleted(RwLock<BTreeSet<FilePointer>>);

impl Deleted {
    pub fn new() -> Self {
        Deleted(RwLock::new(BTreeSet::new()))
    }

    pub fn put(&self, value: FilePointer) {
        let exists = match self.0.write() {
            Ok(ref mut set) => set.insert(value),
            Err(ref mut poison) => poison.get_mut().insert(value),
        };
        assert!(!exists, "Deleted item already tracked");
    }

    pub fn count(&self) -> usize {
        match self.0.read() {
            Ok(ref set) => set.len(),
            Err(ref poison) => poison.get_ref().len(),
        }
    }
}
