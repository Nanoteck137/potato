#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[repr(C, packed)]
struct SystemTable {
}

#[no_mangle]
pub fn efi_main(image_handle: u64, system_table: u64) -> u64 {
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
