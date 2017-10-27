/*
 * serial/reader.rs
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

use std::cmp;
use std::io::{self, BufRead, Read, Seek, SeekFrom};
use super::{PAGE_SIZE, Page};

pub struct PageReader<'a> {
    page: &'a Page,
    cursor: usize,
}

impl<'a> PageReader<'a> {
    #[inline]
    pub fn new(page: &'a Page) -> Self {
        PageReader {
            page: page,
            cursor: 0,
        }
    }
}

impl<'a> Seek for PageReader<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_cursor: i64 = match pos {
            SeekFrom::Start(idx) => cmp::min(idx, PAGE_SIZE) as i64,
            SeekFrom::End(off) => PAGE_SIZE as i64 + off,
            SeekFrom::Current(off) => cmp::min(self.cursor as i64 + off, PAGE_SIZE as i64),
        };

        if new_cursor >= 0 {
            self.cursor = new_cursor as usize;
            Ok(new_cursor as u64)
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "seek before first byte"))
        }
    }
}

impl<'a> Read for PageReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = cmp::min(PAGE_SIZE as usize - self.cursor, buf.len());
        let src = &self.page[self.cursor..self.cursor+len];
        let dest = &mut buf[..len];
        dest.copy_from_slice(src);

        self.cursor += len;
        Ok(len)
    }
}

impl<'a> BufRead for PageReader<'a> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(&self.page[..])
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.cursor = cmp::min(PAGE_SIZE as usize, self.cursor + amt);
    }
}

pub struct StrandReader;
