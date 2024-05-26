#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(primoria::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use primoria::kprintln;

extern crate alloc;

use alloc::boxed::Box;

#[no_mangle]
pub extern "C" fn _start() -> ! {

    primoria::init();

    kprintln!("Hello World{}", "!");

    primoria::system::kshell::KSHELL.lock().init();

    #[cfg(test)]
    test_main();

    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);

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
