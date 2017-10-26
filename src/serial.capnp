@0xdc38b4da1f7b85db;

# Top-level
struct Header {
    # Magic constants
    signature @0 :UInt64 = 0x864d26e37a418b16;
    version @1 :Version;

    struct Version {
        major @0 :UInt8;
        minor @1 :UInt8;
        patch @2 :UInt8;
    }

    # Configuration
    strands @2 :UInt32;
    statePtr @3 :DiskPointer;
}

struct StrandHeader {
    magic @0 :UInt32 = 0x1a456bf6;
    id @1 :UInt32;
    capacity @2 :UInt64;
}

struct Item {
    key @0 :Data;
    value @1 :Data;
}

struct DatastoreState {
    index @0 :Map(Data, DiskPointer);
    deleted @1 :List(DiskPointer);
}

# Composite
struct Map(Key, Value) {
    entries @0 :List(Entry);

    struct Entry {
        key @0 :Key;
        value @1 :Value;
    }
}

struct DiskPointer {
    value @0 :UInt64;
}
