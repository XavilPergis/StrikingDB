/*
 * unix_device.rs
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

use nix::libc;
use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use std::os::unix::prelude::*;
use super::{PAGE_SIZE, Error, Result};

mod ioctl {
    const BLK: u8 = 0x12;
    ioctl!(read blkgetsize64 with BLK, 114; u64);
    ioctl!(write_buf blkdiscard with BLK, 119; [u64; 2]);
}

fn get_metadata(fh: &mut File) -> Result<(u64, bool)> {
    let metadata = fh.metadata()?;
    let ftype = metadata.file_type();

    if ftype.is_block_device() {
        let mut capacity = 0;
        let result = unsafe {
            ioctl::blkgetsize64(fh.as_raw_fd(), &mut capacity)
        };

        match result {
            Ok(_) => Ok((capacity, true)),
            Err(_) => Err(Error::Io),
        }
    } else if ftype.is_file() {
        match fh.seek(SeekFrom::End(0)) {
            Ok(capacity) => Ok((capacity, false)),
            Err(_) => Err(Error::Io),
        }
    } else {
        Err(Error::FileType)
    }
}

#[derive(Debug)]
pub struct Device {
    fh: File,
    capacity: u64,
    block: bool,
}

impl Device {
    pub fn new(mut fh: File) -> Result<Self> {
        let (capacity, block) = get_metadata(&mut fh)?;

        Ok(Device {
            fh: fh,
            capacity: capacity,
            block: block,
        })
    }

    pub fn read(&self, off: u64, buf: &mut [u8]) -> Result<()> {
        let len = buf.len() as u64;
        assert_eq!(off % PAGE_SIZE, 0, "Offset not a multiple of the page size");
        assert_eq!(len % PAGE_SIZE, 0, "Length not a multiple of the page size");
        assert!(off + len < self.capacity, "Read is out of bounds");

        match self.fh.read_at(buf, off) {
            Ok(read) => {
                assert_eq!(read, buf.len(), "Did not read full buffer");
                Ok(())
            },
            Err(_) => Err(Error::Io),
        }
    }

    pub fn write(&self, off: u64, buf: &[u8]) -> Result<()> {
        let len = buf.len() as u64;
        assert_eq!(off % PAGE_SIZE, 0, "Offset not a multiple of the page size");
        assert_eq!(len % PAGE_SIZE, 0, "Length not a multiple of the page size");
        assert!(off + len < self.capacity, "Write is out of bounds");

        match self.fh.write_at(buf, off) {
            Ok(written) => {
                assert_eq!(written, buf.len(), "Did not write full buffer");
                Ok(())
            },
            Err(_) => Err(Error::Io),
        }
    }

    pub fn trim(&self, off: u64, len: u64) -> Result<()> {
        assert_eq!(off % PAGE_SIZE, 0, "Offset not a multiple of the page size");
        assert_eq!(len % PAGE_SIZE, 0, "Length not a multiple of the page size");
        assert!(off + len < self.capacity, "Trim is out of bounds");

        if self.block {
            // TODO
            let tuple = [off, len];
            let result = unsafe {
                ioctl::blkdiscard(self.fh.as_raw_fd(), &[tuple])
            };

            match result {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::Io),
            }
        } else {
            let ret = unsafe {
                libc::fallocate(
                    self.fh.as_raw_fd(),
                    0x01 | 0x02,
                    off as libc::off_t,
                    len as libc::off_t,
                )
            };

            match ret {
                0 => Ok(()),
                _ => Err(Error::Io),
            }
        }
    }
}
