#![no_std]
#![allow(dead_code)]

#[macro_use] extern crate bitflags;
#[macro_use] extern crate alloc;

use core::ffi::c_void;

pub type EFIHandle = usize;

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

#[repr(C)]
pub struct EFIGuid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

pub const LOADED_IMAGE_GUID: EFIGuid = EFIGuid { data1: 0x5B1B31A1, data2: 0x9562, data3: 0x11d2, data4: [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B] };

#[repr(C)]
#[derive(Debug)]
pub struct EFIDevicePathProtocol {
    typ: u8,
    sub_typ: u8,
    length: [u8; 2],
}

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

pub const SIMPLE_FILESYSTEM_GUID: EFIGuid = EFIGuid { data1: 0x964e5b22, data2: 0x6459, data3: 0x11d2, data4: [0x8e, 0x39, 0x0, 0xa0, 0xc9, 0x69, 0x72, 0x3b] };
pub const GET_INFO_GUID: EFIGuid = EFIGuid { data1: 0x09576e92, data2: 0x6d3f, data3: 0x11d2, data4: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b] };

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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EFIFileInfo {
    size: u64,
    pub file_size: u64,
    pub physical_size: u64,
    pub create_time: EFITime,
    pub last_access_time: EFITime,
    pub modification_time: EFITime,
    pub attribute: u64,

    // NOTE(patrik): MISSING FILENAME
}

#[repr(C)]
pub struct EFIFileHandle {
    revision: u64,

    open_fn: unsafe fn(this: &EFIFileHandle, new_handle: &mut *mut EFIFileHandle, filename: *const u16, open_mode: u64, attributes: u64) -> EFIStatus,
    close_fn: usize,
    delete_fn: usize,
    pub read_fn: unsafe fn(this: &EFIFileHandle, buffer_size: &mut u64, buffer: *mut u8) -> EFIStatus,
    write_fn: usize,
    get_position_fn: usize,
    set_position_fn: usize,
    get_info_fn: unsafe fn(this: &EFIFileHandle, infomation_type: &EFIGuid, buffer_size: &mut u64, buffer: *mut u8) -> EFIStatus,
    set_info_fn: usize,
    flush_fn: usize,
    open_ex_fn: usize,
    read_ex_fn: usize,
    write_ex_fn: usize,
    flush_ex_fn: usize,
}

impl EFIFileHandle {
    pub fn open<'a>(&self, filename: &str,
                    open_mode: u64, attribute: u64)
        -> &'a EFIFileHandle
    {
        let mut filename_buffer = [0u16; 1024];

        let mut index = 0;
        for c in filename.bytes() {
            filename_buffer[index] = c as u16;
            index += 1;
        }

        filename_buffer[index] = 0u16;

        let mut handle_ptr = core::ptr::null_mut();
        let status = unsafe {
            (self.open_fn)(self, &mut handle_ptr,
                            filename_buffer.as_ptr(), open_mode, attribute)
        };

        if status != EFIStatus::Success {
            panic!("'EFIFileHandle' failed to open '{}' error: {:?}",
                   filename, status)
        }

        let handle = unsafe { &*handle_ptr };

        handle
    }

    pub fn read_to_buffer(&self) -> alloc::vec::Vec<u8> {
        let file_info = self.get_info();
        let mut buffer = vec![0u8; file_info.file_size as usize];

        let mut read_size = file_info.file_size;
        let status = unsafe {
            (self.read_fn)(self, &mut read_size,
                           buffer.as_mut_ptr())
        };

        if status != EFIStatus::Success {
            panic!("'EFIFileHandle::read_to_buffer' failed to read error: {:?}",
                   status);
        }

        buffer
    }

    pub fn get_info(&self) -> EFIFileInfo {
        let mut buffer_size = 0u64;
        let status = unsafe {
            (self.get_info_fn)(self,
                               &GET_INFO_GUID,
                               &mut buffer_size,
                               core::ptr::null_mut())
        };

        if status != EFIStatus::BufferTooSmall {
            panic!("'EFIFileHandle::get_info' expected BufferTooSmall got {:?}",
                   status);
        }

        let mut buffer = vec![0u8; buffer_size as usize];
        let buffer_ptr = buffer.as_mut_ptr();

        let status = unsafe {
            (self.get_info_fn)(self,
                               &GET_INFO_GUID,
                               &mut buffer_size,
                               buffer_ptr)
        };

        if status != EFIStatus::Success {
            panic!("'EFIFIleHandle::get_info' error: {:?}", status);
        }

        let file_info = unsafe { *(buffer.as_ptr() as *const EFIFileInfo) };

        file_info
    }
}

