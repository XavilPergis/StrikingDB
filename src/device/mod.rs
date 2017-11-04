/*
 * device/mod.rs
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

use std::fmt::Debug;
use super::*;

mod memory;
pub use self::memory::Memory;

cfg_if! {
    if #[cfg(unix)] {
        mod unix;
        pub use self::unix::Ssd;
    } else if #[cfg(windows)] {
        mod windows;
        pub use self::windows::Ssd;
    }
}

pub trait Device: Debug {
    fn capacity(&self) -> u64;
    fn block_device(&self) -> bool;
    fn read(&self, off: u64, buf: &mut [u8]) -> Result<()>;
    fn write(&self, off: u64, buf: &[u8]) -> Result<()>;
    fn trim(&self, off: u64, len: u64) -> Result<()>;
}

#[inline(always)]
fn check_read(dev: &Device, off: u64, buf: &[u8]) {
    let len = buf.len() as u64;
    assert_eq!(off % PAGE_SIZE64, 0, "Offset not a multiple of the page size");
    assert_eq!(len % PAGE_SIZE64, 0, "Length not a multiple of the page size");
    assert!(off + len < dev.capacity(), "Read is out of bounds");
}

#[inline(always)]
fn check_write(dev: &Device, off: u64, buf: &[u8]) {
    let len = buf.len() as u64;
    assert_eq!(off % PAGE_SIZE64, 0, "Offset not a multiple of the page size");
    assert_eq!(len % PAGE_SIZE64, 0, "Length not a multiple of the page size");
    assert!(off + len < dev.capacity(), "Write is out of bounds");
}

#[inline(always)]
fn check_trim(dev: &Device, off: u64, len: u64) {
    assert_eq!(off % TRIM_SIZE64, 0, "Offset not a multiple of the trim size");
    assert_eq!(len % TRIM_SIZE64, 0, "Length not a multiple of the trim size");
    assert!(off + len < dev.capacity(), "Trim is out of bounds");
}
