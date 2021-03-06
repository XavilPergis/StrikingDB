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

use self::rentals::{VolumeHeaderRental, StrandHeaderRental};
use super::{MIN_STRANDS, PAGE_SIZE64, VERSION, Error, FilePointer, Result};
use super::alloc::PageAllocator;
use super::buffer::Page;
use super::stats::Stats;
use super::strand::Strand;
use capnp::message::{Builder, ReaderOptions};
use capnp::serialize_packed;
use serial_capnp::{self, strand_header, volume_header};
use std::fmt;

rental! {
    mod rentals {
        use super::*;

        #[rental_mut]
        pub struct VolumeHeaderRental {
            message: Box<Builder<PageAllocator>>,
            header: volume_header::Builder<'message>,
        }

        #[rental_mut]
        pub struct StrandHeaderRental {
            message: Box<Builder<PageAllocator>>,
            header: strand_header::Builder<'message>,
        }
    }
}

pub struct VolumeHeader(VolumeHeaderRental);

impl VolumeHeader {
    pub fn new(strands: u16, state_ptr: Option<FilePointer>) -> Self {
        let message = Builder::new(PageAllocator::new());
        let rental = VolumeHeaderRental::new(Box::new(message), |message| {
            let mut header = message.init_root::<volume_header::Builder>();

            header.set_signature(serial_capnp::VOLUME_MAGIC);
            header.set_strands(strands);
            header.set_state_ptr(state_ptr.unwrap_or(0));

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
        if strands <= MIN_STRANDS {
            return Err(Error::Corrupt);
        }

        let state_ptr = Self::null(header.get_state_ptr());

        Ok(Self::new(strands, state_ptr))
    }

    #[allow(unused)]
    pub fn write(self, page: &mut Page) -> Result<()> {
        let mut slice = &mut page[..];
        serialize_packed::write_message(&mut slice, &*self.0.into_head())?;
        Ok(())
    }

    fn null(ptr: u64) -> Option<FilePointer> {
        match ptr {
            0 => None,
            x => Some(x),
        }
    }

    pub fn get_strands(&self) -> u16 {
        self.0.rent(
            |message| message.borrow_as_reader().get_strands(),
        )
    }

    pub fn get_state_ptr(&self) -> Option<FilePointer> {
        self.0.rent(|message| {
            Self::null(message.borrow_as_reader().get_state_ptr())
        })
    }

    #[allow(unused)]
    pub fn set_strands(&mut self, strands: u16) {
        self.0.rent_mut(|message| message.set_strands(strands));
    }

    #[allow(unused)]
    pub fn set_state_ptr(&mut self, state_ptr: Option<FilePointer>) {
        self.0.rent_mut(|message| {
            message.set_state_ptr(state_ptr.unwrap_or(0))
        });
    }
}

impl fmt::Debug for VolumeHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VolumeHeader {{ .. }}")
    }
}

pub struct StrandHeader(StrandHeaderRental);

impl StrandHeader {
    fn _new(id: u16, capacity: u64, offset: u64, stats: &Stats) -> Self {
        let message = Builder::new(PageAllocator::new());
        let rental = StrandHeaderRental::new(Box::new(message), |message| {
            let mut header = message.init_root::<strand_header::Builder>();

            header.set_signature(serial_capnp::STRAND_MAGIC);
            header.set_id(id);
            header.set_capacity(capacity);
            header.set_offset(offset);

            header.set_stats_read_bytes(stats.read_bytes);
            header.set_stats_written_bytes(stats.written_bytes);
            header.set_stats_trimmed_bytes(stats.trimmed_bytes);
            header.set_stats_buffer_read_bytes(stats.buffer_read_bytes);
            header.set_stats_buffer_written_bytes(stats.buffer_written_bytes);
            header.set_stats_valid_items(stats.valid_items);
            header.set_stats_deleted_items(stats.deleted_items);

            header
        });

        StrandHeader(rental)
    }

    pub fn new(id: u16, capacity: u64) -> Self {
        Self::_new(id, capacity, PAGE_SIZE64, &Stats::default())
    }

    pub fn from(strand: &mut Strand) -> Self {
        Self::_new(
            strand.id(),
            strand.capacity(),
            strand.offset(),
            strand.stats.get_mut(),
        )
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

        let stats = Stats {
            read_bytes: header.get_stats_read_bytes(),
            written_bytes: header.get_stats_written_bytes(),
            trimmed_bytes: header.get_stats_trimmed_bytes(),
            buffer_read_bytes: header.get_stats_buffer_read_bytes(),
            buffer_written_bytes: header.get_stats_buffer_written_bytes(),
            valid_items: header.get_stats_valid_items(),
            deleted_items: header.get_stats_deleted_items(),
        };

        Ok(Self::_new(id, capacity, offset, &stats))
    }

    pub fn write(self, page: &mut Page) -> Result<()> {
        let mut slice = &mut page[..];
        serialize_packed::write_message(&mut slice, &*self.0.into_head())?;
        Ok(())
    }

    #[allow(unused)]
    pub fn get_id(&self) -> u16 {
        self.0.rent(|message| message.borrow_as_reader().get_id())
    }

    #[allow(unused)]
    pub fn get_capacity(&self) -> u64 {
        self.0.rent(
            |message| message.borrow_as_reader().get_capacity(),
        )
    }

    pub fn get_offset(&self) -> u64 {
        self.0.rent(
            |message| message.borrow_as_reader().get_offset(),
        )
    }

    #[allow(unused)]
    pub fn get_stats(&self) -> Stats {
        self.0.rent(|message| {
            let reader = message.borrow_as_reader();

            Stats {
                read_bytes: reader.get_stats_read_bytes(),
                written_bytes: reader.get_stats_written_bytes(),
                trimmed_bytes: reader.get_stats_trimmed_bytes(),
                buffer_read_bytes: reader.get_stats_buffer_read_bytes(),
                buffer_written_bytes: reader.get_stats_buffer_written_bytes(),
                valid_items: reader.get_stats_valid_items(),
                deleted_items: reader.get_stats_deleted_items(),
            }
        })
    }

    #[allow(unused)]
    pub fn set_id(&mut self, id: u16) {
        self.0.rent_mut(|message| message.set_id(id))
    }

    #[allow(unused)]
    pub fn set_capacity(&mut self, capacity: u64) {
        self.0.rent_mut(|message| message.set_capacity(capacity))
    }

    #[allow(unused)]
    pub fn set_offset(&mut self, offset: u64) {
        self.0.rent_mut(|message| message.set_offset(offset))
    }

    #[allow(unused)]
    pub fn set_stats(&mut self, stats: &Stats) {
        self.0.rent_mut(|message| {
            message.set_stats_read_bytes(stats.read_bytes);
            message.set_stats_written_bytes(stats.written_bytes);
            message.set_stats_trimmed_bytes(stats.trimmed_bytes);
            message.set_stats_buffer_read_bytes(stats.buffer_read_bytes);
            message.set_stats_buffer_written_bytes(stats.buffer_written_bytes);
            message.set_stats_valid_items(stats.valid_items);
            message.set_stats_deleted_items(stats.deleted_items);
        })
    }
}

impl fmt::Debug for StrandHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StrandHeader {{ .. }}")
    }
}
