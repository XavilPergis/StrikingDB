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

use std::cmp::min;
use std::io::{self, BufRead, Read, Write};
use super::buffer::{Block, Buffer, BufferStatus, Page};
use super::error::Error;
use super::header::StrandHeader;
use super::strand::Strand;
use super::utils::{align, block_align};
use super::{PAGE_SIZE, PAGE_SIZE64, TRIM_SIZE, TRIM_SIZE64, FilePointer};

thread_local! {
    static READ_BUFFER: Buffer<Page> = Buffer::new();
    static WRITE_BUFFER: Buffer<Block> = Buffer::new();
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
pub struct StrandReader<'s, 'd: 's> {
    strand: &'s Strand<'d>,
    cursor: u64,
}

impl<'s, 'd> StrandReader<'s, 'd> {
    pub fn new(strand: &'s Strand<'d>, ptr: FilePointer) -> Self {
        assert!(
            strand.contains_ptr(ptr),
            "Pointer isn't in the bounds of this strand"
        );

        StrandReader {
            strand: strand,
            cursor: ptr - strand.start(),
        }
    }

    // Ignores the cursor. It is up to the caller to ensure
    // that the cursor matches with the current buffer.
    fn read_page(&mut self, offset: u64) -> io::Result<()> {
        debug_assert_eq!(offset % PAGE_SIZE64, 0, "Read offset isn't page aligned");

        READ_BUFFER.with(|buffer| {
            match self.strand.read(offset, &mut buffer[..]) {
                Ok(_) => {
                    buffer.status = BufferStatus::Clean;
                    Ok(())
                }
                Err(e) => Err(to_io_error(e)),
            }
        })
    }
}

impl<'s, 'd> Read for StrandReader<'s, 'd> {
    // To avoid complex multi-page reads, only
    // read until the end of the current page
    //
    // In the future we could optimize
    // this for long reads that span several pages.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Update buffer
        let page_off = align(self.cursor);
        self.read_page(page_off)?;

        // Determine how many bytes to read
        let off = (self.cursor - page_off) as usize;
        let len = min(PAGE_SIZE - off, buf.len());

        READ_BUFFER.with(|buffer| {
            // Copy data
            let src = &buffer[off..off + len];
            let dest = &mut buf[..len];
            dest.copy_from_slice(src);

            // Update metadata
            {
                let len = len as u64;
                self.cursor += len;

                self.strand.stats.lock().buffer_read_bytes += len;
                let new_off = off as u64 + len;
                if new_off >= PAGE_SIZE64 {
                    debug_assert_eq!(page_off + 1, align(new_off));
                    buffer.status = BufferStatus::Empty;
                }
            }

            Ok(len)
        })
    }
}

impl<'s, 'd> BufRead for StrandReader<'s, 'd> {
    fn fill_buf(&mut self) -> io::Result<&'static [u8]> {
        // Update buffer
        let page_off = align(self.cursor);
        self.read_page(page_off)?;

        // Return slice
        READ_BUFFER.with(|buffer| {
            let off = (self.cursor - page_off) as usize;
            let slice = &buffer[off..];
            Ok(slice)
        })
    }

    fn consume(&mut self, amt: usize) {
        let amt = amt as u64;
        self.cursor = min(self.cursor + amt, self.strand.capacity());
    }
}

#[derive(Debug)]
pub struct StrandWriter<'s, 'd: 's> {
    strand: &'s mut Strand<'d>,
    cursor: u64,
    pub update_offset: bool,
}

impl<'s, 'd> StrandWriter<'s, 'd> {
    pub fn new(strand: &'s mut Strand<'d>) -> Self {
        let offset = strand.offset();

        StrandWriter {
            strand: strand,
            cursor: offset,
            update_offset: true,
        }
    }

    pub fn get_pointer(&self) -> FilePointer {
        self.cursor + self.strand.start()
    }

    pub fn write_metadata(&mut self) -> io::Result<()> {
        let header = StrandHeader::from(&mut self.strand);
        let mut page = Page::default();
        header.write(&mut page).map_err(to_io_error)?;
        self.strand.write(0, &page).map_err(to_io_error)?;
        Ok(())
    }
}

impl<'s, 'd> Write for StrandWriter<'s, 'd> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() > self.strand.remaining() as usize {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Out of disk space",
            ));
        }

        let block_off = block_align(self.cursor);

        // Update buffer if needed
        WRITE_BUFFER.with(|buffer| {
            if buffer.status == BufferStatus::Empty {
                self.strand.read(block_off, &mut buffer[..]).map_err(
                    to_io_error,
                )?;
                buffer.status = BufferStatus::Clean;
            }
        })?;

        // Determine how many bytes to write
        let off = self.cursor as usize % TRIM_SIZE;
        let len = min(TRIM_SIZE - off, buf.len());

        // Copy data
        WRITE_BUFFER.with(|buffer| {
            let src = &buf[..len];
            let dest = &mut buffer[off..off + len];
            dest.copy_from_slice(src);
        });

        // Update metadata
        WRITE_BUFFER.with(|buffer| {
            let len = len as u64;
            buffer.status = BufferStatus::Dirty;
            self.cursor += len;
            self.strand.stats.get_mut().buffer_written_bytes += len;

            if self.update_offset {
                self.strand.push_offset(len);
            }
        });

        // Flush if necessary
        if off + len >= TRIM_SIZE {
            WRITE_BUFFER.with(|buffer| {
                self.strand.write(block_off, &buffer[..]).map_err(
                    to_io_error,
                )?;
                buffer.status = BufferStatus::Clean;
            });
        }

        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        WRITE_BUFFER.with(|buffer| {
            if buffer.status != BufferStatus::Dirty {
                return Ok(());
            }

            // Write current block
            let off = block_align(self.cursor);
            self.strand.write(off, &buffer[..]).map_err(to_io_error)?;
            buffer.status = BufferStatus::Clean;
        });

        Ok(())
    }
}

impl<'s, 'd> Drop for StrandWriter<'s, 'd> {
    fn drop(&mut self) {
        self.flush().expect("Flush during drop failed");
    }
}
