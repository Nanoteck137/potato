#![feature(panic_info_message, alloc_error_handler)]

#![no_std]
#![no_main]

extern crate rlibc;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate alloc;

use core::panic::PanicInfo;
use core::ffi::c_void;

use alloc::alloc::{GlobalAlloc, Layout};

type EFIHandle = usize;

#[derive(PartialEq, Debug)]
#[repr(u64)]
#[allow(dead_code)]
enum EFIStatus {
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
struct EFIGuid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

const LOADED_IMAGE_GUID: EFIGuid = EFIGuid { data1: 0x5B1B31A1, data2: 0x9562, data3: 0x11d2, data4: [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B] };

#[repr(C)]
#[derive(Debug)]
struct EFIDevicePathProtocol {
    typ: u8,
    sub_typ: u8,
    length: [u8; 2],
}

#[repr(C)]
struct EFILoadedImageProtocol<'a> {
    revision: u32,
    parent_handle: EFIHandle, 
    system_table: usize,

    device_handle: EFIHandle,
    file_path: &'a EFIDevicePathProtocol,
    reserved: usize,

    load_options_size: u32,
    load_options: usize,

    image_base: usize,
    image_size: u64,
    image_code_type: EFIMemoryType,
    image_data_type: EFIMemoryType,

    unload_fn: usize,
}

const SIMPLE_FILESYSTEM_GUID: EFIGuid = EFIGuid { data1: 0x964e5b22, data2: 0x6459, data3: 0x11d2, data4: [0x8e, 0x39, 0x0, 0xa0, 0xc9, 0x69, 0x72, 0x3b] };
const GET_INFO_GUID: EFIGuid = EFIGuid { data1: 0x09576e92, data2: 0x6d3f, data3: 0x11d2, data4: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b] };

#[repr(C)]
struct EFIFileHandle {
    revision: u64,

    open_fn: unsafe fn(this: &EFIFileHandle, new_handle: &mut *mut EFIFileHandle, filename: *const u16, open_mode: u64, attributes: u64) -> EFIStatus,
    close_fn: usize,
    delete_fn: usize,
    read_fn: unsafe fn(this: &EFIFileHandle, buffer_size: &mut u64, buffer: *mut u8) -> EFIStatus,
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

#[repr(C)]
struct EFISimpleFilesystem {
    revision: u64,
    open_volume_fn: unsafe fn(&EFISimpleFilesystem, &mut *mut EFIFileHandle) -> EFIStatus,
}

#[repr(C)]
struct SimpleTextOutputInterface {
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
    unsafe fn output_string(&self, bytes: &[u16]) {
        (self.output_string_fn)(self, bytes.as_ptr()); 
    }

