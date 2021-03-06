#![no_std]
#![allow(dead_code)]

pub mod graphics;
pub mod fs;
pub mod memory;

use crate::memory::{ EFIMemoryMap, EFIMemoryType, EFIAllocateType };
use crate::memory::MemoryDescriptor;

/// External crates this library uses
#[macro_use] extern crate bitflags;
#[macro_use] extern crate alloc;

use core::ffi::c_void;

/// Declare a EFIHandle type that should be a pointer size
pub type EFIHandle = usize;

/// A struct to represents a PhysicalAddress
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PhysicalAddress(pub u64);

/// A struct to represents a VirtualAddress
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct VirtualAddress(pub u64);

/// EFIStatus used for most of the UEFI API calls to retrive a status,
/// and this enum has most of the warnings and errors that UEFI can report
#[derive(PartialEq, Debug)]
#[repr(u64)]
#[allow(dead_code)]
pub enum EFIStatus {
    Success = 0,

    WarnUnknownGlyph  = 1,
    WarnDeleteFailure = 2,
    WarnWriteFailure  = 3,
    WarnBufferToSmall = 4,

    LoadError           = 0x8000000000000000 | 1,
    InvalidParameter    = 0x8000000000000000 | 2,
    Unsupported         = 0x8000000000000000 | 3,
    BadBufferSize       = 0x8000000000000000 | 4,
    BufferTooSmall      = 0x8000000000000000 | 5,
    NotReady            = 0x8000000000000000 | 6,
    DeviceError         = 0x8000000000000000 | 7,
    WriteProtected      = 0x8000000000000000 | 8,
    OutOfResources      = 0x8000000000000000 | 9,
    VolumeCorrupted     = 0x8000000000000000 | 10,
    VolumeFull          = 0x8000000000000000 | 11,
    NoMedia             = 0x8000000000000000 | 12,
    MediaChanged        = 0x8000000000000000 | 13,
    NotFound            = 0x8000000000000000 | 14,
    AccessDenied        = 0x8000000000000000 | 15,
    NoResponse          = 0x8000000000000000 | 16,
    NoMapping           = 0x8000000000000000 | 17,
    Timeout             = 0x8000000000000000 | 18,
    NotStarted          = 0x8000000000000000 | 19,
    AlreadyStarted      = 0x8000000000000000 | 20,
    Aborted             = 0x8000000000000000 | 21,
    ICMPError           = 0x8000000000000000 | 22,
    TFTPError           = 0x8000000000000000 | 23,
    ProtocolError       = 0x8000000000000000 | 24,
    IncompatibleVersion = 0x8000000000000000 | 25,
    SecurityViolation   = 0x8000000000000000 | 26,
    CRCError            = 0x8000000000000000 | 27,
    EndOfMedia          = 0x8000000000000000 | 28,
    EndOfFile           = 0x8000000000000000 | 31,
    InvalidLanguage     = 0x8000000000000000 | 32,
    CompromisedData     = 0x8000000000000000 | 33,
}

// Represents a UEFI Guid used for protocols mostly
#[repr(C)]
pub struct EFIGuid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

/// GUID for the LoadedImage Protocol
pub const LOADED_IMAGE_GUID: EFIGuid = EFIGuid { data1: 0x5B1B31A1, data2: 0x9562, data3: 0x11d2, data4: [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B] };

#[repr(C)]
#[derive(Debug)]
pub struct EFIDevicePathProtocol {
    typ: u8,
    sub_typ: u8,
    length: [u8; 2],
}

/// LoadedImage protocol has functions to get infomation about a image
/// TODO(patrik): Wrappers and remove pub
#[repr(C)]
pub struct EFILoadedImageProtocol<'a> {
    pub revision: u32,
    pub parent_handle: EFIHandle,
    pub system_table: usize,

    pub device_handle: EFIHandle,
    pub file_path: &'a EFIDevicePathProtocol,
    pub reserved: usize,

    pub load_options_size: u32,
    pub load_options: usize,

    pub image_base: usize,
    pub image_size: u64,
    pub image_code_type: EFIMemoryType,
    pub image_data_type: EFIMemoryType,

    unload_fn: usize,
}

/// A struct to represent time, used for mostly for files
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EFITime {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    pad1: u8,
    nano_second: u32,
    timezone: u16,
    daylight: u8,
    pad2: u8,
}

// A interface to output text to a output interface like the console or serial
#[repr(C)]
pub struct SimpleTextOutputInterface {
    reset_fn: usize,

