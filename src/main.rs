#![feature(panic_info_message, alloc_error_handler)]

#![no_std]
#![no_main]

extern crate rlibc;
#[macro_use] extern crate alloc;
extern crate uefi;
extern crate option_parser;

use uefi::{ EFIHandle, EFIMemoryType, SimpleTextOutputInterface };
use uefi::{ SystemTable, MemoryDescriptor };
use uefi::{ EFILoadedImageProtocol, EFISimpleFilesystem, EFIFileHandle };
use uefi::{ LOADED_IMAGE_GUID, SIMPLE_FILESYSTEM_GUID };

use option_parser::{ OptionParser, Category };

use core::panic::PanicInfo;

use alloc::alloc::{GlobalAlloc, Layout};
use alloc::vec::Vec;
use alloc::string::{ String, ToString };

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
        println!("[DEBUG]: Allocate {} bytes", layout.size());
        let mut buffer = core::ptr::null_mut();
        TABLE.unwrap()
            .boot_services.allocate_pool(EFIMemoryType::BootServicesData,
                                         layout.size(), &mut buffer);

        buffer
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        println!("[DEBUG]: Deallocate {} bytes", layout.size());
        TABLE.unwrap().boot_services.free_pool(ptr);
    }
}

#[global_allocator]
static A: Allocator = Allocator;

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

fn loaded_image<'a>(table: &SystemTable,
                    handle: EFIHandle) -> &'a EFILoadedImageProtocol<'a> {
    let loaded_image_ptr =
        table.boot_services.handle_protocol(handle, &LOADED_IMAGE_GUID);

    let loaded_image =
        unsafe { &*(loaded_image_ptr as *const EFILoadedImageProtocol) };

    loaded_image
}

fn simple_filesystem<'a>(table: &SystemTable,
                         loaded_image: &'a EFILoadedImageProtocol)
    -> &'a EFISimpleFilesystem
{
    let simple_filesystem_ptr =
        table.boot_services.handle_protocol(loaded_image.device_handle,
                                            &SIMPLE_FILESYSTEM_GUID);

    let simple_filesystem =
        unsafe { &*(simple_filesystem_ptr as *const EFISimpleFilesystem) };

    simple_filesystem
}

fn get_boot_directory(handle: EFIHandle, dirname: &str)
    -> &EFIFileHandle
{
    let table = unsafe { TABLE.unwrap() };

    let loaded_image = loaded_image(&table, handle);
    let simple_filesystem = simple_filesystem(&table, &loaded_image);
    let volume = simple_filesystem.open_volume();

    let handle = volume.open(dirname, 0x0000000000000001, 0x0000000000000001);

    handle
}

fn load_file(directory: &EFIFileHandle, filename: &str)
    -> Option<Vec<u8>>
{
    let file_handle = directory.open(filename,
                                  0x0000000000000001, 0x0000000000000001);

    let file_content = file_handle.read_to_buffer();

    Some(file_content)
}

#[derive(Debug)]
struct BootloaderOptions {
    kernel_font: String,
    kernel_filename: String,
}

impl Default for BootloaderOptions {
    fn default() -> Self {
        Self {
            kernel_font: "font.fnt".to_string(),
            kernel_filename: "kernel.kern".to_string(),
        }
    }
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


    // TODO(patrik): Load the kernel

    let directory = get_boot_directory(image_handle, "EFI\\boot\\");

    let filename = "options.txt";
    println!("Loading: {}", filename);

    let buffer = load_file(directory, filename).unwrap();
    let option_str = core::str::from_utf8(&buffer[..]).unwrap();
    println!("Text:\n{}", option_str);

    let mut bootloader_options = BootloaderOptions::default();

    let mut buffer = [0u8; 2048];
    let mut index = 0;

    let option_parser = OptionParser::new(option_str);
    option_parser.options(|category, key, value| {
        println!("'{:?}': {:?} = {:?}", category, key, value);

        if category == Category::Bootloader {
            match key {
                "load_font" =>
                    bootloader_options.kernel_font = value.to_string(),
                "kernel" =>
                    bootloader_options.kernel_filename = value.to_string(),
                _ => {
                    panic!("Unknown option: '{}'", key);
                }
            }
        } else if category == Category::Kernel {
            buffer[index..index+key.len()].clone_from_slice(&key.as_bytes()[..]);
            index += key.len();

            buffer[index] = b'=';
            index += 1;

            if value.as_bytes()[0] == b'\"' {
                buffer[index..index+value.len()]
                    .clone_from_slice(&value.as_bytes()[..]);
                index += value.len();
            } else {
                buffer[index] = b'\"';
                index += 1;

                buffer[index..index+value.len()]
                    .clone_from_slice(&value.as_bytes()[..]);
                index += value.len();

                buffer[index] = b'\"';
                index += 1;
            }

            buffer[index] = b' ';
            index += 1;
        } else {
            panic!("Unknown category: {:?}", category);
        }

        Some(())
    }).unwrap();

    println!("Bootloader Options: {:#?}", bootloader_options);
    println!("Kernel Options: {}", core::str::from_utf8(&buffer[0..index]).unwrap());

    let filename = bootloader_options.kernel_filename;
    let test_bin = load_file(directory, &filename).unwrap();

    type KernelEntry = extern "sysv64" fn(a: u64, b: u64) -> u64;

    let entry: KernelEntry = unsafe { core::mem::transmute(test_bin.as_ptr()) };
    let result = (entry)(30, 50);
    println!("Kernel Result: {}", result);

    let memory_map = table.boot_services.get_memory_map();
    println!("Num memory map entries: {:#?}", memory_map.entries().count());

    let mut memory_size = 0usize;
    for entry in memory_map.entries() {
        if entry.memory_type == EFIMemoryType::ConventionalMemory ||
            entry.memory_type == EFIMemoryType::BootServicesCode ||
            entry.memory_type == EFIMemoryType::BootServicesData {
            memory_size += entry.number_of_pages as usize;
        }
    }

    println!("Entry #0: {:#?}", memory_map.entries().nth(0).unwrap());
    println!("Total Pages: {}", memory_size);
    println!("Total Memory: {} MiB", (memory_size * 4096) / 1024 / 1024);

    for entry in memory_map.entries() {
        if entry.memory_type == EFIMemoryType::ConventionalMemory ||
            // entry.memory_type == EFIMemoryType::LoaderCode ||
            // entry.memory_type == EFIMemoryType::LoaderData ||
            entry.memory_type == EFIMemoryType::BootServicesCode ||
            entry.memory_type == EFIMemoryType::BootServicesData
        {
            let start = entry.physical_start.0;
            let end = (entry.physical_start.0 + entry.number_of_pages * 4096) - 1;
            let size = end - start + 1;

            print!("[0x{:016x}-0x{:016x}] ", start, end);

            if size > 1024 * 1024 {
                print!("{:>4} MiB ({:>10} B)", size / 1024 / 1024, size);
            } else if size > 1024 {
                print!("{:>4} KiB ({:>10} B)", size / 1024, size);
            } else {
                print!("{:>4} B", size);
            }

            println!();
        }
    }

    /*
    exit_boot_services();
    call_kernel_entry(kernel);
    */

    loop {}
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
