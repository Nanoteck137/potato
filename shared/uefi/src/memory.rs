use crate::{ VirtualAddress, PhysicalAddress };
use alloc::vec::Vec;

/// Flags for the memory attributes
/// TODO(patrik): Change the names
bitflags! {
    pub struct EFIMemoryAttribute: u64 {
        const NONE          = 0x0000000000000000;
        const UC            = 0x0000000000000001;
        const WC            = 0x0000000000000002;
        const WT            = 0x0000000000000004;
        const WB            = 0x0000000000000008;
        const UCE           = 0x0000000000000010;
        const WP            = 0x0000000000001000;
        const RP            = 0x0000000000002000;
        const XP            = 0x0000000000004000;
        const NV            = 0x0000000000008000;
        const MORE_RELIABLE = 0x0000000000010000;
        const RO            = 0x0000000000020000;
        const SP            = 0x0000000000040000;
        const CPU_CRYPTO    = 0x0000000000080000;
        const RUNTIME       = 0x8000000000000000;
    }
}

/// Memory Types
#[derive(PartialEq, Clone, Copy, Debug)]
#[repr(C)]
#[allow(dead_code)]
pub enum EFIMemoryType {
    ReservedMemoryType      = 0x00000000,
    LoaderCode              = 0x00000001,
    LoaderData              = 0x00000002,
    BootServicesCode        = 0x00000003,
    BootServicesData        = 0x00000004,
    RuntimeServicesCode     = 0x00000005,
    RuntimeServicesData     = 0x00000006,
    ConventionalMemory      = 0x00000007,
    UnusableMemory          = 0x00000008,
    ACPIReclaimMemory       = 0x00000009,
    ACPIMemoryNVS           = 0x0000000a,
    MemoryMappedIO          = 0x0000000b,
    MemoryMappedIOPortSpace = 0x0000000c,
    PalCode                 = 0x0000000d,
    PersistentMemory        = 0x0000000e,
}

/// Memory Descritptor represents a chunk of memory with some infomation
/// like the type, start address and more
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryDescriptor {
    pub memory_type: EFIMemoryType,
    pad: u32,
    pub physical_start: PhysicalAddress,
    pub virtual_start: VirtualAddress,
    pub number_of_pages: u64,
    pub attribute: EFIMemoryAttribute,
}

/// A Iterator for the memory map
pub struct EFIMemoryMapIterator<'a> {
    buffer: &'a [u8],

    entry_size: usize,
    num_entries: usize,
    index: usize
}

impl<'a> Iterator for EFIMemoryMapIterator<'a> {
    type Item = MemoryDescriptor;

    /// Get the next memory entry from the map
    fn next(&mut self) -> Option<MemoryDescriptor> {
        // Check if we are in range
        if self.index < self.num_entries {
            unsafe {
                // Calculate the offset inside the map we are
                // and get the pointer for that entry
                let ptr = self.buffer.as_ptr().offset((self.index * self.entry_size) as isize);

                // Cast the pointer to a memory descriptor
                let ptr = ptr as *const MemoryDescriptor;

                // Increment the index
                self.index += 1;

                // Return a copy of the descriptor
                return Some(*ptr);
            }
        }

        None
    }
}

/// Represents a memory map
#[derive(Debug)]
pub struct EFIMemoryMap {
    buffer: Vec<u8>,

    map_size: u64,
    entry_size: u64,
    map_key: u64,
}

impl<'a> EFIMemoryMap {
    /// Create a memory map
    pub(crate) fn new(buffer: Vec<u8>, map_size: u64, entry_size: u64, map_key: u64)
        -> Self
    {
        Self {
            buffer,
            map_size,
            entry_size,
            map_key
        }
    }

    /// Return a new iterator for the memory map
    /// NOTE(patrik): Can be called multiple times
    pub fn entries(&self) -> EFIMemoryMapIterator {
        let num_entries = (self.map_size / self.entry_size) as usize;
        let entry_size = self.entry_size as usize;

        EFIMemoryMapIterator {
            buffer: &self.buffer,
            entry_size: entry_size,
            num_entries,
            index: 0
        }
    }
}
