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

use super::{FilePointer, Result, StrandReader, StrandWriter};
use super::serial_capnp::item;
use super::strand::Strand;
use capnp::message::{Builder, ReaderOptions};
use capnp::serialize_packed;
use std::cmp::min;
use std::io::Write;

#[derive(Clone)]
pub struct ReadContext<'a>(item::Reader<'a>);

impl<'a> ReadContext<'a> {
    #[inline]
    pub fn key(&self) -> Result<&[u8]> {
        let slice = self.0.get_key()?;
        Ok(slice)
    }

    #[inline]
    pub fn val(&self) -> Result<&[u8]> {
        let slice = self.0.get_value()?;
        Ok(slice)
    }

    fn copy_slice(slice: &[u8], buffer: &mut [u8]) -> usize {
        let len = min(slice.len(), buffer.len());

        let dest = &mut buffer[..len];
        let src = &slice[..len];
        dest.copy_from_slice(src);

        len
    }

    #[inline]
    #[allow(unused)]
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

pub fn read_item<F, R>(strand: &Strand, ptr: FilePointer, func: F) -> Result<R>
where
    F: FnOnce(ReadContext) -> Result<R>,
{
    let mut strand_reader = StrandReader::new(strand, ptr);
    let msg_reader = serialize_packed::read_message(&mut strand_reader, ReaderOptions::new())?;
    let item = msg_reader.get_root::<item::Reader>()?;
    let ctx = ReadContext(item);

    // Run callback and return
    Ok(func(ctx)?)
}

pub fn write_item(strand: &mut Strand, key: &[u8], val: &[u8]) -> Result<FilePointer> {
    let mut message = Builder::new_default();
    {
        let mut item = message.init_root::<item::Builder>();
        item.set_key(key);
        item.set_value(val);
    }

    // Write data
    let mut strand_writer = StrandWriter::new(strand);
    let ptr = strand_writer.get_pointer();

    serialize_packed::write_message(&mut strand_writer, &message)?;
    strand_writer.write_metadata()?;
    strand_writer.flush()?;

    Ok(ptr)
}
