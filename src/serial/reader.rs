/*
 * serial/reader.rs
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

use std::cmp;
use std::io::{self, BufRead, Cursor, Read, Seek, SeekFrom};
use super::page::Page;
use super::strand::Strand;
use super::{PAGE_SIZE, FilePointer};

pub type PageReader<'a> = Cursor<&'a Page>;

#[derive(Debug, Clone)]
pub struct StrandReader<'a> {
    strand: &'a Strand,
    buffer: Page,
    start: usize,
    cursor: usize,
}

impl<'a> StrandReader<'a> {
    pub fn new(strand: &'a Strand, ptr: FilePointer) -> Self {
        StrandReader {
            strand: strand,
            buffer: Page::default(),
            start: ptr.checked_sub(strand.start()).unwrap() as usize,
            cursor: 0,
        }
    }

    pub fn into_strand(self) -> &'a Strand {
        self.strand
    }
}
