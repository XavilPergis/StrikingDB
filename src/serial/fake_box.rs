/*
 * fake_box.rs
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

use stable_deref_trait::StableDeref;
use std::ops::{Deref, DerefMut};

// An unsafe container that acts like a Box<T> with regards
// to having a "stable deref address". This is actually done
// by asserting that the user will _not_ move this object
// or any composite structures that contain this.
pub struct FakeBox<T>(T);

impl<T> FakeBox<T> {
    pub unsafe fn new(object: T) -> Self {
        FakeBox(object)
    }
}

impl<T> Deref for FakeBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for FakeBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

unsafe impl<T> StableDeref for FakeBox<T> {}
