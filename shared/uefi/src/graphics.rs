use crate::{ EFIGuid, PhysicalAddress };

// GUID for the GraphicsOutputProtocol (GOP)
pub const GRAPHICS_OUTPUT_PROTOCOL_GUID: EFIGuid =
    EFIGuid {
        data1: 0x9042a9de,
        data2: 0x23dc,
        data3: 0x4a38,
        data4: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a]
    };

/// Pixel formats describes how the pixels should be encoded in the framebuffer
/// TODO(patrik): Change the name
#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(C)]
pub enum EFIGraphicsPixelFormat {
    PixelRedGreenBlueReserved8BitPerColor,
    PixelBlueGreenRedReserved8BitPerColor,
    PixelBitMask,
    PixelBltOnly,
    PixelFormatMax
}

/// Infomation about each color channel in a single pixel
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct EFIGraphicsPixelInfomation {
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    reserved_mask: u32,
}

/// Infomation about the framebuffer
/// i.e the width, the height, how the pixels should be encoded and more
#[derive(Debug)]
#[repr(C)]
pub struct EFIGraphicsOutputInfo {
    version: u32,
    width: u32,
    height: u32,
    pixel_format: EFIGraphicsPixelFormat,
    pixel_infomation: EFIGraphicsPixelInfomation,
    pixels_per_scanline: u32,
}

/// The mode the GOP is in, it contains the framebuffer base address
/// and infomation about the framebuffer
#[repr(C)]
pub struct EFIGraphicsOutputMode<'a> {
    max_mode: u32,
    mode: u32,
    pub info: &'a EFIGraphicsOutputInfo,
    size_of_info: u64,
    pub framebuffer_base: PhysicalAddress,
    pub framebuffer_size: u64,
}

/// The GraphicsOutputProtocol handle have function pointers to
/// muniplulate the framebuffer and ways to get the current framebuffer
#[repr(C)]
pub struct EFIGraphicsOutputProtocol<'a> {
    query_mode: usize,
    set_mode: usize,
    blt: usize,
    pub mode: &'a EFIGraphicsOutputMode<'a>,
}
