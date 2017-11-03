/*
 * options.rs
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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpenMode {
    Read,
    Create,
    Truncate,
}

impl Default for OpenMode {
    fn default() -> Self {
        OpenMode::Read
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct OpenOptions {
    pub mode: OpenMode,
    pub strands: Option<u16>,
    pub read_cache: Option<usize>,
    pub reindex: bool,
}

impl OpenOptions {
    pub fn new() -> Self {
        OpenOptions::default()
    }

    pub fn create(&mut self) -> &mut Self {
        self.mode = OpenMode::Create;
        self
    }

    pub fn truncate(&mut self) -> &mut Self {
        self.mode = OpenMode::Truncate;
        self
    }

    pub fn strands(&mut self, strands: u16) -> &mut Self {
        self.strands = Some(strands);
        self
    }

    pub fn read_cache(&mut self, items: usize) -> &mut Self {
        self.read_cache = Some(items);
        self
    }

    pub fn reindex(&mut self) -> &mut Self {
        self.reindex = true;
        self
    }
}
