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
use std::io::{self, BufRead, Cursor, Read, Seek, SeekFrom};
use super::error::Error;
use super::page::Page;
use super::strand::Strand;
use super::utils::align;
use super::{PAGE_SIZE, FilePointer};

pub type PageReader<'a> = Cursor<&'a Page>;

#[derive(Debug, Clone)]
pub struct StrandReader<'a> {
    strand: &'a Strand,
    page: Page,
    page_id: Option<u64>,
    start: usize,
    cursor: usize,
}

impl<'a> StrandReader<'a> {
    pub fn new(strand: &'a Strand, ptr: FilePointer) -> Self {
        assert!(strand.contains_ptr(ptr));

        StrandReader {
            strand: strand,
            page: Page::default(),
            page_id: None,
            start: ptr - strand.start() as usize,
            cursor: 0,
        }
    }

    fn read_page(&mut self, page_id: u64) -> io::Result<()> {
        if let Some(id) = self.page_id {
            if id == page_id {
                return Ok(());
            }
        }

        let off = page_id * PAGE_SIZE;
        self.strand.read(off, &mut self.page).map_err(Self::to_io_error)
    }

    #[inline]
    fn get_page_id(&self) -> u64 {
        self.cursor as u64 / PAGE_SIZE
    }

    fn to_io_error(err: Error) -> io::Error {
        use Error::Io;

        match err {
            Io(Some(err)) => err,
            Io(None) => io::Error::new(io::ErrorKind::Other, Io(None)),
            _ => panic!("Non-I/O error recieved from strand I/O"),
        }
    }
}

impl<'a> Read for StrandReader<'a> {
    // To avoid complex multi-page reads, only
    // read until the end of the current page
    //
    // In the future we will want to optimize
    // this for long reads that span several pages.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let page_id = self.get_page_id();
        self.read_page(page_id)?;
        let off = self.cursor % PAGE_SIZE as usize;
        let len = cmp::min(PAGE_SIZE as usize - off, buf.len());

        let src = &self.page[off..off+len];
        let dest = &mut buf[..len];
        dest.copy_from_slice(src);

        self.cursor += len;
        Ok(len)
    }
}

impl<'a> BufRead for StrandReader<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let page_id = self.get_page_id();
        let off = self.cursor % PAGE_SIZE as usize;
        self.read_page(page_id)?;
        Ok(&self.page[off..])
    }

    fn consume(&mut self, amt: usize) {
        self.cursor = cmp::min(self.cursor + amt, self.strand.capacity() as usize);
    }
}
