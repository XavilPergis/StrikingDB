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

pub struct Options {
    pub strands: Option<usize>,
    pub cache: Option<usize>,
}

impl Options {
    pub fn new() -> Self {
        Options {
            strands: None,
            cache: None,
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
}
