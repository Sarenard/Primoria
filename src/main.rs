#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod drivers;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    main();
    loop {}
}

fn main() {
    let a = &[1, 2];
    let c = a[3];
}

