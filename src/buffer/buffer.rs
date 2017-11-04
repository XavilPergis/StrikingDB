/*
 * buffer/buffer.rs
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

use std::ops::{Deref, DerefMut};
use super::{ByteArray, BufferStatus};

#[derive(Debug, Clone, Hash)]
pub struct Buffer<B: ByteArray> {
    bytes: B,
    status: BufferStatus,
    cursor: u64,
}

impl<B: ByteArray> Buffer<B> {
    pub fn new() -> Self {
        Buffer {
            bytes: B::default(),
            status: BufferStatus::Empty,
            cursor: 0,
        }
    }
}

impl<B: ByteArray> Deref for Buffer<B> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &*self.bytes
    }
}

impl<B: ByteArray> DerefMut for Buffer<B> {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut *self.bytes
    }
}

impl<B: ByteArray> Default for Buffer<B> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
