use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter};

use crate::drivers::tty::GLOBAL_TTY;

const WINDOW_HEIGHT: usize = 480;
const WINDOW_WIDTH: usize = 640;

pub fn draw_char(c: char, col: usize, row: usize, color: Color16) {
    VGA.draw_character(col * 8, row * 8, c, color);
}
pub fn clear(color: Color16) {
    VGA.clear_screen(color);
}

const VGA: Graphics640x480x16 = Graphics640x480x16;

pub fn init() {
    VGA.set_mode();
    VGA.clear_screen(Color16::Black);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::drivers::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // to prevent deadlocks
    interrupts::without_interrupts(|| {
        GLOBAL_TTY.lock().write_fmt(args).unwrap();
    });
}

// TESTS
#[test_case]
fn test_println_simple() {
    kprintln!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        kprintln!("test_println_many output");
    }
}
