#![no_std]
#![no_main]

use core::panic::PanicInfo;


#[no_mangle]
extern fn kernel_entry(a: u64, b: u64) -> u64 {
    a + b + 100
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
