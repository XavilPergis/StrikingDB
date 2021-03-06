#
# serial.capnp
#
# striking-db - Persistent key/value store for SSDs.
# Copyright (c) 2017 Maxwell Duzen, Ammon Smith
#
# striking-db is free software: you can redistribute it and/or modify
# it under the terms of the GNU Lesser General Public License as
# published by the Free Software Foundation, either version 2 of
# the License, or (at your option) any later version.
#
# striking-db is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU Lesser General Public
# License along with striking-db.  If not, see <http://www.gnu.org/licenses/>.
#

@0x839d0a2a39d2f712;

using Magic = UInt64;
using FilePointer = UInt64;

const volumeMagic :Magic = 0xe64d26e37a418b16;
const strandMagic :Magic = 0x1a456bf69dbf40c8;
const stateMagic  :Magic = 0xfee968216bc3cbd5;

# The header present on the first page of any
# volume. The values here are checked to ensure
# consistency and sanity, as well as provide
# meta information to generate the datastore
# from disk.
struct VolumeHeader {
    signature @0 :Magic;

    # Represents the version of StrikingDB
    # If this value is incompatible on disk,
    # an error will occur on open
    versionMajor @1 :UInt8;
    versionMinor @2 :UInt8;
    versionPatch @3 :UInt8;

    # The number of strands in this volume
    strands @4 :UInt16;

    # A pointer to where the "datastore state" is
    # stored, a serialized form of the indexer and
    # deleted item tree that is saved when the handle
    # is closed.
    #
    # If this value is 0 (i.e. null) then the indexer
    # and deleted item tree will be recreated from disk.
    statePtr @5 :FilePointer;
}

# The header present on the first page of every
# strand. The values here are used to validate
# it, but also this is where statistics about
# the datastore are stored.
struct StrandHeader {
    signature @0 :Magic;
    id @1 :UInt16;

    # How large this strand is.
    # While this value could be calculated from
    # the number of strands and size of the volume,
    # we instead store it explicitly on disk to
    # ensure we never make a bad assumption and
    # overrun a strand's bounds.
    capacity @2 :UInt64;

    # How many bytes from the start to the free area of the strand
    # Must be updated on every write
    offset @3 :UInt64;

    # Stores various statistics and other numbers of
    # interest about this strand

    # The total number of bytes read from this strand
    statsReadBytes @4 :UInt64;

    # The total number of bytes written to this strand
    statsWrittenBytes @5 :UInt64;

    # The total number of bytes trimmed in this strand
    statsTrimmedBytes @6 :UInt64;

    # The number of bytes logically read from this strand
    statsBufferReadBytes @7 :UInt64;

    # The number of bytes logically written to this strand
    statsBufferWrittenBytes @8 :UInt64;

    # The number of valid items in this strand
    statsValidItems @9 :UInt64;

    # The number of deleted items in this strand awaiting GC
    statsDeletedItems @10 :UInt64;
}

# Represents a single item on a strand
# The key may not be an empty buffer
struct Item {
    key @0 :Data;
    value @1 :Data;
}

# Stores the "state" of the datastore
# Really, it just tracks the indexer
# and deleted items so that it can
# be recreated in memory the next
# time the datastore is opened.
struct DatastoreState {
    signature @0 :Magic;
    index @1 :Map(Data, FilePointer2);
    deleted @2 :List(FilePointer2);
}

# Cap'n proto requires generic parameters
# to be pointers
struct FilePointer2 {
    pointer @0 :FilePointer;
}

# The Cap'n Proto form of a HashMap
# It should not have duplicate keys.
struct Map(Key, Value) {
    entries @0 :List(Entry);

    struct Entry {
        key @0 :Key;
        value @1 :Value;
    }
}
