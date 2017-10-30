/*
 * serial/header.rs
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

use capnp::message::{Builder, Reader, ReaderOptions};
use capnp::serialize_packed;
use serial_capnp::{strand_header, volume_header};
use std::io::{Read, Write};
use super::buffer::Page;
use super::serial_capnp;
use super::strand::StrandStats;
use super::{PAGE_SIZE, VERSION, Error, FilePointer, Result};

#[derive(Debug, Clone)]
pub struct VolumeHeader {
    strands: u32,
    pub state_ptr: Option<FilePointer>,
}

impl VolumeHeader {
    pub fn new(strands: u32) -> Self {
        VolumeHeader {
            strands: strands,
            state_ptr: None,
        }
    }

    pub fn read(page: &[u8; PAGE_SIZE as usize]) -> Result<Self> {
        let mut slice = &page[..];
        let msg_reader = serialize_packed::read_message(&mut slice, ReaderOptions::new())?;
        let header = msg_reader.get_root::<volume_header::Reader>()?;

        if header.get_signature() != serial_capnp::VOLUME_MAGIC {
            return Err(Error::Corrupt);
        }

        {
            let version = header.get_version()?;
            let (major, _, _) = *VERSION;

            // Only check the major version for disk format changes
            if version.get_major() != major {
                return Err(Error::IncompatibleVersion);
            }

            let _ = version.get_minor();
            let _ = version.get_patch();
        }

        let strands = header.get_strands();
        let state_ptr = match header.get_state_ptr() {
            ptr => Some(ptr),
            0 => None,
        };

        Ok(VolumeHeader {
            strands: strands,
            state_ptr: state_ptr,
        })
    }

    pub fn write(&self, page: &mut Page) -> Result<()> {
        let mut message = Builder::new_default();
        let mut header = message.init_root::<volume_header::Builder>();

        header.set_signature(serial_capnp::VOLUME_MAGIC);
        header.set_strands(self.strands);
        header.set_state_ptr(self.state_ptr.unwrap_or(0));

        {
            let version = header.borrow().get_version()?;
            let (major, minor, patch) = *VERSION;

            version.set_major(major);
            version.set_minor(minor);
            version.set_patch(patch);
        }

        let mut slice = &mut page[..];
        serialize_packed::write_message(&mut slice, &message)?;

        Ok(())
    }

    #[inline]
    pub fn strands(&self) -> u32 {
        self.strands
    }
}

#[derive(Debug, Clone)]
pub struct StrandHeader {
    id: u32,
    capacity: u64,
    pub offset: u64,
    pub stats: StrandStats,
}

impl StrandHeader {
    pub fn new(id: u32, capacity: u64) -> Self {
        StrandHeader {
            id: id,
            capacity: capacity,
            offset: 0,
            stats: StrandStats::default(),
        }
    }

    pub fn read(page: &Page) -> Result<Self> {
        let mut slice = &page[..];
        let msg_reader = serialize_packed::read_message(&mut slice, ReaderOptions::new())?;
        let header = msg_reader.get_root::<strand_header::Reader>()?;

        if header.get_signature() != serial_capnp::STRAND_MAGIC {
            return Err(Error::Corrupt);
        }

        let id = header.get_id();
        let capacity = header.get_capacity();
        let offset = header.get_offset();
        let stats = {
            let stats = header.get_stats()?;

            StrandStats {
                read_bytes: stats.get_read_bytes(),
                written_bytes: stats.get_written_bytes(),
                valid_items: stats.get_valid_items(),
                deleted_items: stats.get_deleted_items(),
            }
        };

        Ok(StrandHeader {
            id: id,
            capacity: capacity,
            offset: offset,
            stats: stats,
        })
    }

    pub fn write(&self, page: &mut Page) -> Result<()> {
        let mut message = Builder::new_default();
        let mut header = message.init_root::<strand_header::Builder>();

        header.set_signature(serial_capnp::STRAND_MAGIC);
        header.set_id(self.id);
        header.set_capacity(self.capacity);

        {
            let mut stats = header.borrow().get_stats()?;
            stats.set_read_bytes(self.stats.read_bytes);
            stats.set_written_bytes(self.stats.written_bytes);
            stats.set_valid_items(self.stats.valid_items);
            stats.set_deleted_items(self.stats.deleted_items);
        }

        let mut slice = &mut page[..];
        serialize_packed::write_message(&mut slice, &message)?;

        Ok(())
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }
}
