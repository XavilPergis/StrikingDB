/*
 * serial/io.rs
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
use std::io::{self, BufRead, Read, Write};

use super::buffer::{Block, Page};
use super::error::Error;
use super::strand::Strand;
use super::utils::align;
use super::{PAGE_SIZE, PAGE_SIZE64, FilePointer};

#[derive(Debug, Hash, Clone, Copy, PartialEq)]
enum BufferStatus {
    Clean,
    Dirty,
    Empty,
}

fn to_io_error(err: Error) -> io::Error {
    use Error::Io;

    match err {
        Io(Some(err)) => err,
        Io(None) => io::Error::new(io::ErrorKind::Other, Io(None)),
        _ => panic!("Non-I/O error recieved from strand I/O"),
    }
}

#[derive(Debug, Clone)]
pub struct StrandReader<'a> {
    strand: &'a Strand,
    page: Page,
    status: BufferStatus,
    cursor: u64,
}

impl<'a> StrandReader<'a> {
    pub fn new(strand: &'a Strand, ptr: FilePointer) -> Self {
        assert!(strand.contains_ptr(ptr), "Pointer isn't in the bounds of this strand");

        StrandReader {
            strand: strand,
            page: Page::default(),
            status: BufferStatus::Empty,
            cursor: ptr - strand.start(),
        }
    }

    // Ignores the cursor. It is up to the caller to ensure
    // that the cursor matches with the current buffer.
    fn read_page(&mut self, offset: u64) -> io::Result<()> {
        debug_assert_eq!(offset % PAGE_SIZE64, 0, "Read offset isn't page aligned");

        match self.strand.read(offset, &mut self.page) {
            Ok(_) => {
                self.status = BufferStatus::Clean;
                Ok(())
            },
            Err(e) => Err(to_io_error(e)),
        }
    }
}

impl<'a> Read for StrandReader<'a> {
    // To avoid complex multi-page reads, only
    // read until the end of the current page
    //
    // In the future we could optimize
    // this for long reads that span several pages.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let page_off = align(self.cursor);
        self.read_page(page_off)?;

        let off = (self.cursor - page_off) as usize;
        let len = cmp::min(PAGE_SIZE - off, buf.len());

        let src = &self.page[off..off+len];
        let dest = &mut buf[..len];
        dest.copy_from_slice(src);

        if off + len >= PAGE_SIZE {
            debug_assert_eq!(page_off + 1, align((off + len) as u64));
            self.status = BufferStatus::Empty;
        }

        self.cursor += len as u64;
        Ok(len)
    }
}

impl<'a> BufRead for StrandReader<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let page_off = align(self.cursor);
        self.read_page(page_off)?;

        let off = (self.cursor - page_off) as usize;
        Ok(&self.page[off..])
    }

    fn consume(&mut self, amt: usize) {
        let amt = amt as u64;
        self.cursor = cmp::min(self.cursor + amt, self.strand.capacity());
    }
}

#[derive(Debug)]
pub struct StrandWriter<'a> {
    strand: &'a mut Strand,
    block: Block,
    status: BufferStatus,
    cursor: u64,
}

impl<'a> StrandWriter<'a> {
    pub fn new(strand: &'a mut Strand) -> Self {
        let offset = strand.offset();

        StrandWriter {
            strand: strand,
            block: Block::default(),
            status: BufferStatus::Empty,
            cursor: offset,
        }
    }

    pub fn get_pointer(&self) -> FilePointer {
        self.cursor + self.strand.start()
    }

    pub fn write_metadata(&mut self) -> io::Result<()> {
        unimplemented!();
    }
}

// check remaining space
// update strand offset
// update strand stats (incl bytes)
// write strand header

impl<'a> Write for StrandWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!();
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!();
    }
}

impl<'a> Drop for StrandWriter<'a> {
    fn drop(&mut self) {
        self.flush().expect("Flush during drop failed");
    }
}
