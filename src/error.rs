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
use std::{error, io, result};
use std::fmt::{self, Display};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FileType,
    Corrupt,
    BadArgument(&'static str),
    IncompatibleVersion,
    OutOfSpace,
    ItemExists,
    ItemNotFound,
    InvalidKey,
    InvalidValue,
    Unimplemented,
    Network,
    Io(Option<io::Error>),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use Error::*;

        match self {
            &FileType => "Invalid file type",
            &OutOfSpace => "Volume is out of space",
            &Corrupt => "Volume is corrupt",
            &BadArgument(desc) => desc,
            &IncompatibleVersion => "Volume is formatted with incompatible version",
            &ItemExists => "Item already exists",
            &ItemNotFound => "Item not found",
            &InvalidKey => "Specified key was invalid",
            &InvalidValue => "Specified value was invalid",
            &Unimplemented => "That operation isn't implemented yet",
            &Network => "General network error",
            &Io(Some(ref err)) => err.description(),
            &Io(None) => "Low level I/O failure",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        use Error::*;

        match self {
            &Io(Some(ref err)) => Some(err),
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
        Error::Io(Some(err))
    }
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        use capnp::ErrorKind::*;

        match err.kind {
            Failed => Error::Corrupt,
            Overloaded => Error::OutOfSpace,
            Disconnected => Error::Network,
            Unimplemented => Error::Unimplemented,
        }
    }
}
