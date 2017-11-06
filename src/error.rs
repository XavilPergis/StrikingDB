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

/// A specialized [`Result`] type for this crate.
///
/// [`Result`]: https://doc.rust-lang.org/stable/std/io/type.Result.html
pub type Result<T> = result::Result<T, Error>;

/// The error type returned by nearly all library functions.
/// Generally you should not match it exhaustively.
#[derive(Debug)]
pub enum Error {
    /// The file used to open the datastore with is not a regular file
    /// or block device.
    FileType,

    /// Data read from the volume is corrupt. If this error occurs on open,
    /// the file has likely not been formatted before.
    Corrupt,

    /// An argument passed to a library function is invalid. A string
    /// describing what's wrong with the argument is also returned.
    BadArgument(&'static str),

    /// The given volume is valid, but its on-disk format is of an
    /// incompatible version for this library.
    IncompatibleVersion,

    /// The volume has been exhausted of free space. Generally you
    /// should ensure the disk is 30% larger than your intended data set.
    OutOfSpace,

    /// The item already exists in the datastore.
    ItemExists,

    /// The item was not found in the datastore.
    ItemNotFound,

    /// The key is invalid. Either it is too long, or it has a length of zero.
    InvalidKey,

    /// The value is invalid. It is too long.
    InvalidValue,

    /// This library function, or some aspect of it, has not been implemented yet.
    Unimplemented,

    /// A network failure has occurred.
    Network,

    /// An I/O error has occurred. If known, the associated [`io::Error`] is
    /// also specified.
    ///
    /// [`io::Error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html
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
