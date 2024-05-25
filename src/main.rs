#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(primoria::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use primoria::kprintln;

#[no_mangle]
pub extern "C" fn _start() -> ! {

    primoria::init();

    kprintln!("Hello World{}", "!");

    for i in 0..10 {
        kprintln!("n = {}", i);
    }

    #[cfg(test)]
    test_main();

    primoria::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("{}", info);
    primoria::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    primoria::test_panic_handler(info)
}
