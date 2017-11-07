/*
 * device/windows/mod.rs
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

use kernel32;
use self::api::*;
use std::fs::File;
use std::os::raw::c_void;
use std::os::windows::fs::FileExt;
use std::os::windows::prelude::*;
use std::path::Path;
use std::{io, mem, ptr};
use super::{Device, Error, Result};
use super::{check_read, check_write, check_trim, open_file};
use winapi::{winioctl, winnt};

mod api;

#[derive(Debug)]
pub struct Ssd {
    file: File,
    capacity: u64,
    block: bool,
}

impl Ssd {
    unsafe fn get_block_capacity(handle: RawHandle) -> Result<u64> {
        let mut dg = DISK_GEOMETRY_EX::default();
        let mut bytes = 0;
        let ret = kernel32::DeviceIoControl(
            handle,
            winioctl::IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
            ptr::null_mut(),
            0,
            &mut dg as *mut _ as *mut c_void,
            mem::size_of::<DISK_GEOMETRY_EX>() as u32,
            &mut bytes,
            ptr::null_mut(),
        );

        if ret != 0 {
            let mut capacity;
            capacity = dg.Geometry.BytesPerSector as i64;
            capacity *= dg.Geometry.SectorsPerTrack as i64;
            capacity *= dg.Geometry.TracksPerCylinder as i64;
            capacity *= dg.Geometry.Cylinders;
            let capacity2 = dg.DiskSize;
            assert_eq!(capacity, capacity2);
            assert!(capacity > 0, "Capacity is negative");
            Ok(capacity as u64)
        } else {
            let last = Some(io::Error::last_os_error());
            Err(Error::Io(last))
        }
    }

    fn get_metadata(file: &mut File) -> Result<(u64, bool)> {
        let metadata = file.metadata()?;
        let attributes = metadata.file_attributes();

        if attributes & winnt::FILE_ATTRIBUTE_DEVICE != 0 {
            let capacity = unsafe {
                Self::get_block_capacity(file.as_raw_handle())?
            };
            Ok((capacity, true))
        } else if metadata.file_type().is_file() {
            Ok((metadata.file_size(), false))
        } else {
            Err(Error::FileType)
        }
    }

    pub fn open(path: &Path) -> Result<Self> {
        let mut file = open_file(path)?;
        let (capacity, block) = Self::get_metadata(&mut file)?;

        Ok(Ssd {
            file: file,
            capacity: capacity,
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

        match self.file.seek_read(buf, off) {
            Ok(read) => {
                assert_eq!(read, buf.len(), "Did not read full buffer");
                Ok(())
            },
            Err(e) => Err(Error::Io(Some(e))),
        }
    }

    fn write(&self, off: u64, buf: &[u8]) -> Result<()> {
        check_write(self, off, buf);

        match self.file.seek_write(buf, off) {
            Ok(written) => {
                assert_eq!(written, buf.len(), "Did not write full buffer");
                Ok(())
            },
            Err(e) => Err(Error::Io(Some(e))),
        }
    }

    fn trim(&self, off: u64, len: u64) -> Result<()> {
        check_trim(self, off, len);

        // I can't figure out how to issue TRIM commands
        // using the winapi.

        Ok(())
    }
}
