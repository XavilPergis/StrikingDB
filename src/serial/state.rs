/*
 * serial/state.rs
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

use capnp::message::{Builder, HeapAllocator, Reader, ReaderOptions};
use capnp::serialize_packed;
use error::Error;
use self::rentals::DatastoreStateRental;
use serial_capnp::{self, datastore_state, map};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use strand::Strand;
use super::deleted::Deleted;
use super::fake_box::FakeBox;
use super::index::Index;
use super::volume::VolumeState;
use super::StrandReader;
use super::{FilePointer, Result};

rental! {
    mod rentals {
        use super::*;

        #[rental_mut]
        pub struct DatastoreStateRental {
            message: FakeBox<Builder<HeapAllocator>>,
            header: datastore_state::Builder<'message>,
        }
    }
}

pub struct DatastoreState(DatastoreStateRental);

impl DatastoreState {
    pub fn read(strand: &Strand, ptr: FilePointer) -> Result<VolumeState> {
        let mut reader = StrandReader::new(strand, ptr);
        let msg_reader = serialize_packed::read_message(&mut reader, ReaderOptions::new())?;
        let state = msg_reader.get_root::<datastore_state::Reader>()?;

        if state.get_signature() != serial_capnp::STATE_MAGIC {
            return Err(Error::Corrupt);
        }

        let index = {
            let mut index = BTreeMap::new();
            let map = state.get_index()?;
            let list = map.get_entries()?;

            for entry in list.iter() {
                let key = {
                    let slice = entry.get_key()?;
                    Vec::from(slice).into_boxed_slice()
                };

                let ptr = entry.get_value()?.get_pointer();

                if let Some(_) = index.insert(key, (ptr, false)) {
                    // Duplicate item in index
                    return Err(Error::Corrupt);
                }
            }

            Index::from(index)
        };

        let deleted = {
            let mut deleted = BTreeSet::new();
            let list = state.get_deleted()?;

            for entry in list.iter() {
                let ptr = entry.get_pointer();

                if !deleted.insert(ptr) {
                    // Duplicate item
                    return Err(Error::Corrupt);
                }
            }

            Deleted::from(deleted)
        };

        Ok(VolumeState::new(index, deleted))
    }
}
