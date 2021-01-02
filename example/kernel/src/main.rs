#![feature(alloc_error_handler, panic_info_message, asm)]

#![no_std]
#![no_main]

extern crate rlibc;
extern crate boot_common;
extern crate alloc;
extern crate spin;
extern crate uefi;

mod graphics;

use boot_common::{ BootInfo };
use uefi::memory::{ EFIMemoryMap, EFIMemoryType };

use core::panic::PanicInfo;

use alloc::alloc::{ GlobalAlloc, Layout };

fn print_memory_map(memory_map: &EFIMemoryMap) {
    let mut memory_size = 0usize;
    for entry in memory_map.entries() {
        if entry.memory_type == EFIMemoryType::ConventionalMemory ||
            entry.memory_type == EFIMemoryType::BootServicesCode ||
            entry.memory_type == EFIMemoryType::BootServicesData {
            memory_size += entry.number_of_pages as usize;
        }
    }

    println!("Total Pages: {}", memory_size);
    println!("Total Memory: {} MiB", (memory_size * 4096) / 1024 / 1024);

    for entry in memory_map.entries() {
        if entry.memory_type == EFIMemoryType::ConventionalMemory ||
            entry.memory_type == EFIMemoryType::LoaderCode ||
            entry.memory_type == EFIMemoryType::LoaderData ||
            entry.memory_type == EFIMemoryType::BootServicesCode ||
            entry.memory_type == EFIMemoryType::BootServicesData
        {
            let start = entry.physical_start.0;
            let end =
                (entry.physical_start.0 + entry.number_of_pages * 4096) - 1;
            let size = end - start + 1;

            print!("[0x{:016x}-0x{:016x}] ", start, end);

            if size > 1024 * 1024 {
                print!("{:>4} MiB", size / 1024 / 1024);
                // print!(" ({:>10} B)", size);
            } else if size > 1024 {
                print!("{:>4} KiB", size / 1024);
                // print!(" ({:>10} B)", size);
            } else {
                print!("{:>4} B", size);
            }

            print!(" : {:?}", entry.memory_type);

            println!();
        }
    }
}

#[no_mangle]
#[link_section = ".boot"]
extern fn kernel_entry(boot_info: &'static BootInfo) -> ! {
    graphics::init_graphics(&boot_info.framebuffer);

    println!("Welcome to the Example Kernel");
    print_memory_map(&boot_info.memory_map);

    loop {}
}

struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static A: Allocator = Allocator;

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("---------- KERNEL PANIC ----------");

    if let Some(msg) = info.message() {
        println!("Message: {}", msg);
    }

    if let Some(loc) = info.location() {
        println!("Location: {}:{}:{}",
                 loc.file(), loc.line(), loc.column());
    }

    println!("----------------------------------");
    loop {}
}
