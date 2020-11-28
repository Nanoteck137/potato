#![feature(panic_info_message, alloc_error_handler)]

#![no_std]
#![no_main]

extern crate rlibc;
#[macro_use] extern crate alloc;
extern crate uefi;

use uefi::{ EFIHandle, EFIStatus, SimpleTextOutputInterface };
use uefi::{ SystemTable, MemoryDescriptor };
use uefi::{ EFILoadedImageProtocol, EFISimpleFilesystem };
use uefi::{ GET_INFO_GUID, LOADED_IMAGE_GUID, SIMPLE_FILESYSTEM_GUID };

use core::panic::PanicInfo;

use alloc::alloc::{GlobalAlloc, Layout};
use alloc::vec::Vec;

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

struct Allocator;

unsafe impl GlobalAlloc for Allocator {
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
static A: Allocator = Allocator;

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

fn loaded_image<'a>(table: &SystemTable, handle: EFIHandle) -> &'a EFILoadedImageProtocol<'a> {

    let mut loaded_image_ptr = core::ptr::null_mut();
    let status = unsafe {
        (table.boot_services.handle_protocol_fn)(
            handle,
            &LOADED_IMAGE_GUID,
            &mut loaded_image_ptr)
    };

    if status != EFIStatus::Success {
        panic!("'LoadedImageProtocol' error: {:?}", status);
    }

    let loaded_image =
        unsafe { &*(loaded_image_ptr as *const EFILoadedImageProtocol) };

    loaded_image
}

fn simple_filesystem<'a>(table: &SystemTable,
                         loaded_image: &'a EFILoadedImageProtocol)
    -> &'a EFISimpleFilesystem
{
    let mut simple_filesystem_ptr = core::ptr::null_mut();
    let status = unsafe {
        (table.boot_services.handle_protocol_fn)(
            loaded_image.device_handle,
            &SIMPLE_FILESYSTEM_GUID,
            &mut simple_filesystem_ptr)
    };

    if status != EFIStatus::Success {
        panic!("'SimpleFilesystem' error: {:?}", status);
    }

    let simple_filesystem =
        unsafe { &*(simple_filesystem_ptr as *const EFISimpleFilesystem) };
    // TODO(patrik): Check status

    simple_filesystem
}

fn load_file(handle: EFIHandle, filename: &str) -> Option<Vec<u8>> {
    let table = unsafe { TABLE.unwrap() };

    let loaded_image = loaded_image(&table, handle);
    let simple_filesystem = simple_filesystem(&table, &loaded_image);
    let volume = simple_filesystem.open_volume();

    let mut buf = [0u16; 1024];

    let mut index = 0;
    for c in filename.bytes() {
        buf[index] = c as u16;
        index += 1;
    }

    buf[index] = 0u16;

    let mut handle_ptr = core::ptr::null_mut();
    let status = unsafe {
        // TODO(patrik): Create enums for the open function
        (volume.open_fn)(volume,
                         &mut handle_ptr,
                         buf.as_ptr(),
                         0x0000000000000001,
                         0x0000000000000001)
    };
    let handle = unsafe { &*handle_ptr };
    // TODO(patrik): Check status

    if status == EFIStatus::Success {
        println!("Found the file");
    }

    let mut buffer_size = 0u64;
    let status = unsafe {
        (handle.get_info_fn)(handle,
                             &GET_INFO_GUID,
                             &mut buffer_size,
                             core::ptr::null_mut())
    };
    // TODO(patrik): Check status

    let mut buffer = vec![0u8; buffer_size as usize];
    let buffer_ptr = buffer.as_mut_ptr();

    let status = unsafe {
        (handle.get_info_fn)(handle,
                             &GET_INFO_GUID,
                             &mut buffer_size,
                             buffer_ptr)
    };
    // TODO(patrik): Check status

    let file_size = unsafe { *(buffer.as_ptr() as *const u64).offset(1) };

    let mut file_content = vec![0; file_size as usize];
    let mut read_size = file_size;
    let status = unsafe {
        (handle.read_fn)(handle, &mut read_size, file_content.as_mut_ptr())
    };
    // TODO(patrik): Check status

    Some(file_content)
}

#[no_mangle]
fn efi_main(image_handle: EFIHandle, 
            table: &SystemTable<'static>) -> u64
{
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

    let buffer = load_file(image_handle, filename).unwrap();
    println!("Buffer: {:?}", buffer);
    println!("Text:\n{}", core::str::from_utf8(&buffer[..]).unwrap());

    /*
    let kernel_options = load_kernel_options();
    let kernel = load_kernel(kernel_options);
    let memory_map = table.boot_services.get_memory_map();
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
