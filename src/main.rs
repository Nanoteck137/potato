#![feature(panic_info_message, alloc_error_handler)]

#![no_std]
#![no_main]

extern crate rlibc;
#[macro_use] extern crate alloc;
extern crate uefi;

use uefi::{ EFIHandle, EFIStatus, SimpleTextOutputInterface, LOADED_IMAGE_GUID, EFILoadedImageProtocol, SIMPLE_FILESYSTEM_GUID, EFISimpleFilesystem, GET_INFO_GUID, SystemTable, MemoryDescriptor };

use core::panic::PanicInfo;

use alloc::alloc::{GlobalAlloc, Layout};

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
