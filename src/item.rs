use super::FilePointer;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Item {
    // Points to the first byte in the key
    ptr: u64,
    key_len: usize,
    val_len: usize,
}

impl Item {
    fn from_ptr(ptr: FilePointer) -> Self {
        unimplemented!();
    }

    pub fn key(&self, key: &mut [u8]) -> usize {
        unimplemented!();
    }

    pub fn value(&self, value: &mut [u8]) -> usize {
        unimplemented!();
    }
}
