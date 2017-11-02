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
use super::alloc::PageAllocator;
use super::buffer::Page;
use super::fake_box::FakeBox;
use super::serial_capnp;
use super::strand::{Strand, StrandStats};
use super::{PAGE_SIZE, PAGE_SIZE64, VERSION, Error, FilePointer, Result};

rental! {
    mod rentals {
        use super::*;

        #[rental_mut]
        pub struct VolumeHeaderRental {
            message: FakeBox<Builder<PageAllocator>>,
            header: volume_header::Builder<'message>,
        }

        #[rental_mut]
        pub struct StrandHeaderRental {
            message: FakeBox<Builder<PageAllocator>>,
            header: strand_header::Builder<'message>,
        }
    }
}

use self::rentals::{VolumeHeaderRental, StrandHeaderRental};

pub struct VolumeHeader(VolumeHeaderRental);

impl VolumeHeader {
    pub fn new(strands: u16, state_ptr: u64) -> Self {
        let message = Builder::new(PageAllocator::new());
        let fbox = unsafe { FakeBox::new(message) };
        let rental = VolumeHeaderRental::new(fbox, |message| {
            let mut header = message.init_root::<volume_header::Builder>();

            header.set_signature(serial_capnp::VOLUME_MAGIC);
            header.set_strands(strands);
            header.set_state_ptr(state_ptr);

            let (major, minor, patch) = *VERSION;
            header.set_version_major(major);
            header.set_version_minor(minor);
            header.set_version_patch(patch);

            header
        });

        VolumeHeader(rental)
    }

    pub fn read(page: &Page) -> Result<Self> {
        let mut slice = &page[..];
        let msg_reader = serialize_packed::read_message(&mut slice, ReaderOptions::new())?;
        let header = msg_reader.get_root::<volume_header::Reader>()?;

        if header.get_signature() != serial_capnp::VOLUME_MAGIC {
            return Err(Error::Corrupt);
        }

        // Only check the major version for disk format changes
        let (major, _, _) = *VERSION;
        if header.get_version_major() != major {
            return Err(Error::IncompatibleVersion);
        }

        let strands = header.get_strands();
        let state_ptr = header.get_state_ptr();

        Ok(Self::new(strands, state_ptr))
    }
}

#[derive(Debug, Hash, Clone)]
pub struct StrandHeader {
    id: u16,
    capacity: u64,
    pub offset: u64,
    pub stats: StrandStats,
}

impl StrandHeader {
    pub fn new(id: u16, capacity: u64) -> Self {
        StrandHeader {
            id: id,
            capacity: capacity,
            offset: PAGE_SIZE64,
            stats: StrandStats::default(),
        }
    }

    pub fn from(strand: &Strand) -> Self {
        StrandHeader {
            id: strand.id(),
            capacity: strand.capacity(),
            offset: strand.offset(),
            stats: strand.stats.lock().clone(),
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
        let stats = StrandStats {
            read_bytes: header.get_stats_read_bytes(),
            written_bytes: header.get_stats_written_bytes(),
            trimmed_bytes: header.get_stats_trimmed_bytes(),
            buffer_read_bytes: header.get_stats_buffer_read_bytes(),
            buffer_written_bytes: header.get_stats_buffer_written_bytes(),
            valid_items: header.get_stats_valid_items(),
            deleted_items: header.get_stats_deleted_items(),
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
        {
            let mut header = message.init_root::<strand_header::Builder>();
            header.set_signature(serial_capnp::STRAND_MAGIC);
            header.set_id(self.id);
            header.set_capacity(self.capacity);

            header.set_stats_read_bytes(self.stats.read_bytes);
            header.set_stats_written_bytes(self.stats.written_bytes);
            header.set_stats_trimmed_bytes(self.stats.trimmed_bytes);
            header.set_stats_buffer_read_bytes(self.stats.buffer_read_bytes);
            header.set_stats_buffer_written_bytes(self.stats.buffer_written_bytes);
            header.set_stats_valid_items(self.stats.valid_items);
            header.set_stats_deleted_items(self.stats.deleted_items);
        }

        let mut slice = &mut page[..];
        serialize_packed::write_message(&mut slice, &message)?;

        Ok(())
    }

    #[inline]
    pub fn id(&self) -> u16 {
        self.id
    }

    #[inline]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }
}
