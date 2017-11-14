/*
 * index/sync.rs
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

use std::cell::UnsafeCell;
use std::fmt::{self, Debug};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::thread;

pub struct CopyRwLock<T: Debug + Copy> {
    value: UnsafeCell<T>,
    busy: AtomicIsize,
}

impl<T: Debug + Copy> CopyRwLock<T> {
    pub fn new(value: T) -> Self {
        CopyRwLock {
            value: UnsafeCell::new(value),
            busy: AtomicIsize::new(0),
        }
    }

    fn get(&self) -> T {
        unsafe { *self.value.get() }
    }

    pub fn try_read_lock(&self) -> Option<T> {
        let hold = self.busy.load(Ordering::Relaxed);
        if hold == -1 {
            return None;
        }

        let prev = self.busy.compare_and_swap(hold, hold + 1, Ordering::Relaxed);
        if prev == hold {
            Some(self.get())
        } else {
            None
        }
    }

    pub fn read_lock(&self) -> T {
        loop {
            if let Some(value) = self.try_read_lock() {
                return value;
            }

            thread::yield_now();
        }
    }

    pub fn read_unlock(&self) {
        assert!(self.busy.fetch_sub(1, Ordering::Relaxed) >= 0);
    }

    pub fn try_write_lock(&self) -> Option<T> {
        let prev = self.busy.compare_and_swap(0, -1, Ordering::Relaxed);
        match prev {
            0 => Some(self.get()),
            _ => None,
        }
    }

    pub fn write_lock(&self) -> T {
        loop {
            if let Some(value) = self.try_write_lock() {
                return value;
            }

            thread::yield_now();
        }
    }

    pub fn write_unlock(&self, new_value: T) {
        unsafe { *self.value.get() = new_value; }
        assert_eq!(self.busy.swap(0, Ordering::Relaxed), -1);
    }
}

impl<T: Debug + Copy> Debug for CopyRwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CopyRwLock(value = {:?}, ", self.get())?;

        match self.busy.load(Ordering::Relaxed) {
            -1 => write!(f, "write locked"),
            0 => write!(f, "free"),
            n => write!(f, "{} read locks", n),
        }?;

        write!(f, ")")
    }
}

unsafe impl<T: Debug + Copy + Send> Send for CopyRwLock<T> {}
unsafe impl<T: Debug + Copy + Send + Sync> Sync for CopyRwLock<T> {}
