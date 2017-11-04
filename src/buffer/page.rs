/*
 * buffer/page.rs
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

use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use super::{PAGE_SIZE, ByteArray};

#[derive(Clone)]
pub struct Page([u8; PAGE_SIZE]);

impl Deref for Page {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl DerefMut for Page {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl AsRef<[u8]> for Page {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl Default for Page {
    fn default() -> Self {
        Page([0; PAGE_SIZE])
    }
}

impl Hash for Page {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.0[..]);
    }
}

impl fmt::Debug for Page {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Page: {:?}..", &self.0[..16])
    }
}

impl ByteArray for Page {}
