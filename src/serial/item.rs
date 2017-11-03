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

use capnp::message::{Builder, HeapAllocator, Reader, ReaderOptions};
use capnp::serialize_packed;
use self::rentals::{ReadItemRental, WriteItemRental};
use std::cmp::min;
use std::io::Write;
use super::fake_box::FakeBox;
use super::serial_capnp::item;
use super::strand::Strand;
use super::{FilePointer, Result, StrandReader, StrandWriter};

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

rental! {
    mod rentals {
        use super::*;

        #[rental]
        pub struct ReadItemRental<'s, 'd: 's> {
            strand: &'s Strand<'d>,
            reader: FakeBox<StrandReader<'strand>>,
            message: FakeBox<Reader<'reader>>,
            item: item::Reader<'message>,
        }

        #[rental_mut]
        pub struct WriteItemRental {
            message: FakeBox<Builder<HeapAllocator>>,
            item: item::Builder<'message>,
        }
    }
}

/*
            pub fn read<F, R>(strand: &Strand, ptr: FilePointer, func: F) -> Result<R>
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
            */

#[derive(Clone)]
pub struct ReadItem<'s, 'd: 's>(ReadItemRental<'s, 'd>);

impl<'s, 'd: 's> ReadItem<'s, 'd> {
    pub fn new(strand: &'s Strand<'d>, ptr: FilePointer) -> Result<Self> {
        let mut reader = StrandReader::new(strand, ptr);
        let message = serialize_packed::read_message(&mut reader, ReaderOptions::new())?;
        let fbox = unsafe { FakeBox::new(message) };
        let rental = ReadItemRental::new(fbox, |message| {
            message.get_root::<item::Reader>()
        });

        ReadItem(rental)
    }
}

pub struct WriteItem<'s, 'd: 's> {
    strand: &'s mut Strand<'d>,
    rental: WriteItemRental,
}

impl<'s, 'd: 's> WriteItem<'s, 'd> {
    pub fn new(strand: &'s mut Strand<'d>, key: &[u8], val: &[u8]) -> Self {
        let message = Builder::new_default();
        let fbox = unsafe { FakeBox::new(message) };
        let rental = WriteItemRental::new(fbox, |message| {
            let mut item = message.init_root::<item::Builder>();

            item.set_key(key);
            item.set_value(val);

            item
        });

        WriteItem {
            strand: strand,
            rental: rental,
        }
    }

    pub fn write(self) -> Result<FilePointer> {
        let mut writer = StrandWriter::new(self.strand);
        serialize_packed::write_message(&mut writer, &*self.rental.into_head())?;
        writer.write_metadata()?;
        writer.flush()?;

        Ok(writer.get_pointer())
    }
}

#[derive(Debug, Hash, Clone)]
pub struct Item;

impl Item {
    pub fn read<F, R>(strand: &Strand, ptr: FilePointer, func: F) -> Result<R>
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
}