#[repr(C)]
pub struct EFISimpleFilesystem {
    revision: u64,
    open_volume_fn: unsafe fn(&EFISimpleFilesystem, &mut *mut EFIFileHandle) -> EFIStatus,
}

impl EFISimpleFilesystem {
    pub fn open_volume<'a>(&self) -> &'a EFIFileHandle {
        let mut handle_ptr = core::ptr::null_mut();

        let status = unsafe {
            (self.open_volume_fn)(self, &mut handle_ptr)
        };

        if status != EFIStatus::Success {
            panic!("'EFISimpleFilesystem::open_volume' failed to open root volume - error: {:?}", status);
        }

        let handle = unsafe { &*handle_ptr };

        handle
    }
}

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
    pub unsafe fn output_string(&self, bytes: &[u16]) {
        (self.output_string_fn)(self, bytes.as_ptr());
    }

    pub unsafe fn clear_screen(&self) {
        (self.clear_screen_fn)(self);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct TableHeader {
    signature: u64,
    revision: u32,
    header_size: u32,
    crc32: u32,
    reserved: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct PhysicalAddress(u64);

#[derive(Clone, Copy, Debug)]
pub struct VirtualAddress(u64);

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

#[derive(Clone, Copy, Debug)]
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryDescriptor {
    pub typ: EFIMemoryType,
    pad: u32,
    pub physical_start: PhysicalAddress,
    pub virtual_start: VirtualAddress,
    pub number_of_pages: u64,
    pub attribute: EFIMemoryAttribute,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BootServices {
    header: TableHeader,

    raise_tpl_fn: usize,
    restore_tpl_fn: usize,

    allocate_pages_fn: usize,
    free_pages_fn: usize,
    pub get_memory_map_fn: unsafe fn(&mut u64, *mut MemoryDescriptor,
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

    pub handle_protocol_fn: unsafe fn(EFIHandle, &EFIGuid, &mut *mut c_void) -> EFIStatus,
    pc_handle_protocol_fn: usize,
    register_protocol_notify_fn: usize,
    locate_handle_fn: usize,
    locate_device_path_fn: usize,
    install_configuration_table_fn: usize,

    load_image_fn: usize,
    start_image_fn: usize,
    exit_fn: usize,
    unload_image_fn: usize,
    exit_boot_services_fn: usize,

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
    locate_protocol_fn: usize,
    install_multiple_protocol_interfaces_fn: usize,
    uninstall_multiple_protocol_interfaces_fn: usize,

    calculate_crc32_fn: usize,

    copy_mem_fn: usize,
    set_mem_fn: usize,
    create_event_ex_fn: usize,
}

impl BootServices {
    pub fn allocate_pool(&self, memory_type: EFIMemoryType,
                         size: usize, buffer: &mut *mut u8)
    {
        let status = unsafe {
            (self.allocate_pool_fn)(memory_type, size as u64, buffer)
        };

        if status != EFIStatus::Success {
            panic!("Failed to allocate from pool: {:?}", memory_type);
        }
    }

    pub fn free_pool(&self, buffer: *mut u8) {
        let status = unsafe {
            (self.free_pool_fn)(buffer)
        };

        if status != EFIStatus::Success {
            panic!("Failed to free pool");
        }
    }

    pub fn handle_protocol(&self, handle: EFIHandle, guid: &EFIGuid)
        -> *mut c_void
    {
        let mut ptr = core::ptr::null_mut();
        let status = unsafe {
            (self.handle_protocol_fn)(handle, guid, &mut ptr)
        };

        if status != EFIStatus::Success {
            // TODO(patrik): Print the guid
            panic!("Failed to handle protocol");
        }

        ptr
    }
}

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
