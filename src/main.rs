#![feature(rustc_private)]

#![no_std]
#![no_main]

extern crate rlibc;

use core::panic::PanicInfo;

#[repr(C)]
struct SimpleTextOutputInterface {
    reset: u64,

    output_string: unsafe fn(&SimpleTextOutputInterface, *const u16) -> u64,
    test_string: u64,
    
    quary_mode: u64,
    set_mode: u64,
    set_attribute: u64,

    clear_screen: u64,
    set_cursor_position: u64,
    enable_cursor: u64,

    mode: u64,
}

#[repr(C)]
struct SystemTableHeader {
    signature: u64,
    revision: u32,
    header_size: u32,
    crc32: u32,
    reserved: u32,
}

#[repr(C)]
struct SystemTable {
    header: SystemTableHeader,

    firmware_vendor: u64,
    firmware_revision: u32,

    console_in_handle: u64,
    con_in: u64,

    console_out_handle: u64,
    console_out: *const SimpleTextOutputInterface,

    standard_error_handle: u64,
    stderr: u64,

    runtime_services: u64,
    boot_services: u64,

    number_of_table_entries: u64,
    configuration_table: u64,
}

#[no_mangle]
fn efi_main(image_handle: u64, system_table: *const SystemTable) -> u64 {
    unsafe {
        let table = &*system_table;

        let s = "Hello World from the great bootloader";
        let mut arr = [0u16; 1024];
        let mut p = 0;

        for c in s.bytes() {
            arr[p] = c as u16;
            p += 1;
        }

        ((*table.console_out).output_string)(&*table.console_out, arr.as_mut_ptr());
    }

    loop {}
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
