/*
 * pod/item.rs
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

use super::{MAX_KEY_LEN, MAX_VAL_LEN, Pod};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(C, packed)]
pub struct ItemHeader {
    pub key_len: u16,
    pub val_len: u16,
}

impl ItemHeader {
    pub fn new(key_len: u16, val_len: u16) -> Self {
        ItemHeader {
            key_len: key_len,
            val_len: val_len,
        }
    }
}

impl Pod for ItemHeader {
    fn validate(&self) -> bool {
        key_len <= MAX_KEY_LEN && val_len <= MAX_VAL_LEN
    }
}
