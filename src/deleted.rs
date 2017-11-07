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

use super::FilePointer;
use parking_lot::RwLock;
use std::collections::BTreeSet;

pub type DeletedSet = BTreeSet<FilePointer>;

#[derive(Debug)]
pub struct Deleted(RwLock<DeletedSet>);

impl Deleted {
    pub fn new() -> Self {
        Deleted(RwLock::new(BTreeSet::new()))
    }

    pub fn from(set: DeletedSet) -> Self {
        Deleted(RwLock::new(set))
    }

    pub fn add(&self, value: FilePointer) {
        let exists = self.0.write().insert(value);
        assert_eq!(exists, true, "Deleted item already tracked");
    }

    pub fn get_mut(&mut self) -> &mut DeletedSet {
        self.0.get_mut()
    }
}

impl Default for Deleted {
    fn default() -> Self {
        Self::new()
    }
}
