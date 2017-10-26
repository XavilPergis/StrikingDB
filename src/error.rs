/*
 * error.rs
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

use capnp;
use std::fmt::{self, Display};
use std::{error, io, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ItemExists,
    ItemNotFound,
    InvalidKey,
    InvalidValue,
    FileType,
    Corruption,
    Io(io::Error),
    LowLevel,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::ItemExists => "Item already exists",
            &Error::ItemNotFound => "Item not found",
            &Error::FileType => "Invalid file type",
            &Error::Corruption => "Volume is corrupt",
            &Error::Io(ref err) => err.description(),
            &Error::LowLevel => "Low level I/O operation failure",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