    unsafe fn clear_screen(&self) {
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
struct PhysicalAddress(u64);

#[derive(Clone, Copy, Debug)]
struct VirtualAddress(u64);

bitflags! {
    struct EFIMemoryAttribute: u64 {
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
enum EFIMemoryType {
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
struct MemoryDescriptor {
    typ: EFIMemoryType,
    pad: u32,
    physical_start: PhysicalAddress,
    virtual_start: VirtualAddress,
    number_of_pages: u64,
    attribute: EFIMemoryAttribute,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct BootServices {
    header: TableHeader,

    raise_tpl_fn: usize,
    restore_tpl_fn: usize,

    allocate_pages_fn: usize,
    free_pages_fn: usize,
    get_memory_map_fn: unsafe fn(&mut u64, *mut MemoryDescriptor, 
                                 &mut u64, &mut u64, &mut u32) -> EFIStatus,
    allocate_pool_fn: unsafe fn(u32, u64, &mut *mut u8) -> EFIStatus,
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

    handle_protocol_fn: fn(EFIHandle, &EFIGuid, &mut *mut c_void) -> EFIStatus,
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
    unsafe fn allocate_pool(&self, memory_type: u32, 
                            size: usize, buffer: &mut *mut u8) -> EFIStatus
    {
        (self.allocate_pool_fn)(memory_type, size as u64, buffer)
    }

    unsafe fn free_pool(&self, buffer: *mut u8) -> EFIStatus {
        (self.free_pool_fn)(buffer)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SystemTable<'a> {
    header: TableHeader,

    firmware_vendor: usize,
    firmware_revision: u32,

    console_in_handle: usize,
    con_in: usize,

    console_out_handle: usize,
    console_out: &'a SimpleTextOutputInterface,

    standard_error_handle: usize,
    stderr: &'a SimpleTextOutputInterface,

    runtime_services: usize,
    boot_services: &'a BootServices,

    number_of_table_entries: u64,
    configuration_table: usize,
}

struct TextWriter<'a> {
    output: &'a SimpleTextOutputInterface,
}

impl<'a> TextWriter<'a> {
    fn new(output: &'a SimpleTextOutputInterface) -> Self {
        Self {
            output
        }
    }

    fn print(&mut self, s: &str) {
        let mut arr = [0u16; 1024];
        let mut p = 0;

        for c in s.bytes() {
            if c == b'\n' {
                arr[p] = b'\r' as u16;
                p += 1;

                // TODO(patrik): Check 'p' for overflow and flush the buffer

                arr[p] = b'\n' as u16;
                p += 1;

                // TODO(patrik): Check 'p' for overflow and flush the buffer

                continue;
            }

            arr[p] = c as u16;
            p += 1;

            if p >= arr.len() {
                // TODO(patrik): Flush the buffer
            }
        }

        unsafe {
            self.output.output_string(&arr);
        }
    }
}

impl<'a> core::fmt::Write for TextWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s);
        Ok(())
    }
}

static mut WRITER: Option<TextWriter> = None;
static mut TABLE: Option<SystemTable<'static>> = None;

macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        unsafe {
            match WRITER.as_mut() {
                Some(w) => w.write_fmt(format_args!($($arg)*)).unwrap(),
                None => {},
            }
        }
    });
}

macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

struct EFIAllocator;

unsafe impl GlobalAlloc for EFIAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // println!("[DEBUG]: Allocate {} bytes", layout.size());
        let mut buffer = core::ptr::null_mut();
        TABLE.unwrap().boot_services.allocate_pool(4, layout.size(), &mut buffer);

        buffer
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // println!("[DEBUG]: Deallocate {} bytes", layout.size());
        TABLE.unwrap().boot_services.free_pool(ptr);
    }
}

#[global_allocator]
static A: EFIAllocator = EFIAllocator;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

