/*
 * serial/item.rs
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
use std::cmp;
use super::serial_capnp::item;
use super::strand::Strand;
use super::{FilePointer, Result, StrandReader};

#[derive(Clone)]
pub struct ReadItem<'a>(item::Reader<'a>);

impl<'a> ReadItem<'a> {
    pub fn read(strand: &'a Strand, ptr: FilePointer) -> Result<Self> {
        let mut strand_reader = StrandReader::new(strand);
        let msg_reader = serialize_packed::read_message(&mut strand_reader, ReaderOptions::new())?;
        let item_reader = msg_reader.get_root::<item::Reader>()?;

        Ok(ReadItem(item_reader))
    }

    #[inline]
    pub fn copy_key(&self, key_buf: &mut [u8]) -> Result<usize> {
        let slice = self.0.get_key()?;
        let len = cmp::min(slice.len(), key_buf.len());

        let dest = &mut key_buf[..len];
        let src = &slice[..len];
        dest.copy_from_slice(src);

        Ok(len)
    }

    #[inline]
    pub fn copy_value(&self, val_buf: &mut [u8]) -> Result<usize> {
        let slice = self.0.get_value()?;
        let len = cmp::min(slice.len(), val_buf.len());

        let dest = &mut val_buf[..len];
        let src = &slice[..len];
        dest.copy_from_slice(src);

        Ok(len)
    }
}

#[derive(Clone)]
// TODO
pub struct WriteItem;
