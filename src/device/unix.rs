/*
 * device/unix.rs
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
use std::io::{Seek, SeekFrom};
use std::os::unix::prelude::*;
use std::path::Path;
use super::utils::align;
use super::{Device, Error, Result};
use super::{check_read, check_write, check_trim, open_file};

mod ioctl {
    const BLK: u8 = 0x12;
    ioctl!(read blkgetsize64 with BLK, 114; u64);
    ioctl!(write_buf blkdiscard with BLK, 119; [u64; 2]);
}

#[derive(Debug)]
pub struct Ssd {
    file: File,
    capacity: u64,
    block: bool,
}

impl Ssd {
    fn get_metadata(file: &mut File) -> Result<(u64, bool)> {
        let metadata = file.metadata()?;
        let ftype = metadata.file_type();

        if ftype.is_block_device() {
            let mut capacity = 0;
            let result = unsafe { ioctl::blkgetsize64(file.as_raw_fd(), &mut capacity) };

            match result {
                Ok(_) => Ok((capacity, true)),
                Err(_) => Err(Error::Io(None)),
            }
        } else if ftype.is_file() {
            match file.seek(SeekFrom::End(0)) {
                Ok(capacity) => Ok((capacity, false)),
                Err(e) => Err(Error::Io(Some(e))),
            }
        } else {
            Err(Error::FileType)
        }
    }

    pub fn open(path: &Path) -> Result<Self> {
        let mut file = open_file(path)?;
        let (capacity, block) = Self::get_metadata(&mut file)?;

        Ok(Ssd {
            file: file,
            capacity: align(capacity),
            block: block,
        })
    }
}

impl Device for Ssd {
    #[inline]
    fn capacity(&self) -> u64 {
        self.capacity
    }

    #[inline]
    fn block_device(&self) -> bool {
        self.block
    }

    fn read(&self, off: u64, buf: &mut [u8]) -> Result<()> {
        check_read(self, off, buf);

        match self.file.read_at(buf, off) {
            Ok(read) => {
                assert_eq!(read, buf.len(), "Did not read full buffer");
                Ok(())
            }
            Err(e) => Err(Error::Io(Some(e))),
        }
    }

    fn write(&self, off: u64, buf: &[u8]) -> Result<()> {
        check_write(self, off, buf);

        match self.file.write_at(buf, off) {
            Ok(written) => {
                assert_eq!(written, buf.len(), "Did not write full buffer");
                Ok(())
            }
            Err(e) => Err(Error::Io(Some(e))),
        }
    }

    fn trim(&self, off: u64, len: u64) -> Result<()> {
        check_trim(self, off, len);

        if self.block {
            // TODO test
            let tuple = [off, len];
            let result = unsafe { ioctl::blkdiscard(self.file.as_raw_fd(), &[tuple]) };

            match result {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::Io(None)),
            }
        } else {
            let ret = unsafe {
                libc::fallocate(
                    self.file.as_raw_fd(),
                    0x01 | 0x02,
                    off as libc::off_t,
                    len as libc::off_t,
                )
            };

            match ret {
                0 => Ok(()),
                _ => Err(Error::Io(None)),
            }
        }
    }
}
