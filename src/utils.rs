/*
 * utils.rs
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

use std::fmt::Debug;
use std::ops::Deref;
use super::PAGE_SIZE;

#[inline]
pub fn align(off: u64) -> u64 {
    (off / PAGE_SIZE) * PAGE_SIZE
}

#[inline]
pub fn align_up(off: u64) -> u64 {
    let mut align_off = align(off);

    if align_off != off {
        align_off += 1;
    }

    align_off
}

#[derive(Debug)]
pub struct StableRef<T: Debug>(*const T);

impl<T: Debug> StableRef<T> {
    pub fn new(item: &T) -> Self {
        StableRef(item as *const T)
    }
}

impl<T: Debug> Deref for StableRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0 }
    }
}
