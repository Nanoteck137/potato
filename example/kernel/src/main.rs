#![feature(alloc_error_handler, panic_info_message)]

#![no_std]
#![no_main]

extern crate boot_common;
extern crate alloc;

use boot_common::BootInfo;

use core::panic::PanicInfo;

use alloc::alloc::{ GlobalAlloc, Layout };

#[no_mangle]
extern fn kernel_entry(boot_info: &BootInfo) -> u32 {
    boot_info.framebuffer.width
}

struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}

#[global_allocator]
static A: Allocator = Allocator;

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