    output_string_fn: unsafe fn(&SimpleTextOutputInterface, *const u16) -> EFIStatus,
    test_string_fn: usize,

    quary_mode_fn: usize,
    set_mode_fn: usize,
    set_attribute_fn: usize,

    clear_screen_fn: unsafe fn(&SimpleTextOutputInterface) -> EFIStatus,
    set_cursor_position_fn: usize,
    enable_cursor_fn: usize,

    mode_fn: usize,
}

impl SimpleTextOutputInterface {
    /// Output characters on the interface
    pub fn output_string(&self, bytes: &[u16]) {
        // Output the bytes to the interface
        let status = unsafe {
            (self.output_string_fn)(self, bytes.as_ptr())
        };

        // Check the status for success
        if status != EFIStatus::Success {
            // TODO(patrik): Remove panic
            panic!("Failed to output_string");
        }
    }

    /// Clear the screen
    pub fn clear_screen(&self) {
        // Issue the clear command
        let status = unsafe {
            (self.clear_screen_fn)(self)
        };

        if status != EFIStatus::Success {
            panic!("Failed to clear the screen");
        }
    }
}

/// TableHeader for the SystemTable and BootServices
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct TableHeader {
    signature: u64,
    revision: u32,
    header_size: u32,
    crc32: u32,
    reserved: u32,
}

/// All the function pointers for the BootServices
#[derive(Clone, Copy)]
#[repr(C)]
pub struct BootServices {
    header: TableHeader,

    raise_tpl_fn: usize,
    restore_tpl_fn: usize,

    allocate_pages_fn: unsafe fn(EFIAllocateType, EFIMemoryType,
                                 u64, &mut u64) -> EFIStatus,
    free_pages_fn: usize,
    get_memory_map_fn: unsafe fn(&mut u64, *mut MemoryDescriptor,
                                 &mut u64, &mut u64, &mut u32) -> EFIStatus,
    allocate_pool_fn: unsafe fn(EFIMemoryType, u64, &mut *mut u8) -> EFIStatus,
    free_pool_fn: unsafe fn(*mut u8) -> EFIStatus,

    create_event_fn: usize,
    set_timer_fn: usize,
    wait_for_event_fn: usize,
    signal_event_fn: usize,
    close_event_fn: usize,
    check_event_fn: usize,

    install_protocol_interface_fn: usize,
    reinstall_protocol_interface_fn: usize,
    uninstall_protocol_interface_fn: usize,

    handle_protocol_fn: unsafe fn(EFIHandle, &EFIGuid, &mut *mut c_void) -> EFIStatus,
    pc_handle_protocol_fn: usize,
    register_protocol_notify_fn: usize,
    locate_handle_fn: usize,
    locate_device_path_fn: usize,
    install_configuration_table_fn: usize,

    load_image_fn: usize,
    start_image_fn: usize,
    exit_fn: usize,
    unload_image_fn: usize,
    exit_boot_services_fn: unsafe fn(EFIHandle, u64) -> EFIStatus,

    get_next_monotonic_count_fn: usize,
    stall_fn: usize,
    set_watchdog_timer_fn: usize,

    connect_controller_fn: usize,
    disconnect_controller_fn: usize,

    open_protocol_fn: usize,
    close_protocol_fn: usize,
    open_protocol_infomation_fn: usize,

    protocols_per_handle_fn: usize,
    locate_handle_buffer_fn: usize,
    locate_protocol_fn: unsafe fn(protocol: &EFIGuid, registration: *const c_void, interface: &mut *mut c_void) -> EFIStatus,
    install_multiple_protocol_interfaces_fn: usize,
    uninstall_multiple_protocol_interfaces_fn: usize,

    calculate_crc32_fn: usize,

    copy_mem_fn: usize,
    set_mem_fn: usize,
    create_event_ex_fn: usize,
}

impl BootServices {
    pub fn allocate_pages(&self,
                          allocate_type: EFIAllocateType,
                          memory_type: EFIMemoryType,
                          page_count: u64,
                          address: &mut u64)
    {
        unsafe {
            (self.allocate_pages_fn)(allocate_type, memory_type,
                                     page_count, address);
        }
    }

