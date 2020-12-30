#![no_std]

/// Library for common stuff between the bootloader and the kernel
/// Because the bootloader and kernel might use diffrent file format i.e
/// the bootloader is a PE executable and the kernel might be ELF we need
/// to ensure that the bootinfo struct is the same for both

extern crate uefi;

use uefi::memory::{ EFIMemoryMap };

#[derive(Debug)]
#[repr(C)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub pixels_per_scanline: u32,

    pub base: u64,
    pub size: u64,
}

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo<'a> {
    pub framebuffer: Framebuffer,
    pub memory_map: EFIMemoryMap<'a>
}
