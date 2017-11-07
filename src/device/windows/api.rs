/*
 * device/windows/api.rs
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

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use winapi::LARGE_INTEGER;
pub use winapi::minwindef::*;

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct DISK_GEOMETRY {
    pub Cylinders: LARGE_INTEGER,
    pub MediaType: MEDIA_TYPE,
    pub TracksPerCylinder: DWORD,
    pub SectorsPerTrack: DWORD,
    pub BytesPerSector: DWORD,
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct DISK_GEOMETRY_EX {
    pub Geometry: DISK_GEOMETRY,
    pub DiskSize: LARGE_INTEGER,
    pub Data: UCHAR,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum MEDIA_TYPE {
    Unknown         = 0,
    F5_1Pt2_512     = 1,
    F3_1Pt44_512    = 2,
    F3_2Pt88_512    = 3,
    F3_20Pt8_512    = 4,
    F3_720_512      = 5,
    F5_360_512      = 6,
    F5_320_512      = 7,
    F5_320_1024     = 8,
    F5_180_512      = 9,
    F5_160_512      = 10,
    RemovableMedia  = 11,
    FixedMedia      = 12,
    F3_120M_512     = 13,
    F3_640_512      = 14,
    F5_640_512      = 15,
    F5_720_512      = 16,
    F3_1Pt2_512     = 17,
    F3_1Pt23_1024   = 18,
    F5_1Pt23_1024   = 19,
    F3_128Mb_512    = 20,
    F3_230Mb_512    = 21,
    F8_256_128      = 22,
    F3_200Mb_512    = 23,
    F3_240M_512     = 24,
    F3_32M_512      = 25,
}

impl Default for MEDIA_TYPE {
    fn default() -> Self {
        MEDIA_TYPE::Unknown
    }
}
