/*
 * serial/alloc.rs
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

use capnp::Word;
use capnp::message::Allocator;
use super::buffer::Page;

#[derive(Debug, Clone, Default, Hash)]
pub struct PageAllocator {
    page: Page,
    off: usize,
}

impl PageAllocator {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

unsafe impl Allocator for PageAllocator {
    fn allocate_segment(&mut self, min_size: u32) -> (*mut Word, u32) {
        let len = (min_size * 4) as usize;
        if self.off + len >= self.page.len() {
            panic!("PageAllocator is out of free space");
        }

        let ptr: *mut u8 = &mut self.page[self.off];
        self.off += len;

        (ptr as *mut Word, min_size)
    }
}
