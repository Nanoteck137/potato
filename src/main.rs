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
struct SystemTable<'a> {
    header: SystemTableHeader,

    firmware_vendor: u64,
    firmware_revision: u32,

    console_in_handle: u64,
    con_in: u64,

    console_out_handle: u64,
    console_out: &'a SimpleTextOutputInterface,

    standard_error_handle: u64,
    stderr: u64,

    runtime_services: u64,
    boot_services: u64,

    number_of_table_entries: u64,
    configuration_table: u64,
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

                // TODO(patrik): Check 'p' for overflow

                arr[p] = b'\n' as u16;
                p += 1;

                // TODO(patrik): Check 'p' for overflow

                continue;
            }

            arr[p] = c as u16;
            p += 1;

            if p >= arr.len() {
                // TODO(patrik): Flush the buffer
            }
        }

        unsafe {
            (self.output.output_string)(self.output, arr.as_mut_ptr());
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

#[no_mangle]
fn efi_main(_image_handle: u64, system_table: *const SystemTable<'static>) -> u64 {
    let table = unsafe { &*system_table };

    unsafe {
        WRITER = Some(TextWriter::new(table.console_out));
    }

    print!("Hello World from the bootloader");

    loop {}
        
    // panic!("Test");
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