fn load_file(handle: EFIHandle, filename: &str) -> alloc::vec::Vec<u8> {
    let table = unsafe { TABLE.unwrap() };

    let mut loaded_image_ptr = core::ptr::null_mut();
    let status = (table.boot_services.handle_protocol_fn)(handle, &LOADED_IMAGE_GUID, &mut loaded_image_ptr);
    let loaded_image = unsafe { &*(loaded_image_ptr as *const EFILoadedImageProtocol) };
    println!("Status: {:?}", status);
    println!("Loaded Image Protocol Pointer: {:#?}", loaded_image_ptr);
    // println!("Loaded Image Protocol: {:#x?}", loaded_image);

    let mut simple_filesystem_ptr = core::ptr::null_mut();
    let status = (table.boot_services.handle_protocol_fn)(loaded_image.device_handle, &SIMPLE_FILESYSTEM_GUID, &mut simple_filesystem_ptr);
    let simple_filesystem = unsafe { &*(simple_filesystem_ptr as *const EFISimpleFilesystem) };
    println!("Status: {:?}", status);
    println!("Simple Filesystem Protocol Pointer: {:#?}", simple_filesystem_ptr);
    // println!("Simple Filesystem Protocol: {:#x?}", simple_filesystem);

    let mut volume_ptr = core::ptr::null_mut();
    let status = unsafe {
        (simple_filesystem.open_volume_fn)(simple_filesystem, &mut volume_ptr)
    };
    let volume = unsafe { &*volume_ptr };
    println!("Status: {:?}", status);
    println!("Volume Pointer: {:#x?}", volume_ptr);

    let mut buf = [0u16; 1024];

    let mut index = 0;
    for c in filename.bytes() {
        buf[index] = c as u16;
        index += 1;
    }

    buf[index] = 0u16;

    let mut handle_ptr = core::ptr::null_mut();
    let status = unsafe {
        (volume.open_fn)(volume, &mut handle_ptr, buf.as_ptr(), 0x0000000000000001, 0x0000000000000001)
    };
    let handle = unsafe { &*handle_ptr };
    println!("Status: {:?}", status);

    if status == EFIStatus::Success {
        println!("Found the file");
    }

    let mut buffer_size = 0u64;
    let status = unsafe {
        (handle.get_info_fn)(handle, &GET_INFO_GUID, &mut buffer_size, core::ptr::null_mut())
    };

    println!("Get Info Status: {:?}", status);

    let mut buffer = vec![0u8; buffer_size as usize];
    println!("Allocated buffer size: {}", buffer.len());
    println!("Allocated buffer capacity: {}", buffer.capacity());

    let buffer_ptr = buffer.as_mut_ptr();
    println!("Old Buffer: {:?}", buffer);
    println!("Buffer Pointer: {:#?}", buffer_ptr);

    let status = unsafe {
        (handle.get_info_fn)(handle, &GET_INFO_GUID, &mut buffer_size, buffer_ptr)
    };

    let file_size = unsafe { *(buffer.as_ptr() as *const u64).offset(1) };
    println!("Buffer: {:?}", buffer);
    println!("File Size: {}", file_size);
    println!("Get Info Status: {:?}", status);

    let mut file_content = vec![0; file_size as usize];
    let mut read_size = file_size;
    let status = unsafe {
        (handle.read_fn)(handle, &mut read_size, file_content.as_mut_ptr())
    };
    println!("Read Status: {:?}, {}", status, read_size);

    file_content
}

#[no_mangle]
fn efi_main(image_handle: EFIHandle, 
            system_table: *const SystemTable<'static>) -> u64 
{
    let table = unsafe { &*system_table };
    
    unsafe {
        table.console_out.clear_screen();
    }

    unsafe {
        WRITER = Some(TextWriter::new(table.console_out));
        TABLE = Some(*table);
    }

    println!("Welcome to the potato bootloader v0.1");

    let mut map_size = 0;
    let mut map_key = 0;
    let mut entry_size = 0;
    let mut entry_version = 0;

    let status = unsafe {
        (table.boot_services.get_memory_map_fn)(
            &mut map_size,
            core::ptr::null_mut(),
            &mut map_key,
            &mut entry_size,
            &mut entry_version,
        )
    };

    let size = map_size;
    let mut buffer = vec![0u8; size as usize];

    let ptr = buffer.as_mut_ptr() as *mut MemoryDescriptor;

    let mut map_size = buffer.len() as u64;
    let mut map_key = 0;
    let mut entry_size = 0;
    let mut entry_version = 0;

    let status = unsafe {
        (table.boot_services.get_memory_map_fn)(
            &mut map_size,
            ptr,
            &mut map_key,
            &mut entry_size,
            &mut entry_version,
        )
    };

    println!("Status: {:?}", status);

    let num_entries = map_size / entry_size;

    println!("Num entries: {}", num_entries);

    for i in 0..num_entries {
        unsafe {
            let ptr = buffer.as_ptr().offset((i * entry_size) as isize);
            let ptr = ptr as *const MemoryDescriptor;
        }
    }

    // TODO(patrik): Load the kernel

    let filename = "EFI\\boot\\options.txt";
    println!("Loading: {}", filename);
    let buffer = load_file(image_handle, filename);
    println!("Buffer: {:?}", buffer);

    /*
    let kernel_options = load_kernel_options();
    let kernel = load_kernel(kernel_options);
    exit_boot_services();
    call_kernel_entry(kernel);
    */

    0
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("---------- BOOTLOADER PANIC ----------");

    if let Some(msg) = info.message() {
        println!("Message: {}", msg);
    }

    if let Some(loc) = info.location() {
        println!("Location: {}:{}:{}", 
                 loc.file(), loc.line(), loc.column());
    }


    println!("--------------------------------------");

    loop {}
}
