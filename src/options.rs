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

pub enum OpenMode {
    Open,
    Create,
    Truncate,
}

pub struct OpenOptions {
    pub strands: Option<usize>,
    pub cache: Option<usize>,
    pub mode: OpenMode,
}

impl OpenOptions {
    pub fn new() -> Self {
        OpenOptions {
            strands: None,
            cache: None,
            mode: OpenMode::Open,
        }
    }

    pub fn strands(&mut self, strands: usize) -> &mut Self {
        self.strands = Some(strands);
        self
    }

    pub fn cache(&mut self, bytes: usize) -> &mut Self {
        self.cache = Some(bytes);
        self
    }

    pub fn create(&mut self) -> &mut Self {
        self.mode = OpenMode::Create;
        self
    }

    pub fn truncate(&mut self) -> &mut Self {
        self.mode = OpenMode::Truncate;
        self
    }
}
