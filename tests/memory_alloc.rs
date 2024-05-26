#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(primoria::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    primoria::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    primoria::init();

    test_main();

    panic!("Execution continued after stack overflow");
}

use alloc::boxed::Box;

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}
