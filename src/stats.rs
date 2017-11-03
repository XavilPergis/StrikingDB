/*
 * stats.rs
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

use std::ops::AddAssign;

#[derive(Debug, Hash, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    pub read_bytes: u64,
    pub written_bytes: u64,
    pub trimmed_bytes: u64,
    pub buffer_read_bytes: u64,
    pub buffer_written_bytes: u64,
    pub valid_items: u64,
    pub deleted_items: u64,
}

impl AddAssign for Stats {
    fn add_assign(&mut self, rhs: Self) {
        self.read_bytes += rhs.read_bytes;
        self.written_bytes += rhs.written_bytes;
        self.trimmed_bytes += rhs.trimmed_bytes;
        self.buffer_read_bytes += rhs.buffer_read_bytes;
        self.buffer_written_bytes += rhs.buffer_written_bytes;
        self.valid_items += rhs.valid_items;
        self.deleted_items += rhs.deleted_items;
    }
}
