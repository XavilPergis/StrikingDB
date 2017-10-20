/*
 * page.rs
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

use std::ops::{Index, IndexMut, Range, RangeFrom, RangeFull, RangeTo};
use super::PAGE_SIZE;

pub type PageId = u64;

pub struct Page {
    bytes: [u8; PAGE_SIZE as usize],
    dirty: bool,
}

impl Page {
    pub fn new() -> Self {
        Page {
            bytes: [0; PAGE_SIZE as usize],
            dirty: false,
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        assert!(!self.dirty, "Page dropped while dirty");
    }
}

// Read-only slices
impl Index<usize> for Page {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.bytes[idx]
    }
}

impl Index<Range<usize>> for Page {
    type Output = [u8];

    fn index(&self, idx: Range<usize>) -> &[u8] {
        &self.bytes[idx]
    }
}

impl Index<RangeFrom<usize>> for Page {
    type Output = [u8];

    fn index(&self, idx: RangeFrom<usize>) -> &[u8] {
        &self.bytes[idx]
    }
}

impl Index<RangeFull> for Page {
    type Output = [u8];

    fn index(&self, _: RangeFull) -> &[u8] {
        &self.bytes[..]
    }
}

impl Index<RangeTo<usize>> for Page {
    type Output = [u8];

    fn index(&self, idx: RangeTo<usize>) -> &[u8] {
        &self.bytes[idx]
    }
}

// Mutable slices
impl IndexMut<usize> for Page {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        self.dirty = true;
        &mut self.bytes[idx]
    }
}

impl IndexMut<Range<usize>> for Page {
    fn index_mut(&mut self, idx: Range<usize>) -> &mut [u8] {
        self.dirty = true;
        &mut self.bytes[idx]
    }
}

impl IndexMut<RangeFrom<usize>> for Page {
    fn index_mut(&mut self, idx: RangeFrom<usize>) -> &mut [u8] {
        self.dirty = true;
        &mut self.bytes[idx]
    }
}

impl IndexMut<RangeFull> for Page {
    fn index_mut(&mut self, _: RangeFull) -> &mut [u8] {
        self.dirty = true;
        &mut self.bytes[..]
    }
}

impl IndexMut<RangeTo<usize>> for Page {
    fn index_mut(&mut self, idx: RangeTo<usize>) -> &mut [u8] {
        self.dirty = true;
        &mut self.bytes[idx]
    }
}
