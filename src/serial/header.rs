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

use capnp::serialize_packed;
use capnp::message::ReaderOptions;
use super::serial_capnp;
use super::strand::StrandStats;
use super::{PAGE_SIZE, VERSION, Error, FilePointer, Page, PageReader, Result};

#[derive(Debug, Clone)]
pub struct VolumeHeader {
    strands: u32,
    state_ptr: Option<FilePointer>,
}

impl VolumeHeader {
    pub fn read(page: &Page) -> Result<Self> {
        use serial_capnp::volume_header;

        let mut page_reader = PageReader::new(page);
        let msg_reader = serialize_packed::read_message(&mut page_reader, ReaderOptions::new())?;
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
}

#[derive(Debug, Clone)]
pub struct StrandHeader {
    id: u32,
    capacity: u64,
    offset: u64,
    stats: StrandStats,
}

impl StrandHeader {
    pub fn read(page: &Page) -> Result<Self> {
        use serial_capnp::strand_header;

        let mut page_reader = PageReader::new(page);
        let msg_reader = serialize_packed::read_message(&mut page_reader, ReaderOptions::new())?;
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
}