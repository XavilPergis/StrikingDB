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

use std::error::Error;
use std::fmt::{self, Display};
use std::io;

pub type SResult<T> = Result<T, SError>;

#[derive(Debug)]
pub enum SError {
    Io(io::Error),
    LowLevel,
    FileType,
}

impl Error for SError {
    fn description(&self) -> &str {
        match self {
            &SError::Io(ref err) => err.description(),
            &SError::LowLevel => "Low level I/O operation failure",
            &SError::FileType => "Invalid file type",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            &SError::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl Display for SError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<io::Error> for SError {
    fn from(err: io::Error) -> Self {
        SError::Io(err)
    }
}
