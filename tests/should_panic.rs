#![no_std]
#![no_main]

use core::panic::PanicInfo;
use primoria::{exit_qemu, sprint, sprintln, QemuExitCode};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    sprintln!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_fail() {
    sprint!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sprintln!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
