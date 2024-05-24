#![no_std]
#![no_main]

#![allow(dead_code)]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod drivers;
mod system;

#[cfg(not(test))] 
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("{}", info);
    loop {}
}

#[cfg(test)] 
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sprintln!("[failed]\n");
    sprintln!("Error: {}\n", info);
    drivers::qemu::exit_qemu(drivers::qemu::QemuExitCode::Failed);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();
    
    #[cfg(not(test))]
    main();

    loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    sprintln!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    drivers::qemu::exit_qemu(drivers::qemu::QemuExitCode::Success);
}

fn main() {
    let a = &[1, 2];
    let _c = a[3];
}

#[test_case]
fn trivial_assertion() {
    sprint!("trivial assertion... ");
    assert_eq!(1, 1);
    sprintln!("[ok]");
}

