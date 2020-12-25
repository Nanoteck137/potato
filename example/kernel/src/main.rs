#![feature(alloc_error_handler, panic_info_message)]

#![no_std]
#![no_main]

extern crate rlibc;
extern crate boot_common;
extern crate alloc;

use boot_common::{ BootInfo, Framebuffer };

use core::panic::PanicInfo;

use alloc::alloc::{ GlobalAlloc, Layout };

const FONT_BYTES: &'static [u8] = include_bytes!("../res/zap-vga16.psf");

enum PSFFontMode {
    NoUnicode256,
    NoUnicode512,
    Unicode256,
    Unicode512
}

struct PSFFont<'a> {
    bytes: &'a [u8],

    mode: PSFFontMode,
    char_size: u32,
}

impl<'a> PSFFont<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        if bytes[0] != 0x36 && bytes[1] != 0x04 {
            panic!("Invalid Magic for PSF Font version 1");
        }

        let mode = bytes[2];
        let mode = match mode {
            0 => PSFFontMode::NoUnicode256,
            1 => PSFFontMode::NoUnicode512,
            2 => PSFFontMode::Unicode256,
            3 => PSFFontMode::Unicode512,

            _ => panic!("Invalid PSF Font mode"),
        };

        let char_size = bytes[3] as u32;

        Self {
            bytes: &bytes[4..],
            mode,
            char_size
        }
    }

    fn put_char(&self, framebuffer: &Framebuffer, c: char, x: u32, y: u32) {
        let pixel_ptr = framebuffer.base as *mut u32;

        let mut offset = c as usize * self.char_size as usize;

        let x = x as isize;
        let y = y as isize;

        for yoff in 0..16 {
            for xoff in 0..8 {
                let data = self.bytes[offset] as u8;
                if (data & (0b10000000u8.wrapping_shr(xoff as u32))) > 0 {
                    unsafe {
                        let row_offset = (y + yoff) *
                            framebuffer.pixels_per_scanline as isize;
                        let pixel_offset =
                            (x + xoff) + row_offset;
                        *pixel_ptr.offset(pixel_offset) = 0xffffff;
                    }
                }
            }

            offset += 1;
        }
    }
}

#[no_mangle]
#[link_section = ".boot"]
extern fn kernel_entry(boot_info: &BootInfo) -> u32 {

    let font = PSFFont::new(FONT_BYTES);
    font.put_char(&boot_info.framebuffer, 'A', 32, 32);
    font.put_char(&boot_info.framebuffer, 'B', 40, 32);

    0
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
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
