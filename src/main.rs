#![feature(panic_info_message, alloc_error_handler)]

#![no_std]
#![no_main]

extern crate rlibc;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate alloc;

use core::panic::PanicInfo;

use alloc::alloc::{GlobalAlloc, Layout};
use alloc::boxed::Box;

#[repr(C)]
struct SimpleTextOutputInterface {
    reset_fn: usize,

    output_string_fn: unsafe fn(&SimpleTextOutputInterface, *const u16) -> u64,
    test_string_fn: usize,
    
    quary_mode_fn: usize,
    set_mode_fn: usize,
    set_attribute_fn: usize,

    clear_screen_fn: unsafe fn(&SimpleTextOutputInterface) -> u64,
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

    // Used to force Rust to use u32 insteed of u8 for this enum
    MaxValue                = 0xffffffff,
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

impl MemoryDescriptor {
    fn empty() -> Self {
        Self {
            typ: EFIMemoryType::ReservedMemoryType,
            pad: 0,
            physical_start: PhysicalAddress(0),
            virtual_start: VirtualAddress(0),
            number_of_pages: 0,
            attribute: EFIMemoryAttribute::NONE,
        }
    }
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
                                 &mut u64, &mut u64, &mut u32) -> u64,
    allocate_pool_fn: unsafe fn(u32, u64, &mut *mut u8),
    free_pool_fn: unsafe fn(*mut u8),

    create_event_fn: usize,
    set_timer_fn: usize,
    wait_for_event_fn: usize,
    signal_event_fn: usize,
    close_event_fn: usize,
    check_event_fn: usize,

    install_protocol_interface_fn: usize,
    reinstall_protocol_interface_fn: usize,
    uninstall_protocol_interface_fn: usize,
    handle_protocol_fn: usize,
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
                            size: usize, buffer: &mut *mut u8) 
    {
        (self.allocate_pool_fn)(memory_type, size as u64, buffer);
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
        let mut buffer = core::ptr::null_mut();
        TABLE.unwrap().boot_services.allocate_pool(4, layout.size(), &mut buffer);

        buffer
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // panic!("Dealloc");
    }
}

#[global_allocator]
static A: EFIAllocator = EFIAllocator;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[no_mangle]
fn efi_main(_image_handle: u64, 
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

    println!("Status: {}", status & !0x8000000000000000);

    let num_entries = map_size / entry_size;
    println!("Entry Size: {}", entry_size);
    println!("Rust Entry Size: {}", core::mem::size_of::<MemoryDescriptor>());

    println!("Enum Size: {}", core::mem::size_of::<EFIMemoryType>());

    println!("Welcome to the potato bootloader v0.1");
    println!("Num entries: {}", num_entries);

    for i in 0..num_entries {
        unsafe {
            let ptr = buffer.as_ptr().offset((i * entry_size) as isize);
            let ptr = ptr as *const MemoryDescriptor;
            println!("Entry: {:#x?}", *ptr);
        }
    }

    // TODO(patrik): Load the kernel

    load_file();

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