    /// A Function to allocate from a pool selected by the ´memory_type´
    /// TODO(patrik): Return the pointer insteed
    pub fn allocate_pool(&self, memory_type: EFIMemoryType,
                         size: usize, buffer: &mut *mut u8)
    {
        // Allocate the memory
        let status = unsafe {
            (self.allocate_pool_fn)(memory_type, size as u64, buffer)
        };

        // Check the status
        if status != EFIStatus::Success {
            // TODO(patrik): Remove the panic
            panic!("Failed to allocate from pool: {:?}", memory_type);
        }
    }

    /// Free the memory allocated from a pool
    pub fn free_pool(&self, buffer: *mut u8) {
        let status = unsafe {
            (self.free_pool_fn)(buffer)
        };

        if status != EFIStatus::Success {
            panic!("Failed to free pool");
        }
    }

    /// Handles a protocol and returns the handle pointer
    pub fn handle_protocol(&self, handle: EFIHandle, guid: &EFIGuid)
        -> *mut c_void
    {
        // Pointer for the handle
        let mut ptr = core::ptr::null_mut();

        // Get the handle to the protocol
        let status = unsafe {
            (self.handle_protocol_fn)(handle, guid, &mut ptr)
        };

        // Check the status
        if status != EFIStatus::Success {
            // TODO(patrik): Print the guid
            // TODO(patrik): Remove the panic
            panic!("Failed to handle protocol");
        }

        // Return the handle
        ptr
    }

    /// Exit boot services
    pub fn exit_boot_services(&self,
                              image_handle: EFIHandle,
                              map_key: u64) -> Option<()>
    {
        // Call the function
        let status = unsafe {
            (self.exit_boot_services_fn)(image_handle, map_key)
        };

        // Check the status
        if status == EFIStatus::InvalidParameter {
            None
        } else if status == EFIStatus::Success {
            Some(())
        } else {
            panic!("Exit boot services failed: {:?}", status);
        }
    }

    /// Locate a protocol
    pub fn locate_protocol(&self, protocol: &EFIGuid) -> *mut c_void {
        // Pointer to the protocol
        let mut ptr = core::ptr::null_mut();

        // Get the handle to protocol
        let status = unsafe {
            (self.locate_protocol_fn)(protocol, core::ptr::null_mut(), &mut ptr)
        };

        // Check the status
        if status != EFIStatus::Success {
            // TODO(patrik): Print the guid
            // TODO(patrik): Remove the panic
            panic!("Failed to locate protocol");
        }

        // Return the handle
        ptr
    }

    pub fn get_memory_map_size(&self) -> usize {
        // Create some variables that the memory map call gives us
        let mut map_size = 0;
        let mut map_key = 0;
        let mut entry_size = 0;
        let mut entry_version = 0;

        // Get some infomation about the memory
        let _status = unsafe {
            (self.get_memory_map_fn)(
                &mut map_size,
                core::ptr::null_mut(),
                &mut map_key,
                &mut entry_size,
                &mut entry_version,
            )
        };

        // TODO(patrik): Check the status

        // Allocate a buffer to hold the memory map
        // let mut buffer = vec![0u8; map_size as usize];

        map_size as usize
    }

    /// Get a memory map
    pub fn get_memory_map<'a>(&self, buffer: *mut u8, buffer_size: usize) -> EFIMemoryMap<'a> {

        // Get a pointer to the buffer
        let ptr = buffer as *mut MemoryDescriptor;

        let mut map_size = buffer_size as u64;
        let mut map_key = 0;
        let mut entry_size = 0;
        let mut entry_version = 0;

        // Get the memory map and put it in the buffer
        let status = unsafe {
            (self.get_memory_map_fn)(
                &mut map_size,
                ptr,
                &mut map_key,
                &mut entry_size,
                &mut entry_version,
            )
        };

        if status != EFIStatus::Success {
            panic!("Failed to retrive the memory map: {:?}", status);
        }

        // Return a instance of a memory map struct
        EFIMemoryMap::new(unsafe { core::slice::from_raw_parts(buffer, buffer_size) }, map_size, entry_size, map_key)
    }
}

/// A SystemTable is what UEFI gives you when you first boot and it have
/// all the functions and infomation to boot the OS
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SystemTable<'a> {
    header: TableHeader,

    firmware_vendor: usize,
    firmware_revision: u32,

    console_in_handle: usize,
    con_in: usize,

    console_out_handle: usize,
    pub console_out: &'a SimpleTextOutputInterface,

    standard_error_handle: usize,
    stderr: &'a SimpleTextOutputInterface,

    runtime_services: usize,
    pub boot_services: &'a BootServices,

    number_of_table_entries: u64,
    configuration_table: usize,
}
