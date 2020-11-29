use crate::{ EFIGuid, PhysicalAddress };

pub const GRAPHICS_OUTPUT_PROTOCOL_GUID: EFIGuid = EFIGuid { data1: 0x9042a9de, data2: 0x23dc, data3: 0x4a38, data4: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a] };

#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(C)]
pub enum EFIGraphicsPixelFormat {
    PixelRedGreenBlueReserved8BitPerColor,
    PixelBlueGreenRedReserved8BitPerColor,
    PixelBitMask,
    PixelBltOnly,
    PixelFormatMax
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct EFIGraphicsPixelInfomation {
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    reserved_mask: u32,
}

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

#[repr(C)]
pub struct EFIGraphicsOutputMode<'a> {
    max_mode: u32,
    mode: u32,
    pub info: &'a EFIGraphicsOutputInfo,
    size_of_info: u64,
    pub framebuffer_base: PhysicalAddress,
    pub framebuffer_size: u64,
}

#[repr(C)]
pub struct EFIGraphicsOutputProtocol<'a> {
    query_mode: usize,
    set_mode: usize,
    blt: usize,
    pub mode: &'a EFIGraphicsOutputMode<'a>,
}
