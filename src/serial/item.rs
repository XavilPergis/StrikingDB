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

use capnp::message::{Reader, ReaderOptions};
use capnp::serialize::OwnedSegments;
use capnp::serialize_packed;
use std::cmp;
use super::serial_capnp::item;
use super::strand::Strand;
use super::{FilePointer, Result, StrandReader};

#[derive(Clone)]
pub struct ReadContext<'a>(item::Reader<'a>);

impl<'a> ReadContext<'a> {
    fn copy_slice(slice: &[u8], buffer: &mut [u8]) -> usize {
        let len = cmp::min(slice.len(), buffer.len());

        let dest = &mut buffer[..len];
        let src = &slice[..len];
        dest.copy_from_slice(src);

        len
    }

    #[inline]
    pub fn copy_key(&self, key_buf: &mut [u8]) -> Result<usize> {
        let slice = self.0.get_key()?;
        Ok(Self::copy_slice(slice, key_buf))
    }

    #[inline]
    pub fn copy_val(&self, val_buf: &mut [u8]) -> Result<usize> {
        let slice = self.0.get_value()?;
        Ok(Self::copy_slice(slice, val_buf))
    }
}

#[derive(Clone)]
pub struct WriteContext;

impl WriteContext {
    // TODO
}

#[derive(Debug, Clone)]
pub struct Item;

impl Item {
    pub fn read<F, T>(strand: &Strand, ptr: FilePointer, func: F) -> Result<T>
        where F: FnOnce(ReadContext) -> T
    {
        let mut strand_reader = StrandReader::new(strand, ptr);
        let msg_reader = serialize_packed::read_message(&mut strand_reader, ReaderOptions::new())?;
        let item_reader = msg_reader.get_root::<item::Reader>()?;
        let ctx = ReadContext(item_reader);

        Ok(func(ctx))
    }

    pub fn write(strand: &mut Strand, key: &[u8], val: &[u8]) -> Result<FilePointer> {
        unimplemented!();
    }
}
