/*
 * device/windows.rs
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

use std::fs::File;
use std::{io, mem};
use std::os::windows::prelude::*;
use super::{Device, Error, Result};
use winapi::minwindef::*;
use winapi::{kernel32, winioctl, winnt};

#[repr(C)]
#[derive(Debug, Default, Clone)]
struct DISK_GEOMETRY {
    pub Cylinders: LARGE_INTEGER,
    pub MediaType: MEDIA_TYPE,
    pub TracksPerCylinder: DWORD,
    pub SectorsPerTrack: DWORD,
    pub BytesPerSector: DWORD,
}

#[derive(Debug)]
pub struct Ssd {
    file: File,
    capacity: u64,
    block: bool,
}

impl Ssd {
    unsafe fn get_block_capacity(handle: RawHandle) -> Result<u64> {
        let mut dg = DISK_GEOMETRY::default();
        let mut bytes = 0;
        let ret = kernel32::DeviceIoControl(
            handle,
            winioctl::IOCTL_DISK_GET_DISK_GEOMETRY,
            &mut dg,
            mem::size_of::<DISK_GEOMETRY>(),
            &mut dg,
            mem::size_of::<DISK_GEOMETRY>(),
            &mut bytes,
            0,
        );

        match ret {
            true => {
                let mut capacity;
                capacity = dg.BytesPerSector;
                capacity *= dg.SectorsPerTrack;
                capacity *= dg.TracksPerCylinder;
                capacity *= dg.Cylinders.QuadPart;
                Ok(capacity)
            },
            false => {
                let last = Some(io::Error::last_os_error());
                Err(Error::Io(last))
            },
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
        } else if metdata.file_type().is_file() {
            Ok((metadata.file_size(), false))
        } else {
            Err(Error::FileType)
        }
    }

    pub fn open(mut file: File) -> Result<Self> {
        let (capacity, block) = Self::get_metadata()?;

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
        unimplemented!();
    }

    fn read(&self, off: u64, buf: &mut [u8]) -> Result<()> {
        unimplemented!();
    }

    fn write(&self, off: u64, buf: &[u8]) -> Result<()> {
        unimplemented!();
    }

    fn trim(&self, off: u64, len: u64) -> Result<()> {
        unimplemented!();
    }
}
