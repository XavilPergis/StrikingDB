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

/// How to open the datastore.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpenMode {
    /// Open the datastore that already exists on disk.
    Read,

    /// Create a new datastore.
    Create,

    /// The same as `Create`, but executes a trim on the entire
    /// disk before formatting the volume.
    Truncate,
}

impl Default for OpenMode {
    fn default() -> Self {
        OpenMode::Read
    }
}

/// Specify options when opening a datastore.
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct OpenOptions {
    /// What mode to open the volume in.
    /// See [`OpenMode`].
    ///
    /// [`OpenMode`]: enum.OpenMode.html
    pub mode: OpenMode,

    /// How many strands to use when creating the
    /// volume. If `None`, then a default value is
    /// calculated based on the device.
    ///
    /// This option is ignored if not creating a
    /// new datastore.
    pub strands: Option<u16>,

    /// How many items to keep in the item cache.
    /// If `None`, then use a default value.
    pub read_cache: Option<usize>,

    /// If this is `true`, then ignore the indexer
    /// as written on disk, and instead rebuild it
    /// from the items actually on disk.
    pub reindex: bool,
}

impl OpenOptions {
    /// Creates a new `OpenOptions` struct.
    pub fn new() -> Self {
        OpenOptions::default()
    }

    /// Sets the mode to [`OpenMode::Create`], and
    /// returns `&mut self` for chaining methods.
    ///
    /// [`OpenMode::Create`]: enum.OpenMode.html
    pub fn create(&mut self) -> &mut Self {
        self.mode = OpenMode::Create;
        self
    }

    /// Sets the mode to [`OpenMode::Truncate`], and
    /// returns `&mut self` for chaining methods.
    ///
    /// [`OpenMode::Truncate`]: enum.OpenMode.html
    pub fn truncate(&mut self) -> &mut Self {
        self.mode = OpenMode::Truncate;
        self
    }

    /// Sets the number of strands, and returns `&mut self`
    /// for chaining methods.
    pub fn strands(&mut self, strands: u16) -> &mut Self {
        self.strands = Some(strands);
        self
    }

    /// Sets the size of the read cache, and returns `&mut self`
    /// for chaining methods.
    pub fn read_cache(&mut self, items: usize) -> &mut Self {
        self.read_cache = Some(items);
        self
    }

    /// Indicates to recreate the indexer, and returns `&mut self`
    /// for chaining methods.
    pub fn reindex(&mut self) -> &mut Self {
        self.reindex = true;
        self
    }
}
