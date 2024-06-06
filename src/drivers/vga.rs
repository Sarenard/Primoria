use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter};

use crate::drivers::tty::GLOBAL_TTY;

const WINDOW_HEIGHT: usize = 480;
const WINDOW_WIDTH: usize = 640;

pub struct VgaWriter {
    vga: Graphics640x480x16,
}

impl VgaWriter {
    pub fn write_char(&mut self, c: char, col: usize, row: usize, color: Color16) {
        self.vga.draw_character(col * 8, row * 8, c, color);
    }
    pub fn clear(&mut self, color: Color16) {
        self.vga.clear_screen(color);
    }
}


lazy_static! {
    pub static ref WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter {
        vga: Graphics640x480x16::new()
    });
}

pub fn init() {
    let writer = WRITER.lock();
    writer.vga.set_mode();
    writer.vga.clear_screen(Color16::Black);
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

